//! Registros de contenido (`game/registry.py`): `data/<nombre>.json` (clases, mobs, bestiario, ítems, estados,
//! conjuros, ...). Primero el juego base y después los mods (`load_mods`): lo último pisa a lo anterior. Una entrada con
//! `"base": "<id>"` copia esa otra entrada y aplica sus campos encima.
//! El orden de las entradas se conserva (serde_json con `preserve_order`): decide tiradas, como el dict de Python.
use std::path::Path;
use std::sync::{LazyLock, OnceLock, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Map, Value};

/// El contenido del juego base va dentro del binario (Android sin AssetManager).
const BASE: [(&str, &str); 11] = [
    ("classes", include_str!("../../data/classes.json")),
    ("mobs", include_str!("../../data/mobs.json")),
    ("bestiary", include_str!("../../data/bestiary.json")),
    ("items", include_str!("../../data/items.json")),
    ("buffs", include_str!("../../data/buffs.json")),
    ("spells", include_str!("../../data/spells.json")),
    ("npcs", include_str!("../../data/npcs.json")),
    ("dialogs", include_str!("../../data/dialogs.json")),
    ("skills", include_str!("../../data/skills.json")),
    ("skins", include_str!("../../data/skins.json")),
    ("quests", include_str!("../../data/quests.json")),
];

/// (carpeta, `mod.json`, [(registro, json)]) de un mod incluido.
type Builtin = (&'static str, &'static str, &'static [(&'static str, &'static str)]);

/// Mods que vienen con el juego (`data/mods/<mod>/`): (carpeta, `mod.json`, registros). Van antes de los del usuario
/// (que pueden pisarlos) y se desactivan como cualquier otro.
static BUILTIN: [Builtin; 1] = [(
    "mercader-ambulante",
    include_str!("../../data/mods/mercader-ambulante/mod.json"),
    &[
        ("mobs", include_str!("../../data/mods/mercader-ambulante/mobs.json")),
        ("dialogs", include_str!("../../data/mods/mercader-ambulante/dialogs.json")),
        ("npcs", include_str!("../../data/mods/mercader-ambulante/npcs.json")),
    ],
)];

/// Tope del `icon.png` de un mod.
const ICON_MAX: usize = 64 * 1024;

/// Un mod de la lista (incluido o de `<datos>/mods`): su carpeta, si se cargó, su `mod.json` (`name`, `desc`,
/// `author`, `version`, `settings`; opcional) y su `icon.png`.
pub struct ModInfo {
    pub folder: String,
    pub builtin: bool,
    pub enabled: bool,
    pub manifest: Value,
    pub icon: Option<Vec<u8>>,
}

static INFOS: OnceLock<Vec<ModInfo>> = OnceLock::new();

/// Todos los mods (también los desactivados), en orden de carga.
pub fn mods() -> &'static [ModInfo] {
    INFOS.get().map_or(&[], Vec::as_slice)
}

/// Opciones de los mods (`settings` de su `mod.json`): mod -> {id: valor}. Las pone la app (de Local.config o el
/// `default`) y las leen los scripts (`ctx.settings`) y `npcs.json` (`"scale"`). Cambian en caliente.
static SETTINGS: LazyLock<RwLock<Map<String, Value>>> = LazyLock::new(Default::default);

pub fn set_setting(m: &str, id: &str, v: Value) {
    if let Ok(mut s) = SETTINGS.write()
        && let Some(o) = s.entry(m).or_insert_with(|| Value::Object(Map::new())).as_object_mut()
    {
        o.insert(id.to_string(), v);
    }
}

pub fn settings() -> Value {
    SETTINGS.read().map(|s| Value::Object(s.clone())).unwrap_or_default()
}

pub fn setting(m: &str, id: &str) -> Option<Value> {
    SETTINGS.read().ok()?.get(m)?.get(id).cloned()
}

/// Scripts de los mods (`<mod>/script.lua`): (mod, código), en orden de carga (`script.rs`).
static SCRIPTS: OnceLock<Vec<(String, String)>> = OnceLock::new();

pub fn scripts() -> &'static [(String, String)] {
    SCRIPTS.get().map_or(&[], Vec::as_slice)
}

/// Registros de los mods activos (los incluidos y los de `<datos>/mods`), en orden de carga.
static MODS: OnceLock<Vec<(String, String)>> = OnceLock::new();

