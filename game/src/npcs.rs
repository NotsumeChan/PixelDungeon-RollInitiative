//! NPCs (`roadmap/npcs.md`, pasos N1 a N4): mobs con diálogo (`"npc"` en data/mobs.json) que no cazan (`State::Passive`).
//! Hablar abre su diálogo (data/dialogs.json): nodos con texto y opciones; cada opción con condiciones (`if`),
//! efectos (`do`) y el nodo siguiente (`go`; sin él, la charla termina). Los flags son de la partida (en la LAN, del
//! grupo). Una condición o un efecto desconocido (de un mod más nuevo) no se cumple / no hace nada.
//! N2: una opción con `check` [pericia, CD] tira d20 + característica + competencia (data/skills.json, `skill_prof` de
//! la clase); bien = `go` y `do`, mal = `fail` y `fail_do`. La actitud del NPC (`Mob::disposition`) baja la CD 2 por
//! punto (o la sube si desconfía).
//! N3: misiones (data/quests.json: `name`, `desc`, `objectives` [["kill", mob, n] | ["has", ítem, n]]). Su estado es el
//! flag `quest:<id>` (0 sin empezar, 1 en curso, 2 hecha): de la partida, compartido en la LAN. Las muertes se cuentan
//! en `kills:<mob>` (desde que se acepta: `qbase:<misión>:<mob>`). Los diálogos las aceptan, las revisan ("ready") y
//! las entregan con sus efectos; la recompensa es lo que dé el diálogo.
//! N4: inventario y equipo (`inventory` [[ítem, n]], `stock` [[categoría, n]] del generador, `weapon`, `armor` del mob):
//! pelea con su equipo y lo suelta al morir. La tienda (efecto `shop`) vende de su inventario y lo que se le vende pasa
//! a él; precio = `price` del ítem, -10 % por punto de actitud (vender = la mitad). `give` saca de su inventario si lo
//! tiene y `take` lo guarda ahí.
use serde_json::Value;

use crate::actor::ActorId;
use crate::dice;
use crate::dungeon::Dungeon;
use crate::geom::LENGTH;
use crate::hero::Action;
use crate::items::{HeapKind, Item};
use crate::level::level_seed;
use crate::mobs::State;
use crate::registry::text;
use crate::rng::Rng;
use crate::{tr, trf};

/// Lo que la UI muestra del diálogo abierto: el NPC, su texto y las opciones disponibles (índice en el nodo, texto).
pub struct DialogView {
    pub npc: ActorId,
    pub name: String,
    /// Su actitud ("indiferente", "amistoso"...).
    pub attitude: &'static str,
    pub text: String,
    pub options: Vec<(usize, String)>,
}

/// La tienda abierta, para la UI: el NPC, lo que vende (ítem y precio) y cuánto paga por cada ítem de la mochila.
pub struct ShopView {
    pub npc: ActorId,
    pub name: String,
    pub stock: Vec<(Item, i32)>,
    pub sell: Vec<i32>,
}

/// Actitud por disposición, de -2 a 2.
fn attitude(disp: i32) -> &'static str {
    match disp.clamp(-2, 2) {
        -2 => tr!("hostil", "hostile"),
        -1 => tr!("desconfiado", "wary"),
        0 => tr!("indiferente", "indifferent"),
        1 => tr!("amistoso", "friendly"),
        _ => tr!("muy amistoso", "very friendly"),
    }
}

