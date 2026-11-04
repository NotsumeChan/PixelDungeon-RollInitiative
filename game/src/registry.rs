//! Registros de contenido (`game/registry.py`): `data/<nombre>.json` (clases, mobs, bestiario, ítems, estados,
//! conjuros). Las carpetas se leen en orden y lo último pisa a lo anterior (base de los mods). Una entrada con
//! `"base": "<id>"` copia esa otra entrada y aplica sus campos encima.
//! El orden de las entradas se conserva (serde_json con `preserve_order`): decide tiradas, como el dict de Python.
use serde_json::{Map, Value};

/// El contenido del juego base va dentro del binario (Android sin AssetManager).
/// ponytail: sólo el juego base; los mods agregarán sus carpetas aquí (1.x)
const BASE: [(&str, &str); 6] = [
    ("classes", include_str!("../../data/classes.json")),
    ("mobs", include_str!("../../data/mobs.json")),
    ("bestiary", include_str!("../../data/bestiary.json")),
    ("items", include_str!("../../data/items.json")),
    ("buffs", include_str!("../../data/buffs.json")),
    ("spells", include_str!("../../data/spells.json")),
];

pub fn load(name: &str) -> Result<Map<String, Value>, String> {
    let mut merged = Map::new();
    for (n, text) in BASE {
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

/// Texto de contenido: `str` o `{"es": ..., "en": ...}`.
/// ponytail: idioma fijo en español hasta la configuración (R5); si falta "es", el primero que haya
pub fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(o) => o.get("es").or_else(|| o.values().next()).and_then(Value::as_str).unwrap_or("?").to_string(),
        _ => "?".to_string(),
    }
}