/// Lee los mods, una vez y antes de `load`: los incluidos y después `dir/mods` (carpetas en orden alfabético), salvo
/// los de `disabled` (que igual quedan en `mods()` para la lista). Por mod: los registros conocidos (un JSON que no es
/// un objeto se salta), `script.lua`, `mod.json` e `icon.png`. Devuelve una línea por mod (para el log).
pub fn load_mods(dir: &Path, disabled: &[String]) -> Vec<String> {
    let (mut out, mut msgs, mut scripts, mut infos) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (folder, manifest, regs) in &BUILTIN {
        let enabled = !disabled.iter().any(|d| d == folder);
        if enabled {
            out.extend(regs.iter().map(|(r, t)| (r.to_string(), t.to_string())));
        }
        infos.push(ModInfo { folder: folder.to_string(), builtin: true, enabled, manifest: serde_json::from_str(manifest).unwrap_or_default(), icon: None });
    }
    let mut dirs: Vec<_> = std::fs::read_dir(dir.join("mods")).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    for m in dirs {
        let name = m.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let enabled = !disabled.contains(&name);
        let manifest = std::fs::read_to_string(m.join("mod.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        let icon = std::fs::read(m.join("icon.png")).ok().filter(|b| b.len() <= ICON_MAX);
        infos.push(ModInfo { folder: name.clone(), builtin: false, enabled, manifest, icon });
        if !enabled {
            msgs.push(format!("mod {name}: desactivado"));
            continue;
        }
        let mut n = 0;
        for (reg, _) in BASE {
            let Ok(text) = std::fs::read_to_string(m.join(format!("{reg}.json"))) else { continue };
            match serde_json::from_str::<Map<String, Value>>(&text) {
                Ok(_) => {
                    out.push((reg.to_string(), text));
                    n += 1;
                }
                Err(e) => msgs.push(format!("mod {name}/{reg}.json: {e}")),
            }
        }
        if let Ok(src) = std::fs::read_to_string(m.join("script.lua")) {
            scripts.push((name.clone(), src));
            n += 1;
        }
        msgs.push(format!("mod {name}: {n} archivos"));
    }
    let _ = MODS.set(out);
    let _ = SCRIPTS.set(scripts);
    let _ = INFOS.set(infos);
    msgs
}

/// Un archivo que puede traer un mod: `<registro>.json`, `script.lua`, `mod.json` o `icon.png`.
pub fn is_mod_file(name: &str) -> bool {
    matches!(name, "script.lua" | "mod.json" | "icon.png") || name.strip_suffix(".json").is_some_and(|r| BASE.iter().any(|(n, _)| *n == r))
}

/// El juego base y los mods activos: lo último pisa a lo anterior. Sin `load_mods` (los tests), base + incluidos.
fn sources() -> Box<dyn Iterator<Item = (&'static str, &'static str)>> {
    match MODS.get() {
        Some(m) => Box::new(BASE.into_iter().chain(m.iter().map(|(n, t)| (n.as_str(), t.as_str())))),
        None => Box::new(BASE.into_iter().chain(BUILTIN.iter().flat_map(|(_, _, r)| r.iter().copied()))),
    }
}

/// Hash del contenido (los JSON de los registros, en orden): un cliente con otro contenido no entra a la partida.
/// FNV-1a de 64 bits: estable entre plataformas y versiones de Rust (no es criptográfico: sólo detecta diferencias).
pub fn content_hash() -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (n, text) in sources().chain(scripts().iter().map(|(_, s)| ("script", s.as_str()))) {
        for b in n.bytes().chain(text.bytes()) {
            h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

pub fn load(name: &str) -> Result<Map<String, Value>, String> {
    let mut merged = Map::new();
    for (n, text) in sources() {
        if n == name {
            let m: Map<String, Value> = serde_json::from_str(text).map_err(|e| format!("data/{n}.json: {e}"))?;
            merged.extend(m); // una clave repetida pisa el valor y conserva su lugar (como dict.update)
        }
    }
    merged.keys().map(|k| Ok((k.clone(), resolve(&merged, k, &mut Vec::new())?))).collect()
}

fn resolve(entries: &Map<String, Value>, key: &str, seen: &mut Vec<String>) -> Result<Value, String> {
    let entry = &entries[key];
    let Some(base) = entry.get("base").and_then(Value::as_str) else { return Ok(entry.clone()) };
    if seen.iter().any(|s| s == base) || !entries.contains_key(base) {
        return Err(format!("{key}: base inválida {base:?}"));
    }
    seen.push(key.to_string());
    let mut out = resolve(entries, base, seen)?;
    if let (Some(o), Some(e)) = (out.as_object_mut(), entry.as_object()) {
        for (k, v) in e {
            if k != "base" {
                o.insert(k.clone(), v.clone());
            }
        }
    }
    Ok(out)
}

/// Idiomas del contenido (`game/lang.py`): (código, nombre). Lo elige la configuración (`language` en Local.config).
pub const LANGS: [(&str, &str); 2] = [("es", "Español"), ("en", "English")];
static LANG: AtomicUsize = AtomicUsize::new(0);

/// Cambia el idioma del contenido (un código desconocido se ignora). Sólo afecta textos: nunca una tirada.
pub fn set_lang(code: &str) {
    if let Some(i) = LANGS.iter().position(|l| l.0 == code) {
        LANG.store(i, Ordering::Relaxed);
    }
}

pub fn lang() -> &'static str {
    LANGS[LANG.load(Ordering::Relaxed)].0
}

/// Texto del código (UI y mensajes) en el idioma actual: `tr!("Atrás", "Back")` da un `&'static str`.
#[macro_export]
macro_rules! tr {
    ($es:literal, $en:literal) => {
        if $crate::registry::lang() == "en" { $en } else { $es }
    };
}

/// Como `tr!` pero con formato (`format!`): `trf!("Recoges {n}", "You pick up {n}")` o con argumentos al final.
#[macro_export]
macro_rules! trf {
    ($es:literal, $en:literal $(, $a:expr)* $(,)?) => {
        if $crate::registry::lang() == "en" { format!($en $(, $a)*) } else { format!($es $(, $a)*) }
    };
}

/// Texto de contenido: `str` o `{"es": ..., "en": ...}`. Si falta el idioma actual, inglés; si no, el primero que haya
/// (un mod puede traer sólo los suyos).
pub fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(o) => {
            o.get(lang()).or_else(|| o.get("en")).or_else(|| o.values().next()).and_then(Value::as_str).unwrap_or("?").to_string()
        }
        _ => "?".to_string(),
    }
}
