//! Lista de personajes creados (`game/characters.py`), en `<datos>/characters.json`. Personaje = {"id", "name",
//! "class", "abilities", "created"}. Las entradas se guardan tal cual (una de otra versión o de una clase que no está
//! cargada se conserva aunque no se pueda jugar).
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};

use crate::dice::ABILITIES;

const FILE_NAME: &str = "characters.json";

pub fn load(dir: &Path) -> Vec<Value> {
    let path = dir.join(FILE_NAME);
    let Ok(text) = std::fs::read_to_string(&path) else { return Vec::new() };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Array(v)) => v,
        _ => {
            // archivo roto: se aparta (no se pisa) para no perder los personajes
            let _ = std::fs::rename(&path, dir.join(format!("{FILE_NAME}.roto-{}", now())));
            Vec::new()
        }
    }
}

/// Escritura atómica: si el juego se corta a mitad, el archivo anterior queda intacto.
pub fn save(dir: &Path, chars: &[Value]) -> std::io::Result<()> {
    write_atomic(&dir.join(FILE_NAME), &serde_json::to_string_pretty(chars).unwrap_or_default())
}

pub(crate) fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(tmp, path)
}

pub(crate) fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Un personaje nuevo. El id sale del reloj (único en la práctica: nanosegundos).
pub fn new_character(name: &str, class_id: &str, abilities: [i32; 6]) -> Value {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let ab: Map<String, Value> = ABILITIES.iter().zip(abilities).map(|(a, v)| (a.to_string(), json!(v))).collect();
    json!({"id": format!("{nanos:032x}"), "name": name, "class": class_id, "abilities": ab, "created": now()})
}

/// Las características del personaje en el orden de `ABILITIES` (10 si falta alguna).
pub fn abilities(c: &Value) -> [i32; 6] {
    ABILITIES.map(|a| c["abilities"].get(a).and_then(Value::as_i64).unwrap_or(10) as i32)
}

pub fn id(c: &Value) -> &str {
    c["id"].as_str().unwrap_or("")
}
