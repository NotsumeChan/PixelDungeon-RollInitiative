//! Ítems: items/*.java + Generator.java + Heap.java + ItemStatusHandler (`game/items.py`). El contenido es
//! `data/items.json` (IDs "namespace:nombre"); los efectos de pociones y pergaminos se nombran ahí y viven en hero.rs.
use serde_json::{Map, Value};

use crate::dungeon::Dungeon;
use crate::hero::Hero;
use crate::registry::text;
use crate::rng::Rng;

pub const GOLD: &str = "pdri:gold";
/// Generator.Category: peso de cada categoría (se divide a la mitad cada vez que sale, se reinicia por piso).
const CATEGORY_WEIGHTS: [(&str, f64); 7] =
    [("weapon", 10.0), ("missile", 5.0), ("armor", 10.0), ("potion", 50.0), ("scroll", 40.0), ("food", 0.0), ("gold", 50.0)];
/// Potion.colors: nombre [es, en] e índice en items.png (56 + i).
pub const POTION_LOOKS: [[&str; 2]; 12] = [
    ["turquesa", "turquoise"],
    ["carmesí", "crimson"],
    ["celeste", "azure"],
    ["jade", "jade"],
    ["dorada", "golden"],
    ["magenta", "magenta"],
    ["negra", "charcoal"],
    ["marfil", "ivory"],
    ["ámbar", "amber"],
    ["parda", "bistre"],
    ["índigo", "indigo"],
    ["plateada", "silver"],
];
/// Scroll.runes, con su índice en items.png.
pub const SCROLL_LOOKS: [(&str, i32); 12] = [
    ("KAUNAN", 40),
    ("SOWILO", 41),
    ("LAGUZ", 42),
    ("YNGVI", 43),
    ("GYFU", 44),
    ("RAIDO", 45),
    ("ISAZ", 46),
    ("MANNAZ", 47),
    ("NAUDIZ", 76),
    ("BERKANAN", 77),
    ("ODAL", 78),
    ("TIWAZ", 79),
];
/// Íconos de ranura vacía (ItemSpriteSheet.WEAPON / ARMOR / RING).
pub const SLOT_WEAPON: i32 = 5;
pub const SLOT_ARMOR: i32 = 6;
pub const SLOT_RING: i32 = 7;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: String,
    pub data: Value,
    pub quantity: i32,
    pub level: i32,
    pub cursed: bool,
    pub level_known: bool,
    pub cursed_known: bool,
}

impl Item {
    pub fn new(items: &Map<String, Value>, id: &str) -> Option<Self> {
        let data = items.get(id)?.clone();
        Some(Self { id: id.to_string(), data, quantity: 1, level: 0, cursed: false, level_known: false, cursed_known: false })
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

    pub fn category(&self) -> &str {
        self.str("category")
    }

    pub fn stackable(&self) -> bool {
        self.flag("stackable")
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self.category(), "weapon" | "armor")
    }

    /// Saca n de una pila (el original queda con el resto).
    pub fn split(&mut self, n: i32) -> Item {
        let mut part = self.clone();
        part.quantity = n;
        self.quantity -= n;
        part
    }
}

/// Dados de daño con el nivel del arma sumado: +1 sword -> "1d8+1".
pub fn weapon_dice(w: &Item) -> String {
    let base = w.str("dice");
    if w.level != 0 { format!("{base}{:+}", w.level) } else { base.to_string() }
}

/// CA con esta armadura (None = sin armadura: 10 + DES). `dex_max` null = sin tope. El nivel suma a la CA.
pub fn armor_ac(a: Option<&Item>, dex_mod: i32) -> i32 {
    let Some(a) = a else { return 10 + dex_mod };
    let dex = match a.data.get("dex_max").and_then(Value::as_i64) {
        Some(max) => dex_mod.min(max as i32),
        None => dex_mod,
    };
    a.int("ac") + dex + a.level
}

/// Weapon.random / Armor.random: 40% de tener nivel; la mitad de esos son negativos y malditos.
fn roll_equipment(mut item: Item, rng: &mut Rng) -> Item {
    if rng.float() < 0.4 {
        let mut n = 1;
        if rng.int(3) == 0 {
            n += 1;
            if rng.int(3) == 0 {
                n += 1;
            }
        }
        if rng.int(2) == 0 {
            item.level += n;
        } else {
            item.level -= n;
            item.cursed = true;
        }
    }
    item
}

pub fn make_gold(items: &Map<String, Value>, rng: &mut Rng, depth: i32) -> Item {
    let mut g = Item::new(items, GOLD).expect("data/items.json sin oro");
    g.quantity = rng.int_range(20 + depth * 10, 40 + depth * 20);
    g
}

/// Generator.java. Estado por partida (los pesos de categoría se gastan al salir; se reinician por piso).
#[derive(Clone)]
pub struct Generator {
    weights: Vec<(&'static str, f64)>,
}

impl Default for Generator {
    fn default() -> Self {
        Self { weights: CATEGORY_WEIGHTS.to_vec() }
    }
}

impl Generator {
    pub fn reset(&mut self) {
        self.weights = CATEGORY_WEIGHTS.to_vec();
    }

