//! Lo mínimo de `game/items.py` para R3: el arma y la armadura iniciales, sus dados de daño y la CA.
//! ponytail: sin nivel, maldición, apilado ni inventario; llegan con los ítems (R5)
use serde_json::{Map, Value};

pub struct Item {
    pub id: String,
    pub data: Value,
}

impl Item {
    pub fn new(items: &Map<String, Value>, id: &str) -> Option<Self> {
        Some(Self { id: id.to_string(), data: items.get(id)?.clone() })
    }

    pub fn int(&self, key: &str) -> i32 {
        self.data.get(key).and_then(Value::as_i64).unwrap_or(0) as i32
    }

    pub fn str(&self, key: &str) -> &str {
        self.data.get(key).and_then(Value::as_str).unwrap_or("")
    }

    pub fn flag(&self, key: &str) -> bool {
        self.data.get(key).and_then(Value::as_bool).unwrap_or(false)
    }
}

/// Dados de daño del arma ("1d6"). ponytail: el nivel del arma se suma cuando existan las mejoras (R5)
pub fn weapon_dice(w: &Item) -> &str {
    w.str("dice")
}

/// CA con esta armadura (None = sin armadura: 10 + DES). `dex_max` null = sin tope.
pub fn armor_ac(a: Option<&Item>, dex_mod: i32) -> i32 {
    let Some(a) = a else { return 10 + dex_mod };
    let dex = match a.data.get("dex_max").and_then(Value::as_i64) {
        Some(max) => dex_mod.min(max as i32),
        None => dex_mod,
    };
    a.int("ac") + dex
}
