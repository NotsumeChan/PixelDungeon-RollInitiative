//! Scripts Lua de los mods (`docs/mods.md`): `<mod>/script.lua`. Un diálogo los llama con ["lua", función, args...]
//! (condición o efecto, `npcs.rs`): la función recibe el contexto (héroe, NPC, piso, flags) y los args, y devuelve un
//! valor (condición) o una lista de efectos con el formato de los diálogos. No toca la partida directo: los efectos
//! los aplica el juego, así que valen las mismas reglas (y en la LAN sólo corre en el host).
//! Sandbox: sin `io`, `os`, `package`, `debug` ni lectura de archivos; tope de instrucciones por llamada y de memoria.
use std::cell::{Cell, RefCell};

use mlua::{HookTriggers, Lua, LuaOptions, LuaSerdeExt, MultiValue, StdLib, VmState};
use serde_json::Value;

/// Tope por llamada: BUDGET * 10 000 instrucciones (un bucle infinito corta con error, no cuelga el juego).
const BUDGET: u32 = 100;
const MEMORY: usize = 32 * 1024 * 1024;

thread_local! {
    static LUA: RefCell<Option<Lua>> = const { RefCell::new(None) };
}

fn init() -> Lua {
    let lua = Lua::new_with(StdLib::MATH | StdLib::STRING | StdLib::TABLE, LuaOptions::default()).expect("lua");
    let _ = lua.set_memory_limit(MEMORY);
    let g = lua.globals();
    for k in ["dofile", "loadfile", "require"] {
        let _ = g.set(k, mlua::Nil);
    }
    for (name, src) in crate::registry::scripts() {
        if let Err(e) = lua.load(src.as_str()).set_name(format!("{name}/script.lua")).exec() {
            eprintln!("mod {name}/script.lua: {e}");
        }
    }
    lua
}

/// Llama `func(ctx, args...)` de los scripts. `None` si no existe o falla (el error va a la consola). `math.random`
/// sale de `seed` (el rng de la partida): la misma partida da lo mismo.
pub fn call(func: &str, ctx: Value, args: &[Value], seed: i64) -> Option<Value> {
    LUA.with(|cell| {
        let mut slot = cell.borrow_mut();
        let lua = slot.get_or_insert_with(init);
        let f = lua.globals().get::<mlua::Function>(func).ok()?;
        let _ = lua.load(format!("math.randomseed({seed})")).exec();
        let steps = Cell::new(0u32);
        lua.set_hook(HookTriggers::new().every_nth_instruction(10_000), move |_, _| {
            steps.set(steps.get() + 1);
            if steps.get() > BUDGET { Err(mlua::Error::runtime("demasiadas instrucciones")) } else { Ok(VmState::Continue) }
        });
        let mut a = vec![lua.to_value(&ctx).ok()?];
        for x in args {
            a.push(lua.to_value(x).ok()?);
        }
        let r = f.call::<mlua::Value>(MultiValue::from_vec(a));
        lua.remove_hook();
        match r {
            Ok(v) => lua.from_value(v).ok(),
            Err(e) => {
                eprintln!("lua {func}: {e}");
                None
            }
        }
    })
}
