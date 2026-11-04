//! Guardado de partidas en JSON (`game/save.py`): una por personaje, en `<datos>/saves/<id>.json`. Sin código en el
//! archivo (las partidas del host se compartirán en red). Los niveles no se guardan enteros: se regeneran con la seed
//! y se pisa lo que cambia (terreno, lo visto, montones), más los mobs, los estados y el planificador de cada piso
//! visitado (con el combate en curso). Antes de guardar se aplica lo que esperaba al dado (`pending`).
//! Formato propio de Rust (`format: "pdri-rust"`): los guardados del prototipo no se leen (se apartan, no se borran).
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::actor::{ActorId, Scheduler};
use crate::buffs::Buff;
use crate::char::Char;
use crate::characters::{now, write_atomic};
use crate::dungeon::{Dungeon, Floor, Rules};
use crate::hero::Hero;
use crate::items::{Heap, HeapKind, Item};
use crate::level::Level;
use crate::mobs::{Mob, State};
use crate::rng::Rng;

/// Subirla si cambia el formato; uno que no se puede leer se aparta (no se borra).
const VERSION: i64 = 1;
const FORMAT: &str = "pdri-rust";

fn path(dir: &Path, char_id: &str) -> PathBuf {
    dir.join("saves").join(format!("{char_id}.json"))
}

/// Guarda con escritura atómica. Si no queda ningún héroe vivo, borra el guardado (como el original).
pub fn save_run(dir: &Path, d: &mut Dungeon, char_id: &str) -> std::io::Result<()> {
    if !d.heroes.iter().any(Hero::is_alive) {
        delete_run(dir, char_id);
        return Ok(());
    }
    d.resolve_pending();
    write_atomic(&path(dir, char_id), &dump(d).to_string())
}

/// La partida guardada, o None. Una rota, de otra versión o con contenido que ya no existe se aparta con otro nombre.
pub fn load_run(dir: &Path, char_id: &str) -> Option<Dungeon> {
    let p = path(dir, char_id);
    let text = std::fs::read_to_string(&p).ok()?;
    let data: Option<Value> = serde_json::from_str(&text).ok();
    match data.as_ref().and_then(load) {
        Some(d) => Some(d),
        None => {
            let _ = std::fs::rename(&p, p.with_file_name(format!("{char_id}.json.roto-{}", now())));
            None
        }
    }
}

pub fn delete_run(dir: &Path, char_id: &str) {
    let _ = std::fs::remove_file(path(dir, char_id));
}

// --- volcar ---

fn item_v(i: &Item) -> Value {
    json!([i.id, i.quantity, i.level, i.cursed, i.level_known, i.cursed_known])
}

fn actor_v(a: ActorId) -> Value {
    match a {
        ActorId::Hero(i) => json!(["hero", i]),
        ActorId::Mob(id) => json!(["mob", id]),
        ActorId::Regen(i) => json!(["regen", i]),
        ActorId::Hunger(i) => json!(["hunger", i]),
        ActorId::Buff(uid) => json!(["buff", uid]),
        ActorId::Respawner => json!(["respawner"]),
        ActorId::Trap => json!(["trap"]),
    }
}

fn state_name(s: State) -> &'static str {
    match s {
        State::Sleeping => "SLEEPING",
        State::Wandering => "WANDERING",
        State::Hunting => "HUNTING",
    }
}

fn kind_name(k: HeapKind) -> &'static str {
    match k {
        HeapKind::Heap => "heap",
        HeapKind::Chest => "chest",
        HeapKind::Skeleton => "skeleton",
        HeapKind::Tomb => "tomb",
    }
}

/// Lo de un personaje que cambia (lo demás sale de sus datos).
fn char_v(c: &Char) -> Value {
    let buffs: Vec<Value> = c.buffs.iter().map(|b| json!([b.uid, b.id, b.left, b.level, b.dc])).collect();
    json!({"pos": c.pos, "hp": c.hp, "ht": c.ht, "init": c.init, "moves": c.moves, "has_action": c.has_action,
           "no_moves": c.no_moves, "surprised": c.surprised, "buffs": buffs})
}

fn hero_v(h: &Hero) -> Value {
    json!({"name": h.ch.name, "class": h.class_key, "abilities": h.abilities, "lvl": h.lvl, "exp": h.exp,
           "depth": h.depth, "hunger": h.hunger, "gold": h.gold,
           "weapon": h.weapon.as_ref().map(item_v), "armor": h.armor.as_ref().map(item_v),
           "backpack": h.backpack.iter().map(item_v).collect::<Vec<_>>(), "quick": h.quickslots,
           "second_wind": h.second_wind, "slots": h.slots, "free_casts": h.free_casts,
           "marked": h.marked.map(actor_v), "dead": h.dead, "saves": h.saves, "stable": h.stable,
           "char": char_v(&h.ch)})
}