    /// Los pesos actuales, en el orden de las categorías (guardado de partidas).
    pub fn weights(&self) -> Vec<f64> {
        self.weights.iter().map(|c| c.1).collect()
    }

    pub fn set_weights(&mut self, w: &[f64]) {
        for (c, &v) in self.weights.iter_mut().zip(w) {
            c.1 = v;
        }
    }

    /// cur_str = FUE del héroe local (equipo: de dos, el de FUE más cercana).
    pub fn random(&mut self, items: &Map<String, Value>, rng: &mut Rng, depth: i32, cur_str: i32) -> Item {
        let w: Vec<f64> = self.weights.iter().map(|c| c.1).collect();
        let cat = self.weights[rng.chances(&w)].0;
        self.random_of(cat, items, rng, depth, cur_str)
    }

    pub fn random_of(&mut self, cat: &str, items: &Map<String, Value>, rng: &mut Rng, depth: i32, cur_str: i32) -> Item {
        if let Some(c) = self.weights.iter_mut().find(|c| c.0 == cat) {
            c.1 /= 2.0;
        }
        if cat == "gold" {
            return make_gold(items, rng, depth);
        }
        if cat == "weapon" || cat == "armor" {
            let a = roll_equipment(Item::new(items, &pick(items, cat, rng)).unwrap(), rng);
            let b = roll_equipment(Item::new(items, &pick(items, cat, rng)).unwrap(), rng);
            return if (cur_str - a.int("STR")).abs() < (cur_str - b.int("STR")).abs() { a } else { b };
        }
        let mut item = Item::new(items, &pick(items, cat, rng)).unwrap();
        if let Some(q) = item.data.get("qty").and_then(Value::as_array) {
            // arrojadizas y flechas salen en montón
            item.quantity = rng.int_range(q[0].as_i64().unwrap_or(1) as i32, q[1].as_i64().unwrap_or(1) as i32);
        }
        item
    }
}

fn pick(items: &Map<String, Value>, cat: &str, rng: &mut Rng) -> String {
    let ids: Vec<&String> = items.iter().filter(|(_, d)| d["category"] == cat).map(|(k, _)| k).collect();
    let w: Vec<f64> = ids.iter().map(|i| items[*i].get("gen_weight").and_then(Value::as_f64).unwrap_or(0.0)).collect();
    ids[rng.chances(&w)].clone()
}

// --- montones en el suelo (Heap.java) ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeapKind {
    Heap,
    Chest,
    Skeleton,
    Tomb,
}

#[derive(Clone, Debug)]
pub struct Heap {
    pub kind: HeapKind,
    /// items[0] = el de arriba.
    pub items: Vec<Item>,
}

impl Heap {
    pub fn add(&mut self, item: Item) {
        if item.stackable()
            && let Some(i) = self.items.iter().position(|i| i.id == item.id)
        {
            let mut same = self.items.remove(i);
            same.quantity += item.quantity;
            self.items.insert(0, same);
            return;
        }
        self.items.insert(0, item);
    }
}

/// Qué se dibuja de un ítem: un índice de items.png o, sin sprite, su nombre.
#[derive(Clone, Debug, PartialEq)]
pub enum Image {
    Sheet(i32),
    Text(String),
}

// --- nombres y textos según lo que el jugador sabe ---
impl Dungeon {
    pub fn is_known(&self, item: &Item) -> bool {
        match item.category() {
            "potion" | "scroll" => self.known.contains(&item.id),
            _ if item.is_equipment() => item.level_known && item.cursed_known,
            _ => true,
        }
    }

    pub fn item_name(&self, item: &Item) -> String {
        let cat = item.category();
        if (cat == "potion" || cat == "scroll") && !self.known.contains(&item.id) {
            let look = self.looks.get(&item.id).map_or("?", |l| l.0);
            return if cat == "potion" { format!("poción {look}") } else { format!("pergamino \"{look}\"") };
        }
        let name = text(&item.data["name"]);
        if (item.is_equipment() || item.flag("upgradable")) && item.level_known && item.level != 0 {
            format!("{:+} {name}", item.level)
        } else {
            name
        }
    }

    pub fn item_image(&self, item: &Item) -> Image {
        if let Some(l) = self.looks.get(&item.id) {
            return Image::Sheet(l.1);
        }
        match item.data.get("image").and_then(Value::as_i64) {
            Some(i) => Image::Sheet(i as i32),
            None => Image::Text(self.item_name(item)),
        }
    }

    /// Lo que se dibuja de un montón: cofre, tumba o restos, o el ítem de arriba.
    pub fn heap_image(&self, heap: &Heap) -> Image {
        match heap.kind {
            HeapKind::Chest => Image::Sheet(11),
            HeapKind::Tomb => Image::Sheet(13),
            HeapKind::Skeleton => Image::Sheet(0),
            HeapKind::Heap => heap.items.first().map_or(Image::Sheet(0), |i| self.item_image(i)),
        }
    }

