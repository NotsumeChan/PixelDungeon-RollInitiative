//! Registros de contenido (`game/registry.py`): `data/<nombre>.json` (clases, mobs, bestiario, ítems, estados,
//! conjuros, ...). Primero el juego base y después los mods (`load_mods`): lo último pisa a lo anterior. Una entrada con
//! `"base": "<id>"` copia esa otra entrada y aplica sus campos encima.
//! El orden de las entradas se conserva (serde_json con `preserve_order`): decide tiradas, como el dict de Python.
use std::path::Path;
use std::sync::OnceLock;
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

/// Registros de los mods (`<datos>/mods/<mod>/<registro>.json`), en orden de carpeta: se leen una vez, al arrancar.
static MODS: OnceLock<Vec<(String, String)>> = OnceLock::new();

/// Lee los mods de `dir/mods` (carpetas en orden alfabético; sólo los archivos de registros conocidos). Un JSON que no
/// es un objeto se salta. Devuelve una línea por mod leído o archivo salteado (para el log). Llamar antes de `load`.
pub fn load_mods(dir: &Path) -> Vec<String> {
    let mut mods: Vec<_> = std::fs::read_dir(dir.join("mods")).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    mods.sort();
    let (mut out, mut msgs) = (Vec::new(), Vec::new());
    for m in mods {
        let name = m.file_name().unwrap_or_default().to_string_lossy().into_owned();
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
        if n > 0 {
            msgs.push(format!("mod {name}: {n} registros"));
        }
    }
    let _ = MODS.set(out);
    msgs
}

/// El juego base y después los mods: lo último pisa a lo anterior.
fn sources() -> impl Iterator<Item = (&'static str, &'static str)> {
    BASE.into_iter().chain(MODS.get().into_iter().flatten().map(|(n, t)| (n.as_str(), t.as_str())))
}

/// Hash del contenido (los JSON de los registros, en orden): un cliente con otro contenido no entra a la partida.
/// FNV-1a de 64 bits: estable entre plataformas y versiones de Rust (no es criptográfico: sólo detecta diferencias).
pub fn content_hash() -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (n, text) in sources() {
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