impl Dungeon {
    /// Los NPCs de un piso nuevo (`data/npcs.json`: piso -> [[mob, probabilidad]]; `"*"`, en cualquier piso). Con un rng propio: no gasta el del
    /// nivel, así los pisos, sus mobs y sus ítems salen iguales que sin NPCs.
    pub(crate) fn create_npcs(&mut self) {
        let table: Vec<Value> = [self.depth.to_string(), "*".to_string()]
            .iter()
            .filter_map(|k| self.npcs.get(k).and_then(Value::as_array))
            .flatten()
            .cloned()
            .collect();
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
                    let id = self.next_mob - 1;
                    self.equip_npc(id, &mut rng);
                    break;
                }
            }
        }
    }

    /// Inventario y equipo iniciales del NPC (`inventory`, `stock`, `weapon` y `armor` de su mob), con el rng del piso
    /// para NPCs (no el del nivel).
    fn equip_npc(&mut self, id: u32, rng: &mut Rng) {
        let Some(stats) = self.mob(id).map(|m| m.stats.clone()) else { return };
        let mut pack = Vec::new();
        for e in stats["inventory"].as_array().into_iter().flatten() {
            let Some(mut it) = e[0].as_str().and_then(|k| Item::new(&self.items, k)) else { continue };
            let n = e[1].as_i64().unwrap_or(1).clamp(1, 999) as i32;
            if it.stackable() {
                it.quantity = n;
                pack.push(it);
            } else {
                pack.extend(std::iter::repeat_n(it, n as usize));
            }
        }
        let (depth, cur_str) = (self.depth, self.local().score("STR"));
        for e in stats["stock"].as_array().into_iter().flatten() {
            let cat = e[0].as_str().unwrap_or("");
            for _ in 0..e[1].as_i64().unwrap_or(1).clamp(0, 20) {
                let mut it = self.generator.random_of(cat, &self.items, rng, depth, cur_str);
                it.cursed = false; // la tienda no vende cosas malditas
                pack.push(it);
            }
        }
        let eq = |k: &str| stats[k].as_str().and_then(|s| Item::new(&self.items, s));
        let (weapon, armor) = (eq("weapon"), eq("armor"));
        if let Some(m) = self.mob_mut(id) {
            (m.pack, m.weapon, m.armor) = (pack, weapon, armor);
        }
    }

    /// Guarda un ítem en la mochila del NPC (apila si se puede).
    fn npc_store(&mut self, id: u32, item: Item) {
        let Some(m) = self.mob_mut(id) else { return };
        let same = |b: &&mut Item| b.id == item.id && b.level == item.level && b.cursed == item.cursed;
        match m.pack.iter_mut().find(same).filter(|_| item.stackable()) {
            Some(b) => b.quantity += item.quantity,
            None => m.pack.push(item),
        }
    }

    /// Saca una unidad de ese ítem de la mochila del NPC (None si no tiene).
    fn npc_take_one(&mut self, id: u32, item_id: &str) -> Option<Item> {
        let m = self.mob_mut(id)?;
        let k = m.pack.iter().position(|b| b.id == item_id)?;
        Some(if m.pack[k].quantity > 1 { m.pack[k].split(1) } else { m.pack.remove(k) })
    }

    /// Precio de compra de una unidad: `price` del ítem, -10 % por punto de actitud (+10 % si desconfía).
    /// ponytail: sin contar el nivel del ítem ni si está identificado
    pub fn buy_price(&self, id: u32, item: &Item) -> i32 {
        let base = item.int("price").max(0);
        let disp = self.mob(id).map_or(0, |m| m.disposition);
        if base == 0 { 0 } else { (base * (10 - disp) / 10).max(1) }
    }

    /// Lo que paga por una unidad: la mitad del precio, +10 % por punto de actitud. 0 = no le interesa.
    pub fn sell_price(&self, id: u32, item: &Item) -> i32 {
        let base = item.int("price").max(0) / 2;
        let disp = self.mob(id).map_or(0, |m| m.disposition);
        if base == 0 { 0 } else { (base * (10 + disp) / 10).max(1) }
    }

    /// El NPC de la tienda abierta del héroe i (si sigue tranquilo).
    fn trader(&self, i: usize) -> Option<u32> {
        self.heroes[i].trading.filter(|&id| self.passive(ActorId::Mob(id)))
    }

    pub fn shop(&self, i: usize) -> Option<ShopView> {
        let id = self.trader(i)?;
        let m = self.mob(id)?;
        Some(ShopView {
            npc: ActorId::Mob(id),
            name: m.ch.name.clone(),
            stock: m.pack.iter().map(|it| (it.clone(), self.buy_price(id, it))).collect(),
            sell: self.heroes[i].backpack.iter().map(|it| self.sell_price(id, it)).collect(),
        })
    }

    /// Compra una unidad del ítem k del NPC.
    pub(crate) fn buy(&mut self, i: usize, k: usize) {
        let Some(id) = self.trader(i) else {
            self.heroes[i].trading = None;
            return;
        };
        let Some(item) = self.mob(id).and_then(|m| m.pack.get(k)).cloned() else { return };
        let price = self.buy_price(id, &item);
        if price == 0 {
            self.tell(i, tr!("Eso no está a la venta", "That's not for sale"));
            return;
        }
        if self.heroes[i].gold < price {
            self.tell(i, tr!("No te alcanza el oro", "You can't afford it"));
            return;
        }
        let Some(one) = self.npc_take_one(id, &item.id) else { return };
        let name = self.item_name(&one);
        match self.collect(i, one) {
            Ok(()) => {
                self.heroes[i].gold -= price;
                self.tell(i, &trf!("Compras {name} por {price} de oro", "You buy {name} for {price} gold"));
            }
            Err(back) => {
                self.npc_store(id, back);
                self.tell(i, tr!("No te queda lugar en la mochila", "Your backpack is full"));
            }
        }
    }

    /// Vende una unidad del ítem k de la mochila (lo equipado no se vende).
    pub(crate) fn sell(&mut self, i: usize, k: usize) {
        let Some(id) = self.trader(i) else {
            self.heroes[i].trading = None;
            return;
        };
        let Some(item) = self.heroes[i].backpack.get(k) else { return };
        let price = self.sell_price(id, item);
        if price == 0 {
            self.tell(i, tr!("No le interesa", "Not interested"));
            return;
        }
        let bp = &mut self.heroes[i].backpack;
        let one = if bp[k].quantity > 1 { bp[k].split(1) } else { bp.remove(k) };
        let name = self.item_name(&one);
        self.heroes[i].gold += price;
        self.npc_store(id, one);
        self.tell(i, &trf!("Vendes {name} por {price} de oro", "You sell {name} for {price} gold"));
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
        let label = |o: &Value| match self.check_of(id, o) {
            Some((sk, dc)) => trf!("{} [{} {:+}, CD {dc}]", "{} [{} {:+}, DC {dc}]", fill(&o["text"]), self.skill_name(&sk), self.skill_bonus(i, &sk)),
            None => fill(&o["text"]),
        };
        let options = n["options"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, o)| self.all(i, &o["if"]))
            .map(|(k, o)| (k, label(o)))
            .collect();
        let m = self.mob(id)?;
        let attitude = attitude(m.disposition);
        Some(DialogView { npc: ActorId::Mob(id), name: m.ch.name.clone(), attitude, text: fill(&n["text"]), options })
    }

    /// Elige la opción k (si sus condiciones se cumplen): pasa al nodo siguiente (o termina) y aplica sus efectos.
    pub(crate) fn choose(&mut self, i: usize, k: usize) {
        let Some((id, n)) = self.node(i) else {
            self.heroes[i].talking = None;
            return;
        };
        let Some(o) = n["options"].get(k).filter(|o| self.all(i, &o["if"])).cloned() else { return };
        // ponytail: el resultado se aplica en el acto (el dado lo muestra después); si molesta, esperar al dado
        let ok = self.check_of(id, &o).is_none_or(|(sk, dc)| self.skill_check(i, &sk, dc));
        let (go, effects) = if ok { ("go", "do") } else { ("fail", "fail_do") };
        self.heroes[i].talking = o[go].as_str().map(|g| (id, g.to_string()));
        for e in o[effects].as_array().into_iter().flatten() {
            self.apply(i, id, e);
        }
    }

    /// La tirada de una opción: (pericia, CD con la actitud del NPC: -2 por punto, entre 5 y 30).
    fn check_of(&self, id: u32, o: &Value) -> Option<(String, i32)> {
        let c = o.get("check")?;
        let skill = c[0].as_str()?.to_string();
        let disp = self.mob(id).map_or(0, |m| m.disposition);
        let dc = (c[1].as_i64()? as i32 - 2 * disp).clamp(5, 30);
        Some((skill, dc))
    }

    pub fn skill_name(&self, skill: &str) -> String {
        self.skills.get(skill).map_or_else(|| skill.to_string(), |s| text(&s["name"]))
    }

    /// Bono de pericia (5e): mod de su característica + competencia si la clase la tiene.
    pub fn skill_bonus(&self, i: usize, skill: &str) -> i32 {
        let h = &self.heroes[i];
        let ability = self.skills.get(skill).and_then(|s| s["ability"].as_str()).unwrap_or("CHA");
        h.ability_mod(ability) + if h.proficient_skill(skill) { h.proficiency() } else { 0 }
    }

    /// Las pericias con competencia del héroe i, con su bono (la ficha).
    pub fn proficient_skills(&self, i: usize) -> Vec<(String, i32)> {
        let h = &self.heroes[i];
        self.skills.keys().filter(|k| h.proficient_skill(k)).map(|k| (self.skill_name(k), self.skill_bonus(i, k))).collect()
    }

    /// Prueba de pericia con el dado grande; el resultado va al log de todos.
    fn skill_check(&mut self, i: usize, skill: &str, dc: i32) -> bool {
        let (nat, _) = dice::d20(&mut self.rng, 0);
        let b = self.skill_bonus(i, skill);
        let ok = nat + b >= dc;
        let name = self.skill_name(skill);
        self.roll_event(&trf!("{name} CD{dc}", "{name} DC{dc}"), nat + b, 1 + b, 20 + b, Some(ok));
        let who = self.heroes[i].ch.name.clone();
        let res = if ok { tr!("lo logra", "success") } else { tr!("falla", "failure") };
        self.log(&trf!("{who}: {name} {} contra CD {dc}, {res}", "{who}: {name} {} vs DC {dc}, {res}", nat + b));
        ok
    }

    /// Atacaron al NPC `id`: los otros NPCs tranquilos del piso que lo vieron desconfían más (disposición -1).
    pub(crate) fn npc_attacked(&mut self, id: u32) {
        let Some(pos) = self.mob(id).map(|m| m.ch.pos as usize) else { return };
        for m in &mut self.floor_mut().mobs {
            if m.id != id && m.state == State::Passive && m.fov.get(pos) == Some(&true) {
                m.disposition = (m.disposition - 1).max(-2);
            }
        }
    }

    fn all(&self, i: usize, conds: &Value) -> bool {
        conds.as_array().is_none_or(|c| c.iter().all(|c| self.cond(i, c)))
    }

    /// Condiciones: ["has", ítem, n], ["gold", n], ["flag", clave, "=" | "!=" | ">=" | "<", valor], ["class", id],
    /// ["level", n], ["disposition", n] (la actitud del NPC es al menos n), ["skill", pericia] (con competencia),
    /// ["quest", misión, "none" | "active" | "ready" (en curso y con los objetivos cumplidos) | "done"],
    /// ["not", condición].
    fn cond(&self, i: usize, c: &Value) -> bool {
        let h = &self.heroes[i];
        let n = |k: usize| c[k].as_i64().unwrap_or(0);
        match c[0].as_str().unwrap_or("") {
            "has" => self.count_item(i, c[1].as_str().unwrap_or("")) >= n(2).max(1),
            "quest" => {
                let q = c[1].as_str().unwrap_or("");
                match c[2].as_str() {
                    Some("none") => self.quest_state(q) == 0,
                    Some("active") => self.quest_state(q) == 1,
                    Some("ready") => self.quest_ready(i, q),
                    Some("done") => self.quest_state(q) == 2,
                    _ => false,
                }
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
            "disposition" => {
                let npc = h.talking.as_ref().and_then(|t| self.mob(t.0));
                npc.is_some_and(|m| i64::from(m.disposition) >= n(1))
            }
            "skill" => h.proficient_skill(c[1].as_str().unwrap_or("")),
            "not" => !self.cond(i, &c[1]),
            _ => false,
        }
    }

    /// Efectos: ["give", ítem, n], ["take", ítem, n], ["gold", ±n], ["flag", clave, valor], ["add", clave, ±n],
    /// ["log", texto] (sólo para el que habla), ["map"] (el mapa del piso), ["disposition", ±n] (su actitud),
    /// ["shop"] (abre su tienda), ["quest", misión, "start" | "done"], ["place", ítem] (lo deja en unos restos de esqueleto del piso; si no hay,
    /// en unos nuevos), ["attack"] (lo vuelve hostil), ["end"].
    fn apply(&mut self, i: usize, id: u32, e: &Value) {
        let n = |k: usize| e[k].as_i64().unwrap_or(0);
        let key = e[1].as_str().unwrap_or("").to_string();
        match e[0].as_str().unwrap_or("") {
            "give" => {
                // de su inventario si lo tiene; si no, uno nuevo
                let Some(model) = Item::new(&self.items, &key) else { return };
                let count = n(2).clamp(1, 999) as i32;
                let name = self.item_name(&model);
                for _ in 0..count {
                    let one = self.npc_take_one(id, &key).unwrap_or_else(|| model.clone());
                    if let Err(it) = self.collect(i, one) {
                        let pos = self.heroes[i].ch.pos;
                        self.floor_mut().level.drop(it, pos, HeapKind::Heap); // mochila llena: al suelo
                    }
                }
                self.tell(i, &trf!("Recibes {name}", "You receive {name}"));
            }
            "take" => {
                // pasa a su inventario
                let mut left = n(2).clamp(1, 999) as i32;
                let mut taken = Vec::new();
                let h = &mut self.heroes[i];
                for b in h.backpack.iter_mut().filter(|b| b.id == key) {
                    let t = left.min(b.quantity);
                    if t > 0 {
                        taken.push(b.split(t));
                    }
                    left -= t;
                }
                h.backpack.retain(|b| b.quantity > 0);
                for it in taken {
                    self.npc_store(id, it);
                }
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
                self.tell(i, tr!("Ahora conoces el mapa de este piso", "You now know the map of this floor"));
            }
            "disposition" => {
                let Some(m) = self.mob_mut(id) else { return };
                let before = m.disposition;
                m.disposition = (m.disposition + n(1) as i32).clamp(-2, 2);
                let (after, name) = (m.disposition, m.ch.name.clone());
                if after != before {
                    let msg = trf!("{} se vuelve {}", "{} becomes {}", crate::items::capitalize(&name), attitude(after));
                    self.tell(i, &msg);
                }
            }
            "attack" => {
                // traición: lo que quede de charla se corta; el ataque inicia el combate (y lo sorprende)
                self.npc_attacked(id);
                self.heroes[i].talking = None;
                if let Some(m) = self.mob_mut(id) {
                    m.state = State::Wandering;
                    m.alerted = true;
                }
                self.heroes[i].action = Some(Action::Attack(ActorId::Mob(id)));
            }
            "end" => self.heroes[i].talking = None,
            "shop" => {
                self.heroes[i].talking = None;
                self.heroes[i].trading = Some(id);
            }
            "quest" => {
                let qname = self.quests.get(&key).map_or_else(|| key.clone(), |q| text(&q["name"]));
                match e[2].as_str() {
                    Some("start") if self.quest_state(&key) == 0 => {
                        self.flags.insert(format!("quest:{key}"), 1);
                        // las muertes cuentan desde ahora
                        let objectives = self.quests.get(&key).and_then(|q| q["objectives"].as_array()).cloned();
                        let kills: Vec<String> = objectives
                            .into_iter()
                            .flatten()
                            .filter(|o| o[0] == "kill")
                            .filter_map(|o| o[1].as_str().map(str::to_string))
                            .collect();
                        for mob in kills {
                            let now = self.flags.get(&format!("kills:{mob}")).copied().unwrap_or(0);
                            self.flags.insert(format!("qbase:{key}:{mob}"), now);
                        }
                        self.log(&trf!("Nueva misión: {qname}", "New quest: {qname}"));
                    }
                    Some("done") if self.quest_state(&key) == 1 => {
                        self.flags.insert(format!("quest:{key}"), 2);
                        let who = self.heroes[i].ch.name.clone();
                        self.log(&trf!("{who} completa la misión: {qname}", "{who} completes the quest: {qname}"));
                    }
                    _ => {}
                }
            }
            "place" => {
                let Some(item) = Item::new(&self.items, &key) else { return };
                let bones: Vec<i32> = self.level().heaps.iter().filter(|(_, h)| h.kind == HeapKind::Skeleton).map(|(c, _)| *c).collect();
                let mut rng = self.rng.clone();
                let cell = match rng.pick(bones.len()) {
                    Some(k) => bones[k],
                    None => self.level().random_drop_cell(&mut rng),
                };
                self.rng = rng;
                self.floor_mut().level.drop(item, cell, HeapKind::Skeleton);
            }
            _ => {}
        }
    }

    /// Cuántos tiene el héroe i de ese ítem (mochila y equipo).
    fn count_item(&self, i: usize, id: &str) -> i64 {
        let h = &self.heroes[i];
        let worn = [h.weapon.as_ref(), h.armor.as_ref()].into_iter().flatten();
        h.backpack.iter().chain(worn).filter(|b| b.id == id).map(|b| i64::from(b.quantity)).sum()
    }

    /// 0 sin empezar, 1 en curso, 2 hecha.
    fn quest_state(&self, q: &str) -> i64 {
        self.flags.get(&format!("quest:{q}")).copied().unwrap_or(0)
    }

    /// El progreso de cada objetivo de la misión: (qué, cuántos, cuántos pide).
    fn objectives(&self, i: usize, q: &str) -> Vec<(String, i64, i64)> {
        let Some(qd) = self.quests.get(q) else { return Vec::new() };
        let list = qd["objectives"].as_array().into_iter().flatten();
        list.map(|o| {
            let id = o[1].as_str().unwrap_or("");
            let want = o[2].as_i64().unwrap_or(1).max(1);
            let name = |m: &serde_json::Map<String, Value>| m.get(id).map_or_else(|| id.to_string(), |v| text(&v["name"]));
            match o[0].as_str().unwrap_or("") {
                "kill" => {
                    let now = self.flags.get(&format!("kills:{id}")).copied().unwrap_or(0);
                    let base = self.flags.get(&format!("qbase:{q}:{id}")).copied().unwrap_or(0);
                    (name(&self.mobs_data), (now - base).clamp(0, want), want)
                }
                "has" => (name(&self.items), self.count_item(i, id).min(want), want),
                _ => ("?".to_string(), 0, want),
            }
        })
        .collect()
    }

    fn quest_ready(&self, i: usize, q: &str) -> bool {
        self.quest_state(q) == 1 && self.objectives(i, q).iter().all(|o| o.1 >= o.2)
    }

    /// Las misiones empezadas, para la ficha: "Ratas en la despensa: rata marsupial 3/5" o "...: hecha".
    pub fn quest_lines(&self, i: usize) -> Vec<String> {
        self.quests
            .iter()
            .filter_map(|(q, qd)| {
                let name = text(&qd["name"]);
                match self.quest_state(q) {
                    1 => {
                        let prog: Vec<String> = self.objectives(i, q).iter().map(|(t, h, n)| format!("{t} {h}/{n}")).collect();
                        Some(format!("{name}: {}", prog.join(", ")))
                    }
                    2 => Some(trf!("{name}: hecha", "{name}: done")),
                    _ => None,
                }
            })
            .collect()
    }
}