    /// Texto de la ventana del ítem (WndItem / item_info), con los números de 5e.
    pub fn item_info(&self, item: &Item, hero: &Hero) -> String {
        let kind = |k: &str| match k {
            "simple" => "simple",
            "martial" => "marcial",
            "clothing" => "ropa",
            "light" => "ligera",
            "medium" => "media",
            "heavy" => "pesada",
            _ => "?",
        };
        let d = &item.data;
        let str_ = hero.score("STR");
        let need = item.int("STR");
        let mut t = match item.category() {
            "weapon" => {
                let dice = if item.level_known { weapon_dice(item) } else { item.str("dice").to_string() };
                let mut t = if item.flag("ranged") {
                    let ammo_id = item.str("ammo");
                    let ammo: i32 = hero.backpack.iter().filter(|i| i.id == ammo_id).map(|i| i.quantity).sum();
                    let r = &d["range"];
                    let ammo_name = self.items.get(ammo_id).map_or(String::new(), |a| capitalize(&text(&a["name"])));
                    format!(
                        "Arma a distancia {} (DES). Daño {dice}. Alcance {}/{} casillas (más lejos, desventaja). {ammo_name}s: {ammo}. Cuerpo a cuerpo golpea como un garrote (1d4)",
                        kind(item.str("kind")),
                        r[0],
                        r[1]
                    )
                } else {
                    let fin = if item.flag("finesse") { ", sutil (FUE o DES)" } else { "" };
                    format!("Arma {}{fin}. Daño {dice}", kind(item.str("kind")))
                };
                if item.level_known && item.level != 0 {
                    t += &format!(", {:+} al ataque", item.level);
                }
                t += &speed_text(item);
                t += &format!(". Requiere FUE {need}");
                if need > str_ {
                    t += &format!(" (te falta fuerza: desventaja y -{} al ataque)", need - str_);
                }
                if !hero.proficient_weapon_item(Some(item)) {
                    t += ". No eres competente: sin bonificador de competencia";
                }
                t
            }
            "missile" if item.flag("thrown") => {
                let r = &d["range"];
                let fin = if item.flag("finesse") { ", sutil (FUE o DES)" } else { " (FUE)" };
                let mut t = format!(
                    "Arma arrojadiza {}{fin}. Daño {}. Alcance {}/{} casillas (más lejos, desventaja)",
                    kind(item.str("kind")),
                    weapon_dice(item),
                    r[0],
                    r[1]
                );
                t += &speed_text(item);
                t += if item.flag("returns") { ". Vuelve a tu mano" } else { ". Si acierta se pierde; si falla, cae al suelo" };
                if let Some(desc) = d.get("desc") {
                    t += &format!(". {}", text(desc).trim_end_matches('.'));
                }
                t += &format!(". Requiere FUE {need}");
                if need > str_ {
                    t += " (te falta fuerza: desventaja)";
                }
                if !hero.proficient_weapon_item(Some(item)) {
                    t += ". No eres competente: sin bonificador de competencia";
                }
                t + "."
            }
            "armor" => {
                let dex = match d.get("dex_max").and_then(Value::as_i64) {
                    None => "+ DES".to_string(),
                    Some(0) => String::new(),
                    Some(m) => format!("+ DES (max {m})"),
                };
                let bonus = if item.level_known && item.level != 0 { format!(" {:+}", item.level) } else { String::new() };
                let mut t = format!("Armadura {}. CA {}{bonus} {dex}", kind(item.str("kind")), item.int("ac"));
                if need != 0 {
                    t += &format!(". Requiere FUE {need}");
                }
                if need > str_ {
                    t += " (te falta fuerza: más lento y desventaja al atacar)";
                }
                if !hero.proficient_armor_item(Some(item)) {
                    t += ". No eres competente: atacas con desventaja";
                }
                t
            }
            "potion" | "scroll" if !self.known.contains(&item.id) => "No sabes qué hace. Hay una sola forma de averiguarlo.".into(),
            "gold" => format!("{} monedas de oro.", item.quantity),
            _ => d.get("desc").map_or(String::new(), text),
        };
        if item.is_equipment() {
            if !item.level_known {
                t += ". No conoces su nivel ni si está maldita";
            } else if item.cursed && item.cursed_known {
                t += ". Está maldita: no te la podrás quitar";
            }
            t += ".";
        }
        t
    }

    /// Una moneda de oro (la ranura de oro de la mochila).
    pub fn gold_item(&self) -> Item {
        self.new_item(GOLD)
    }

    /// Un ítem nuevo del registro (cantidad 1).
    pub(crate) fn new_item(&self, id: &str) -> Item {
        Item::new(&self.items, id).unwrap_or_else(|| panic!("ítem desconocido: {id}"))
    }
}

fn speed_text(item: &Item) -> String {
    let dly = item.data.get("dly").and_then(Value::as_f64).unwrap_or(1.0);
    if dly != 1.0 { format!(". Velocidad x{}", 1.0 / dly) } else { String::new() }
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}
