//! NPCs (`roadmap/npcs.md`, paso N1): mobs con diálogo (`"npc"` en data/mobs.json) que no cazan (`State::Passive`).
//! Hablar abre su diálogo (data/dialogs.json): nodos con texto y opciones; cada opción con condiciones (`if`),
//! efectos (`do`) y el nodo siguiente (`go`; sin él, la charla termina). Los flags son de la partida (en la LAN, del
//! grupo). Una condición o un efecto desconocido (de un mod más nuevo) no se cumple / no hace nada.
use serde_json::Value;

use crate::actor::ActorId;
use crate::dungeon::Dungeon;
use crate::geom::LENGTH;
use crate::hero::Action;
use crate::items::{HeapKind, Item};
use crate::level::level_seed;
use crate::mobs::State;
use crate::registry::text;
use crate::rng::Rng;

/// Lo que la UI muestra del diálogo abierto: el NPC, su texto y las opciones disponibles (índice en el nodo, texto).
pub struct DialogView {
    pub npc: ActorId,
    pub name: String,
    pub text: String,
    pub options: Vec<(usize, String)>,
}

impl Dungeon {
    /// Los NPCs de un piso nuevo (`data/npcs.json`: piso -> [[mob, probabilidad]]). Con un rng propio: no gasta el del
    /// nivel, así los pisos, sus mobs y sus ítems salen iguales que sin NPCs.
    pub(crate) fn create_npcs(&mut self) {
        let Some(table) = self.npcs.get(&self.depth.to_string()).and_then(Value::as_array).cloned() else { return };
        let mut rng = Rng::new(level_seed(self.seed, self.depth) ^ 0x4E50_4353);
        for e in table {
            let (Some(key), Some(p)) = (e[0].as_str(), e[1].as_f64()) else { continue };
            if !self.mobs_data.contains_key(key) || rng.float() >= p {
                continue;
            }
            for _ in 0..50 {
                let l = self.level();
                let c = l.random_destination(&mut rng);
                if c != l.entrance && c != l.exit && self.find_char(c).is_none() {
                    self.add_mob(key, c, State::Passive);
                    break;
                }
            }
        }
    }

    /// ¿Es un NPC tranquilo (se le habla, no pelea)?
    pub fn passive(&self, a: ActorId) -> bool {
        matches!(a, ActorId::Mob(id) if self.mob(id).is_some_and(|m| m.state == State::Passive))
    }

    fn dialog_of(&self, id: u32) -> Option<&Value> {
        let key = self.mob(id).filter(|m| m.state == State::Passive)?.stats.get("npc")?.as_str()?;
        self.dialogs.get(key)
    }

    /// El nodo abierto del héroe i (None si el NPC ya no está tranquilo o el nodo no existe).
    fn node(&self, i: usize) -> Option<(u32, &Value)> {
        let (id, node) = self.heroes[i].talking.as_ref()?;
        let n = self.dialog_of(*id)?["nodes"].get(node)?;
        Some((*id, n))
    }

    pub(crate) fn talk(&mut self, i: usize, id: u32) {
        if let Some(start) = self.dialog_of(id).and_then(|d| d["start"].as_str()).map(str::to_string) {
            self.heroes[i].talking = Some((id, start));
        }
    }