fn floor_v(f: &Floor) -> Value {
    let l = &f.level;
    let heaps: Vec<Value> =
        l.heaps.iter().map(|(p, h)| json!([p, kind_name(h.kind), h.items.iter().map(item_v).collect::<Vec<_>>()])).collect();
    let mobs: Vec<Value> = f
        .mobs
        .iter()
        .map(|m| json!([m.id, m.key, state_name(m.state), m.target, m.enemy_seen, m.alerted, char_v(&m.ch)]))
        .collect();
    let s = &f.scheduler;
    json!({"now": s.now, "map": l.map, "visited": l.visited, "mapped": l.mapped, "heaps": heaps, "mobs": mobs,
           "actors": s.actors.iter().map(|(a, t)| json!([actor_v(*a), t])).collect::<Vec<_>>(),
           "order": s.order.as_ref().map(|o| o.iter().map(|a| actor_v(*a)).collect::<Vec<_>>()),
           "turn": s.turn, "round": s.round, "calm": s.calm})
}

pub fn dump(d: &Dungeon) -> Value {
    let (mt, i) = d.rng.state();
    json!({"format": FORMAT, "version": VERSION, "saved": now(), "seed": d.seed, "rng": [mt, i],
           "messages": d.messages, "known": d.known, "generator": d.generator.weights(),
           "quotas": [d.potions_of_strength, d.scrolls_of_upgrade],
           "rules": [d.rules.dot_death_saves, d.rules.save_half],
           "next_mob": d.next_mob, "next_buff": d.next_buff,
           "heroes": d.heroes.iter().map(hero_v).collect::<Vec<_>>(), "local": d.hero,
           "floors": d.floors.iter().map(|(k, f)| (k.to_string(), floor_v(f))).collect::<serde_json::Map<_, _>>()})
}

// --- cargar ---

fn unitem(d: &Dungeon, v: &Value) -> Option<Item> {
    let mut i = Item::new(&d.items, v[0].as_str()?)?; // un ítem que ya no existe (un mod que se quitó): no se lee
    i.quantity = v[1].as_i64()? as i32;
    i.level = v[2].as_i64()? as i32;
    i.cursed = v[3].as_bool()?;
    i.level_known = v[4].as_bool()?;
    i.cursed_known = v[5].as_bool()?;
    Some(i)
}

fn unactor(v: &Value) -> Option<ActorId> {
    let n = || v[1].as_u64();
    Some(match v[0].as_str()? {
        "hero" => ActorId::Hero(n()? as usize),
        "mob" => ActorId::Mob(n()? as u32),
        "regen" => ActorId::Regen(n()? as usize),
        "hunger" => ActorId::Hunger(n()? as usize),
        "buff" => ActorId::Buff(n()? as u32),
        "respawner" => ActorId::Respawner,
        "trap" => ActorId::Trap,
        _ => return None,
    })
}

fn load_char(d: &Dungeon, c: &mut Char, v: &Value) -> Option<()> {
    c.pos = v["pos"].as_i64()? as i32;
    c.hp = v["hp"].as_i64()? as i32;
    c.ht = v["ht"].as_i64()? as i32;
    c.init = v["init"].as_i64()? as i32;
    c.moves = v["moves"].as_i64()? as i32;
    c.has_action = v["has_action"].as_bool()?;
    c.no_moves = v["no_moves"].as_bool()?;
    c.surprised = v["surprised"].as_bool()?;
    c.buffs = Vec::new();
    for b in v["buffs"].as_array()? {
        let id = b[1].as_str()?.to_string();
        let data = d.buffs.get(&id)?.clone();
        c.buffs.push(Buff { uid: b[0].as_u64()? as u32, id, data, left: b[2].as_f64(), level: b[3].as_i64()? as i32, dc: b[4].as_i64()? as i32 });
    }
    Some(())
}

fn ints<T: TryFrom<i64>>(v: &Value) -> Option<Vec<T>> {
    v.as_array()?.iter().map(|x| x.as_i64().and_then(|n| T::try_from(n).ok())).collect()
}

fn bools(v: &Value) -> Option<Vec<bool>> {
    v.as_array()?.iter().map(Value::as_bool).collect()
}

pub fn load(data: &Value) -> Option<Dungeon> {
    if data["format"] != FORMAT || data["version"].as_i64() != Some(VERSION) {
        return None;
    }
    let seed = data["seed"].as_i64()?;
    let hds = data["heroes"].as_array()?;
    let ab0: [i32; 6] = ints::<i32>(&hds.first()?["abilities"])?.try_into().ok()?;
    // mismas apariencias de pociones / pergaminos (salen de la seed); el piso 1 que genera se reemplaza abajo
    let mut d = Dungeon::new_with(seed, hds[0]["class"].as_str()?, ab0);
    let mt: Vec<u32> = ints(&data["rng"][0])?;
    d.rng = Rng::from_state(&mt, data["rng"][1].as_u64()? as usize)?;
    d.messages = data["messages"].as_array()?.iter().filter_map(|m| m.as_str().map(str::to_string)).collect();
    d.known = data["known"].as_array()?.iter().filter_map(|k| k.as_str().map(str::to_string)).collect();
    let weights: Vec<f64> = data["generator"].as_array()?.iter().filter_map(Value::as_f64).collect();
    d.generator.set_weights(&weights);
    d.potions_of_strength = data["quotas"][0].as_i64()? as i32;
    d.scrolls_of_upgrade = data["quotas"][1].as_i64()? as i32;
    d.rules = Rules { dot_death_saves: data["rules"][0].as_bool()?, save_half: data["rules"][1].as_bool()? };
    d.next_mob = data["next_mob"].as_u64()? as u32;
    d.next_buff = data["next_buff"].as_u64()? as u32;
    // héroes
    let mut heroes = Vec::new();
    for hd in hds {
        let ab: [i32; 6] = ints::<i32>(&hd["abilities"])?.try_into().ok()?;
        let mut h = Hero::new(&d.classes, &d.items, hd["class"].as_str()?, ab);
        h.ch.name = hd["name"].as_str()?.to_string();
        h.lvl = hd["lvl"].as_i64()? as i32;
        h.exp = hd["exp"].as_i64()? as i32;
        h.depth = hd["depth"].as_i64()? as i32;
        h.hunger = hd["hunger"].as_f64()?;
        h.gold = hd["gold"].as_i64()? as i32;
        h.weapon = if hd["weapon"].is_null() { None } else { Some(unitem(&d, &hd["weapon"])?) };
        h.armor = if hd["armor"].is_null() { None } else { Some(unitem(&d, &hd["armor"])?) };
        h.backpack = hd["backpack"].as_array()?.iter().map(|v| unitem(&d, v)).collect::<Option<_>>()?;
        let q = hd["quick"].as_array()?;
        h.quickslots = [q[0].as_str().map(str::to_string), q[1].as_str().map(str::to_string)];
        h.second_wind = hd["second_wind"].as_bool()?;
        h.slots = ints(&hd["slots"])?;
        h.free_casts =
            hd["free_casts"].as_array()?.iter().map(|f| Some((f[0].as_str()?.to_string(), f[1].as_i64()? as i32))).collect::<Option<_>>()?;
        h.marked = if hd["marked"].is_null() { None } else { Some(unactor(&hd["marked"])?) };
        h.dead = hd["dead"].as_bool()?;
        h.saves = ints::<i32>(&hd["saves"])?.try_into().ok()?;
        h.stable = hd["stable"].as_bool()?;
        load_char(&d, &mut h.ch, &hd["char"])?;
        h.update_awareness();
        heroes.push(h);
    }
    d.heroes = heroes;
    d.hero = data["local"].as_u64()? as usize;
    // pisos visitados: mismo nivel (sale de la seed) y se pisa lo que cambió
    d.floors.clear();
    for (key, f) in data["floors"].as_object()? {
        let depth: i32 = key.parse().ok()?;
        let mut level = Level::new(seed, depth);
        let map: Vec<u8> = ints(&f["map"])?;
        for (c, &t) in map.iter().enumerate() {
            if level.map.get(c) != Some(&t) {
                level.set(c as i32, t);
            }
        }
        level.visited = bools(&f["visited"])?;
        level.mapped = bools(&f["mapped"])?;
        level.heaps.clear();
        for h in f["heaps"].as_array()? {
            let kind = match h[1].as_str()? {
                "chest" => HeapKind::Chest,
                "skeleton" => HeapKind::Skeleton,
                "tomb" => HeapKind::Tomb,
                _ => HeapKind::Heap,
            };
            let items = h[2].as_array()?.iter().map(|v| unitem(&d, v)).collect::<Option<_>>()?;
            level.heaps.insert(h[0].as_i64()? as i32, Heap { kind, items });
        }
        let mut mobs = Vec::new();
        for m in f["mobs"].as_array()? {
            let key = m[1].as_str()?;
            let stats = d.mobs_data.get(key)?.clone();
            let state = match m[2].as_str()? {
                "HUNTING" => State::Hunting,
                "WANDERING" => State::Wandering,
                _ => State::Sleeping,
            };
            let mut mob = Mob::new(m[0].as_u64()? as u32, key, stats, 0, state);
            mob.target = m[3].as_i64()? as i32;
            mob.enemy_seen = m[4].as_bool()?;
            mob.alerted = m[5].as_bool()?;
            load_char(&d, &mut mob.ch, &m[6])?;
            mobs.push(mob);
        }
        let mut scheduler = Scheduler { now: f["now"].as_f64()?, ..Default::default() };
        for a in f["actors"].as_array()? {
            scheduler.actors.push((unactor(&a[0])?, a[1].as_f64()?));
        }
        if let Some(o) = f["order"].as_array() {
            scheduler.order = Some(o.iter().map(unactor).collect::<Option<_>>()?);
        }
        scheduler.turn = f["turn"].as_u64()? as usize;
        scheduler.round = f["round"].as_u64()? as u32;
        scheduler.calm = f["calm"].as_u64()? as u32;
        d.floors.insert(depth, Floor { level, mobs, scheduler });
    }
    // la vista de cada piso con sus héroes; queda en contexto el del héroe local
    let depths: Vec<i32> = d.floors.keys().copied().collect();
    for depth in depths {
        d.depth = depth;
        d.observe();
    }
    d.depth = d.heroes.get(d.hero)?.depth;
    d.view_changed = true;
    Some(d)
}