    pub fn dialog(&self, i: usize) -> Option<DialogView> {
        let (id, n) = self.node(i)?;
        let me = &self.heroes[i].ch.name;
        let fill = |v: &Value| text(v).replace("{name}", me);
        let options = n["options"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, o)| self.all(i, &o["if"]))
            .map(|(k, o)| (k, fill(&o["text"])))
            .collect();
        Some(DialogView { npc: ActorId::Mob(id), name: self.mob(id)?.ch.name.clone(), text: fill(&n["text"]), options })
    }

    /// Elige la opción k (si sus condiciones se cumplen): pasa al nodo siguiente (o termina) y aplica sus efectos.
    pub(crate) fn choose(&mut self, i: usize, k: usize) {
        let Some((id, n)) = self.node(i) else {
            self.heroes[i].talking = None;
            return;
        };
        let Some(o) = n["options"].get(k).filter(|o| self.all(i, &o["if"])).cloned() else { return };
        self.heroes[i].talking = o["go"].as_str().map(|g| (id, g.to_string()));
        for e in o["do"].as_array().into_iter().flatten() {
            self.apply(i, id, e);
        }
    }

    fn all(&self, i: usize, conds: &Value) -> bool {
        conds.as_array().is_none_or(|c| c.iter().all(|c| self.cond(i, c)))
    }

    /// Condiciones: ["has", ítem, n], ["gold", n], ["flag", clave, "=" | "!=" | ">=" | "<", valor], ["class", id],
    /// ["level", n], ["not", condición].
    fn cond(&self, i: usize, c: &Value) -> bool {
        let h = &self.heroes[i];
        let n = |k: usize| c[k].as_i64().unwrap_or(0);
        match c[0].as_str().unwrap_or("") {
            "has" => {
                let id = c[1].as_str().unwrap_or("");
                let worn = [h.weapon.as_ref(), h.armor.as_ref()].into_iter().flatten();
                let have: i64 = h.backpack.iter().chain(worn).filter(|b| b.id == id).map(|b| i64::from(b.quantity)).sum();
                have >= n(2).max(1)
            }
            "gold" => i64::from(h.gold) >= n(1),
            "flag" => {
                let v = self.flags.get(c[1].as_str().unwrap_or("")).copied().unwrap_or(0);
                let x = n(3);
                match c[2].as_str() {
                    Some("=") => v == x,
                    Some("!=") => v != x,
                    Some(">=") => v >= x,
                    Some("<") => v < x,
                    _ => false,
                }
            }
            "class" => c[1] == h.class_key.as_str(),
            "level" => i64::from(h.lvl) >= n(1),
            "not" => !self.cond(i, &c[1]),
            _ => false,
        }
    }

    /// Efectos: ["give", ítem, n], ["take", ítem, n], ["gold", ±n], ["flag", clave, valor], ["add", clave, ±n],
    /// ["log", texto] (sólo para el que habla), ["map"] (el mapa del piso), ["attack"] (lo vuelve hostil), ["end"].
    fn apply(&mut self, i: usize, id: u32, e: &Value) {
        let n = |k: usize| e[k].as_i64().unwrap_or(0);
        let key = e[1].as_str().unwrap_or("").to_string();
        match e[0].as_str().unwrap_or("") {
            "give" => {
                let Some(mut item) = Item::new(&self.items, &key) else { return };
                let count = n(2).clamp(1, 999) as i32;
                let reps = if item.stackable() {
                    item.quantity = count;
                    1
                } else {
                    count
                };
                let name = self.item_name(&item);
                for _ in 0..reps {
                    if let Err(it) = self.collect(i, item.clone()) {
                        let pos = self.heroes[i].ch.pos;
                        self.floor_mut().level.drop(it, pos, HeapKind::Heap); // mochila llena: al suelo
                    }
                }
                self.tell(i, &format!("Recibes {name}"));
            }
            "take" => {
                let mut left = n(2).clamp(1, 999) as i32;
                let h = &mut self.heroes[i];
                for b in h.backpack.iter_mut().filter(|b| b.id == key) {
                    let t = left.min(b.quantity);
                    b.quantity -= t;
                    left -= t;
                }
                h.backpack.retain(|b| b.quantity > 0);
            }
            "gold" => {
                let h = &mut self.heroes[i];
                h.gold = (i64::from(h.gold) + n(1)).clamp(0, i64::from(i32::MAX)) as i32;
            }
            "flag" => _ = self.flags.insert(key, n(2)),
            "add" => *self.flags.entry(key).or_insert(0) += n(2),
            "log" => {
                let msg = text(&e[1]).replace("{name}", &self.heroes[i].ch.name);
                self.tell(i, &msg);
            }
            "map" => {
                let l = &mut self.floor_mut().level;
                for c in 0..LENGTH as usize {
                    if l.discoverable[c] {
                        l.mapped[c] = true;
                    }
                }
                self.observe();
                self.view_changed = true;
                self.tell(i, "Ahora conoces el mapa de este piso");
            }
            "attack" => {
                // traición: lo que quede de charla se corta; el ataque inicia el combate (y lo sorprende)
                self.heroes[i].talking = None;
                if let Some(m) = self.mob_mut(id) {
                    m.state = State::Wandering;
                    m.alerted = true;
                }
                self.heroes[i].action = Some(Action::Attack(ActorId::Mob(id)));
            }
            "end" => self.heroes[i].talking = None,
            _ => {}
        }
    }
}
