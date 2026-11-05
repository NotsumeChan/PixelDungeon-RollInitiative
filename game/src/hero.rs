//! actors/hero/Hero.java + HeroClass.java + buffs/Regeneration.java (`game/hero.py`), con el combate de D&D 5e.
//! Las clases son datos (`data/classes.json`). R3: moverse, atacar cuerpo a cuerpo, esperar, buscar, escaleras,
//! estabilizar a un aliado caído, salvaciones contra la muerte, regeneración, XP y niveles.
//! R5: mochila (Belongings.java), oro, accesos rápidos, recoger y abrir, comer, beber, leer, equipar, soltar y el
//! hambre (buffs/Hunger.java).
//! R5d: armas a distancia (MissileWeapon.java): lanzar arrojadizas (el bumerán vuelve), el arco con flechas, cobertura,
//! desventaja más allá del alcance normal o con un enemigo al lado, y el estado que pone un dardo al acertar.
use serde_json::{Map, Value};

use crate::actor::ActorId;
use crate::char::{adjacent, distance, Char};
use crate::dice;
use crate::dungeon::{Dungeon, Fx, Pending, Tone};
use crate::geom::{LENGTH, NEIGHBOURS9};
use crate::items::{armor_ac, capitalize, HeapKind, Item, GOLD};
use crate::registry::text;
use crate::mobs::State;

pub const TIME_TO_REST: f64 = 1.0;
pub const TIME_TO_SEARCH: f64 = 2.0;
pub const REGENERATION_DELAY: f64 = 10.0;
pub const BACKPACK_SIZE: usize = 19;
pub const TIME_TO_PICK_UP: f64 = 1.0;
/// Key.TIME_TO_UNLOCK: abrir un cofre, una tumba o unos restos.
pub const TIME_TO_OPEN: f64 = 1.0;
pub const TIME_TO_DROP: f64 = 0.5;
pub const TIME_TO_EAT: f64 = 3.0;
/// Beber / leer.
pub const TIME_TO_USE: f64 = 1.0;
/// Arma; la armadura tarda el doble.
pub const TIME_TO_EQUIP: f64 = 1.0;
/// Item.TIME_TO_THROW (por la velocidad del arma).
pub const TIME_TO_THROW: f64 = 1.0;
pub const HUNGER_STEP: f64 = 10.0;
pub const HUNGRY: f64 = 260.0;
pub const STARVING: f64 = 360.0;

/// Dónde está un ítem del héroe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    Weapon,
    Armor,
    Pack(usize),
}

/// Qué se hace con un ítem (WndItem).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verb {
    Eat,
    Drink,
    Read,
    Equip,
    Unequip,
    /// Arrojadizas: elegir la casilla.
    Throw,
    Drop,
}

impl Verb {
    pub fn label(self) -> &'static str {
        match self {
            Verb::Eat => "COMER",
            Verb::Drink => "BEBER",
            Verb::Read => "LEER",
            Verb::Equip => "EQUIPAR",
            Verb::Unequip => "QUITAR",
            Verb::Throw => "LANZAR",
            Verb::Drop => "SOLTAR",
        }
    }
}

/// Lo que el héroe hace en sus turnos (Hero.curAction). Se repite turno a turno hasta cumplirse o cortarse.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Move(i32),
    Attack(ActorId),
    /// Acercarse a un aliado caído y estabilizarlo.
    Help(usize),
    Rest,
    Search,
    Descend(i32),
    Ascend(i32),
    /// Recoger el montón de esa celda (parado encima).
    PickUp(i32),
    /// Abrir el cofre / tumba / restos de esa celda (desde al lado o encima).
    Open(i32),
    /// Usar un ítem; el último es el objetivo de un pergamino (identificar, mejorar).
    Use(Slot, Verb, Option<Slot>),
    /// Lanzar el conjuro k de la clase (spells.rs) sobre la celda.
    Cast(usize, i32),
    /// Segundo Aliento (acción adicional: no gasta tiempo).
    SecondWind,
    /// Lanzar una unidad de la arrojadiza hacia la celda.
    Throw(Slot, i32),
    /// Hablar con un NPC (acercarse si hace falta): abre su diálogo. No gasta tiempo.
    Talk(u32),
    /// Elegir la opción k del diálogo abierto.
    Choose(usize),
    /// Cerrar el diálogo.
    Bye,
}

pub struct Hero {
    pub ch: Char,
    pub class_key: String,
    pub cls: Value,
    /// Características en el orden de `dice::ABILITIES`.
    pub abilities: [i32; 6],
    pub lvl: i32,
    pub exp: i32,
    pub awareness: f64,
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub backpack: Vec<Item>,
    pub gold: i32,
    /// Accesos rápidos: el id del ítem (el primero que se tenga con ese id).
    /// ponytail: por id también el equipo; dos iguales no se distinguen
    pub quickslots: [Option<String>; 2],
    pub hunger: f64,
    /// Segundo Aliento disponible (se recarga con `recover`).
    pub second_wind: bool,
    /// Espacios de conjuro disponibles por nivel, y usos gratis (Enemigo predilecto: Marca del cazador).
    pub slots: Vec<i32>,
    pub free_casts: Vec<(String, i32)>,
    /// Quién tiene su Marca del cazador.
    pub marked: Option<ActorId>,
    pub action: Option<Action>,
    /// Hero.rangedWeapon: el arma del ataque a distancia en curso (la arrojadiza o el arco) y la cobertura del
    /// objetivo (resta a la tirada). Sólo mientras dura el ataque.
    pub ranged: Option<Item>,
    pub cover: i32,
    /// true tras moverse: el render lo baja al terminar la animación del paso (la lógica espera).
    pub animating: bool,
    pub visible_enemies: Vec<ActorId>,
    /// Piso en el que está (0 = todavía en ninguno).
    pub depth: i32,
    pub fov: Vec<bool>,
    // salvaciones contra la muerte (5e)
    pub dead: bool,
    pub saves: [i32; 2],
    pub stable: bool,
    /// Diálogo abierto: (id del NPC, nodo).
    pub talking: Option<(u32, String)>,
    /// Apariencia (id de `data/skins.json`), aparte de la clase. Vacía = la de su clase (`Dungeon::hero_sprite`).
    pub skin: String,
}

impl Hero {
    pub fn new(classes: &Map<String, Value>, items: &Map<String, Value>, class_key: &str, abilities: [i32; 6]) -> Self {
        let cls = classes.get(class_key).unwrap_or_else(|| panic!("clase desconocida: {class_key}")).clone();
        let item = |k: &str| {
            let mut i = cls.get(k).and_then(Value::as_str).and_then(|id| Item::new(items, id))?;
            i.level_known = true; // el equipo inicial se conoce
            i.cursed_known = true;
            Some(i)
        };
        let mut backpack = Vec::new();
        for e in cls.get("items").and_then(Value::as_array).into_iter().flatten() {
            let (id, qty) = match e {
                Value::Array(a) => (a[0].as_str().unwrap_or(""), a[1].as_i64().unwrap_or(1) as i32),
                v => (v.as_str().unwrap_or(""), 1),
            };
            if let Some(mut i) = Item::new(items, id) {
                i.quantity = qty;
                i.level_known = true;
                backpack.push(i);
            }
        }
        let quick = cls.get("quick").and_then(Value::as_str).map(str::to_string);
        let mut h = Self {
            ch: Char::new(-1, text(&cls["name"])),
            class_key: class_key.to_string(),
            abilities,
            lvl: 1,
            exp: 0,
            awareness: 0.0,
            weapon: item("start_weapon"),
            armor: item("start_armor"),
            backpack,
            gold: 0,
            quickslots: [quick, None],
            hunger: 0.0,
            second_wind: true,
            slots: Vec::new(),
            free_casts: Vec::new(),
            marked: None,
            action: None,
            ranged: None,
            cover: 0,
            animating: false,
            visible_enemies: Vec::new(),
            depth: 0,
            fov: vec![false; LENGTH as usize],
            dead: false,
            saves: [0, 0],
            stable: false,
            talking: None,
            skin: String::new(),
            cls,
        };
        h.ch.base_speed_ft = h.cls.get("speed_ft").and_then(Value::as_i64).unwrap_or(30) as i32;
        // PV de nivel 1 en 5e: dado de golpe al máximo + modificador de CON
        h.ch.ht = (h.hit_die() + h.ability_mod("CON")).max(1);
        h.ch.hp = h.ch.ht;
        h.update_awareness();
        h.recover();
        h
    }

    fn hit_die(&self) -> i32 {
        self.cls["hit_die"].as_i64().unwrap_or(8) as i32
    }

    fn class_has(&self, list: &str, value: &str) -> bool {
        self.cls.get(list).and_then(Value::as_array).is_some_and(|a| a.iter().any(|v| v.as_str() == Some(value)))
    }

    // --- características (5e) ---
    pub fn score(&self, ability: &str) -> i32 {
        dice::ABILITIES.iter().position(|a| *a == ability).map_or(10, |i| self.abilities[i])
    }

    pub fn ability_mod(&self, ability: &str) -> i32 {
        dice::modifier(self.score(ability))
    }

    pub fn proficiency(&self) -> i32 {
        dice::proficiency(self.lvl)
    }

    /// ¿Tiene la clase competencia en esta pericia (`skill_prof`)? El bono lo da `Dungeon::skill_bonus`.
    pub fn proficient_skill(&self, skill: &str) -> bool {
        self.class_has("skill_prof", skill)
    }

    /// Salvación: mod de la característica + competencia si la clase la tiene (`save_prof`).
    pub fn save_bonus(&self, ability: &str) -> i32 {
        self.ability_mod(ability) + if self.class_has("save_prof", ability) { self.proficiency() } else { 0 }
    }

    pub fn proficient_weapon(&self) -> bool {
        self.proficient_weapon_item(self.wep())
    }

    /// El arma del ataque en curso: la arrojadiza / el arco mientras se dispara; si no, la equipada.
    pub fn wep(&self) -> Option<&Item> {
        self.ranged.as_ref().or(self.weapon.as_ref())
    }

    /// Alcance (normal, largo) en casillas de un arma a distancia.
    pub fn range_of(item: &Item) -> (i32, i32) {
        let r = &item.data["range"];
        (r[0].as_i64().unwrap_or(1) as i32, r[1].as_i64().unwrap_or(1) as i32)
    }

    /// La pila de munición del arco equipado en la mochila.
    pub fn ammo(&self) -> Option<usize> {
        let w = self.weapon.as_ref().filter(|w| w.flag("ranged"))?;
        self.backpack.iter().position(|i| i.id == w.str("ammo"))
    }

    /// Sin arma (puños): siempre competente.
    pub fn proficient_weapon_item(&self, w: Option<&Item>) -> bool {
        w.is_none_or(|w| self.class_has("weapon_prof", &w.id) || self.class_has("weapon_prof", w.str("kind")))
    }

    pub fn proficient_armor(&self) -> bool {
        self.proficient_armor_item(self.armor.as_ref())
    }

    pub fn proficient_armor_item(&self, a: Option<&Item>) -> bool {
        a.is_none_or(|a| a.str("kind") == "clothing" || self.class_has("armor_prof", a.str("kind")))
    }

    pub fn is_starving(&self) -> bool {
        self.hunger >= STARVING
    }

    pub fn item(&self, slot: Slot) -> Option<&Item> {
        match slot {
            Slot::Weapon => self.weapon.as_ref(),
            Slot::Armor => self.armor.as_ref(),
            Slot::Pack(k) => self.backpack.get(k),
        }
    }

    /// El equipo puesto y la mochila, con dónde está cada uno.
    pub fn all_items(&self) -> Vec<(Slot, &Item)> {
        let mut v: Vec<(Slot, &Item)> = Vec::new();
        if let Some(w) = &self.weapon {
            v.push((Slot::Weapon, w));
        }
        if let Some(a) = &self.armor {
            v.push((Slot::Armor, a));
        }
        v.extend(self.backpack.iter().enumerate().map(|(k, i)| (Slot::Pack(k), i)));
        v
    }

    /// El ítem del acceso rápido q (None si no hay o ya no se tiene).
    pub fn quick_item(&self, q: usize) -> Option<Slot> {
        let id = self.quickslots[q].as_ref()?;
        self.all_items().into_iter().find(|(_, i)| &i.id == id).map(|(s, _)| s)
    }

    /// Qué se puede hacer con el ítem (la primera es la principal: accesos rápidos).
    pub fn verbs(&self, slot: Slot) -> Vec<Verb> {
        let Some(item) = self.item(slot) else { return Vec::new() };
        let mut v = Vec::new();
        if item.is_equipment() {
            v.push(if matches!(slot, Slot::Pack(_)) { Verb::Equip } else { Verb::Unequip });
        } else {
            match item.category() {
                _ if item.flag("thrown") && matches!(slot, Slot::Pack(_)) => v.push(Verb::Throw),
                "food" => v.push(Verb::Eat),
                "potion" => v.push(Verb::Drink),
                "scroll" => v.push(Verb::Read),
                _ => {}
            }
        }
        v.push(Verb::Drop);
        v
    }

    /// Puntos de FUE que faltan para usar bien el ítem (0 si alcanza).
    pub fn str_missing(&self, item: Option<&Item>) -> i32 {
        item.map_or(0, |i| (i.int("STR") - self.score("STR")).max(0))
    }

    // --- combate 5e ---
    pub fn attack_mod(&self) -> i32 {
        let w = self.wep();
        if self.ranged.is_some() && w.is_some_and(|w| w.flag("ranged")) {
            return self.ability_mod("DEX"); // arco
        }
        // Artes marciales (Monje): sin armas o con armas simples, FUE o DES (la mayor)
        let martial = self.has_trait("pdri:martial_arts") && w.is_none_or(|w| w.str("kind") == "simple");
        if martial || w.is_some_and(|w| w.flag("finesse")) {
            return self.ability_mod("STR").max(self.ability_mod("DEX"));
        }
        self.ability_mod("STR")
    }

    /// CA (5e). Defensa sin armadura (`unarmored_ac` de la clase: Bárbaro CON, Monje SAB): sin armadura (la ropa no
    /// cuenta), 10 + DES + esa característica si es mejor.
    pub fn armor_class(&self) -> i32 {
        let worn = armor_ac(self.armor.as_ref(), self.ability_mod("DEX"));
        match self.cls.get("unarmored_ac").and_then(Value::as_str) {
            Some(ab) if self.armor.as_ref().is_none_or(|a| a.str("kind") == "clothing") => {
                worn.max(10 + self.ability_mod("DEX") + self.ability_mod(ab))
            }
            _ => worn,
        }
    }

    pub fn attack_bonus(&self) -> i32 {
        let prof = if self.proficient_weapon() { self.proficiency() } else { 0 };
        let level = self.wep().map_or(0, |w| w.level);
        self.attack_mod() + prof + level - self.str_missing(self.wep()) - self.cover
    }

    /// Desventaja propia: arma o armadura demasiado pesada, o armadura sin competencia.
    pub fn disadvantaged(&self) -> bool {
        self.str_missing(self.wep()) > 0 || self.str_missing(self.armor.as_ref()) > 0 || !self.proficient_armor()
    }

    pub fn attack_delay(&self) -> f64 {
        self.weapon.as_ref().and_then(|w| w.data.get("dly")).and_then(Value::as_f64).unwrap_or(1.0)
    }

    /// Del original: armadura muy pesada = más lento fuera de combate.
    pub fn speed(&self) -> f64 {
        let enc = self.armor.as_ref().map_or(0, |a| a.int("STR") - self.score("STR"));
        let base = if enc > 0 { self.ch.base_speed / 1.3f64.powi(enc) } else { self.ch.base_speed };
        base * self.ch.buffs.iter().map(|b| b.speed()).product::<f64>()
    }

    /// 5e: armadura pesada sin la FUE que pide = -10 ft.
    /// Sin estados (los suma `Dungeon::start_turn`).
    pub fn speed_ft_base(&self) -> i32 {
        let heavy = self.armor.as_ref().is_some_and(|a| a.str("kind") == "heavy" && a.int("STR") > self.score("STR"));
        (self.ch.base_speed_ft - if heavy { 10 } else { 0 }).max(5)
    }

    pub fn initiative_bonus(&self) -> i32 {
        self.ability_mod("DEX")
    }

    /// Más difícil que los mobs te noten.
    pub fn stealth(&self) -> i32 {
        self.ability_mod("DEX").max(0)
    }

    /// Sube un nivel (5e): promedio del dado de golpe + CON (mínimo 1) de PV máximos, percepción y espacios de
    /// conjuro. Devuelve los PV ganados (los actuales los suma quien llama).
    pub(crate) fn gain_level(&mut self) -> i32 {
        self.lvl += 1;
        let gain = (self.hit_die() / 2 + 1 + self.ability_mod("CON")).max(1);
        self.ch.ht += gain;
        if self.lvl < 10 {
            self.update_awareness();
        }
        self.level_up_spells();
        gain
    }

    pub fn max_exp(&self) -> i32 {
        5 + self.lvl * 5
    }

    pub(crate) fn update_awareness(&mut self) {
        let base = self.cls["awareness_base"].as_f64().unwrap_or(0.9);
        self.awareness = 1.0 - base.powf((1 + self.lvl.min(9)) as f64 * 0.5);
    }

    // --- a 0 PV: inconsciente y salvaciones contra la muerte ---
    pub fn is_alive(&self) -> bool {
        !self.dead
    }

    pub fn is_downed(&self) -> bool {
        !self.dead && self.ch.hp <= 0
    }

    /// El planificador se detiene en él: esperando input o la animación del paso.
    pub fn waiting(&self) -> bool {
        if !self.is_alive() || self.animating {
            return true;
        }
        // paralizado o caído: los turnos pasan solos
        self.action.is_none() && !self.is_downed() && !self.ch.buffs.iter().any(|b| b.has_flag("paralysed"))
    }
}

impl Dungeon {
    /// Hero.handle: decide la acción según qué hay en la celda. stairs = true sólo con click / toque: cambiar de piso
    /// hay que pedirlo; pisar la escalera no basta.
    pub fn handle(&mut self, i: usize, cell: i32, stairs: bool) {
        if !(0..LENGTH).contains(&cell) {
            return;
        }
        let seen = self.heroes[i].fov[cell as usize];
        let ch = self.find_char(cell);
        let level = self.level();
        let action = match ch {
            Some(ActorId::Hero(j)) if seen && j != i => {
                if self.heroes[j].is_downed() { Action::Help(j) } else { Action::Move(cell) }
            }
            Some(ActorId::Mob(id)) if seen && self.passive(ActorId::Mob(id)) => Action::Talk(id),
            Some(a) if seen && a != ActorId::Hero(i) => Action::Attack(a),
            // cofre, tumba o restos: se abren desde al lado; un montón se recoge parado encima
            _ if seen && level.heaps.contains_key(&cell) => {
                if level.heaps[&cell].kind == HeapKind::Heap { Action::PickUp(cell) } else { Action::Open(cell) }
            }
            _ if stairs && cell == level.exit => Action::Descend(cell),
            _ if stairs && cell == level.entrance => Action::Ascend(cell),
            _ => Action::Move(cell),
        };
        self.heroes[i].action = Some(action);
    }

    pub(crate) fn hero_act(&mut self, i: usize) {
        self.check_visible_mobs(i);
        let me = ActorId::Hero(i);
        if self.heroes[i].is_downed() {
            self.heroes[i].action = None;
            if !self.heroes[i].stable {
                self.death_save(i);
            }
            self.spend(me, TIME_TO_REST);
            return;
        }
        if self.has_flag(me, "paralysed") {
            self.heroes[i].action = None;
            self.spend(me, TIME_TO_REST);
            return;
        }
        let Some(action) = self.heroes[i].action else { return }; // interrumpido: espera input sin gastar tiempo
        let pos = self.heroes[i].ch.pos;
        match action {
            Action::Rest => {
                self.heroes[i].action = None;
                self.spend(me, TIME_TO_REST);
            }
            Action::Search => {
                self.heroes[i].action = None;
                self.search(i, true);
                self.spend(me, TIME_TO_SEARCH);
            }
            Action::Throw(slot, cell) => {
                // lanzarle a un mob fuera de combate lo inicia (como atacar); el tiro va en su turno
                let stop = self.shot(pos, cell).0;
                let h = &self.heroes[i];
                let valid = h.fov[cell as usize] && h.item(slot).is_some_and(|it| distance(pos, stop) <= Hero::range_of(it).1);
                let at_mob = valid && matches!(self.find_char(stop), Some(ActorId::Mob(_)));
                if at_mob && !self.floor().scheduler.in_combat(me) {
                    self.start_combat(true);
                } else {
                    self.heroes[i].action = None;
                    self.throw(i, slot, cell);
                }
            }
            Action::Attack(e) if self.can_shoot(i, e) => {
                // con arco: dispara desde donde está
                if self.floor().scheduler.in_combat(me) {
                    self.heroes[i].action = None;
                    self.shoot(i, e);
                } else {
                    self.start_combat(true);
                }
            }
            Action::Attack(e) => {
                let target = if self.alive(e) { self.ch(e).map(|c| c.pos) } else { None };
                match target {
                    Some(tp) if adjacent(pos, tp) && !self.floor().scheduler.in_combat(me) => {
                        // 5e: atacar inicia el combate. Todos tiran iniciativa y el ataque va en su turno; los que no
                        // lo vieron venir quedan sorprendidos (pierden su primer turno)
                        self.start_combat(true);
                    }
                    Some(tp) if adjacent(pos, tp) => {
                        self.heroes[i].action = None;
                        self.attack(me, e);
                        let dly = self.heroes[i].attack_delay();
                        self.spend(me, dly);
                    }
                    Some(tp) if self.heroes[i].fov[tp as usize] && self.hero_get_closer(i, tp) => {}
                    _ => self.heroes[i].action = None,
                }
            }
            Action::Talk(id) => {
                let tp = self.mob(id).filter(|_| self.passive(ActorId::Mob(id))).map(|m| m.ch.pos);
                match tp {
                    Some(_) if self.floor().scheduler.in_combat(me) => {
                        self.heroes[i].action = None;
                        self.tell(i, "En pleno combate no hay tiempo para charlar");
                    }
                    Some(tp) if adjacent(pos, tp) => {
                        self.heroes[i].action = None;
                        self.talk(i, id);
                    }
                    Some(tp) if self.heroes[i].fov[tp as usize] && self.hero_get_closer(i, tp) => {}
                    _ => self.heroes[i].action = None,
                }
            }
            Action::Choose(k) => {
                self.heroes[i].action = None;
                self.choose(i, k);
            }
            Action::Bye => {
                self.heroes[i].action = None;
                self.heroes[i].talking = None;
            }
            Action::Help(j) => {
                let ally = self.heroes[j].ch.pos;
                if adjacent(pos, ally) {
                    self.heroes[i].action = None;
                    self.stabilize(i, j);
                } else if !(self.heroes[j].is_downed() && self.hero_get_closer(i, ally)) {
                    self.heroes[i].action = None;
                }
            }
            Action::Cast(k, cell) => {
                self.heroes[i].action = None;
                self.cast(i, k, cell);
            }
            Action::SecondWind => {
                self.heroes[i].action = None;
                self.second_wind(i);
            }
            Action::Use(slot, verb, target) => {
                self.heroes[i].action = None;
                self.use_item(i, slot, verb, target);
            }
            Action::Open(c) if pos == c || adjacent(pos, c) => {
                self.heroes[i].action = None;
                self.open_heap(i, c);
            }
            Action::Descend(c) | Action::Ascend(c) | Action::PickUp(c) if pos == c => {
                self.heroes[i].action = None;
                match action {
                    Action::Descend(_) => self.descend(i),
                    Action::Ascend(_) => self.ascend(i),
                    _ => self.pick_up(i),
                }
            }
            Action::Move(c) | Action::Descend(c) | Action::Ascend(c) | Action::PickUp(c) | Action::Open(c) => {
                if !self.hero_get_closer(i, c) {
                    self.heroes[i].action = None;
                }
            }
        }
    }

    /// Si aparece un enemigo nuevo a la vista, se corta la acción en curso.
    fn check_visible_mobs(&mut self, i: usize) {
        let fov = &self.heroes[i].fov;
        let visible: Vec<ActorId> =
            self.floor().mobs.iter().filter(|m| fov[m.ch.pos as usize] && m.state != State::Passive).map(|m| ActorId::Mob(m.id)).collect();
        let h = &mut self.heroes[i];
        if visible.iter().any(|m| !h.visible_enemies.contains(m)) {
            h.action = None;
        }
        h.visible_enemies = visible;
    }

    fn hero_get_closer(&mut self, i: usize, target: i32) -> bool {
        let pos = self.heroes[i].ch.pos;
        if pos == target {
            return false;
        }
        if !self.can_step(ActorId::Hero(i)) {
            return true; // sin movimiento en este turno de combate: sigue en el próximo
        }
        let level = self.level();
        let t = target as usize;
        let step = if adjacent(pos, target) {
            // ponytail: el abismo está bloqueado; en Java pide confirmación y caes al piso siguiente
            let free = self.find_char(target).is_none() && !level.pit[t];
            if free && (level.passable[t] || level.avoid[t]) { target } else { -1 }
        } else {
            // sólo camina por lo ya visto (visited / mapped)
            let p: Vec<bool> =
                (0..LENGTH as usize).map(|c| level.passable[c] && (level.visited[c] || level.mapped[c])).collect();
            self.find_path(false, pos, target, &p, &self.heroes[i].fov)
        };
        if step == -1 {
            return false;
        }
        self.move_char(ActorId::Hero(i), step);
        let speed = self.heroes[i].speed();
        self.spend(ActorId::Hero(i), 1.0 / speed);
        self.heroes[i].animating = true;
        self.observe();
        self.search(i, false);
        true
    }

    /// Busca secretos en las celdas vecinas visibles. Pasiva (tras cada paso) con probabilidad `awareness`.
    fn search(&mut self, i: usize, intentional: bool) {
        let pos = self.heroes[i].ch.pos;
        let mut found = false;
        for n in NEIGHBOURS9 {
            let p = pos + n;
            if !self.heroes[i].fov[p as usize] {
                continue;
            }
            if intentional {
                self.effect(Fx::Checked(p)); // destello azul en cada casilla revisada
            }
            if self.level().secret[p as usize] && (intentional || self.rng.float() < self.heroes[i].awareness) {
                self.floor_mut().level.discover(p);
                found = true;
            }
        }
        if intentional {
            self.status(ActorId::Hero(i), "buscando", Tone::Default);
            self.effect(Fx::Anim { who: ActorId::Hero(i), kind: "operate", cell: pos });
        }
        if found {
            self.tell(i, "Encuentras algo oculto");
            self.observe();
        } else if intentional {
            self.tell(i, "Buscas, pero no encuentras nada");
        }
    }

    pub(crate) fn earn_exp(&mut self, i: usize, exp: i32) {
        self.heroes[i].exp += exp;
        while self.heroes[i].exp >= self.heroes[i].max_exp() {
            let h = &mut self.heroes[i];
            h.exp -= h.max_exp();
            let gain = h.gain_level();
            if h.ch.hp > 0 {
                h.ch.hp += gain; // caído (XP repartida) no se levanta por subir de nivel
            }
            let msg = format!("Subes a nivel {}: +{gain} PV, competencia +{}", h.lvl, h.proficiency());
            self.tell(i, &msg);
            self.status(ActorId::Hero(i), "sube de nivel", Tone::Pos);
        }
    }

    /// Cura hasta el máximo y devuelve lo curado. Cualquier curación levanta al caído.
    pub fn heal(&mut self, i: usize, amount: i32) -> i32 {
        let h = &mut self.heroes[i];
        let amount = amount.min(h.ch.ht - h.ch.hp).max(0);
        let was_down = h.is_downed();
        h.ch.hp += amount;
        if was_down && h.ch.hp > 0 {
            let msg = format!("{} recupera la conciencia", h.ch.name);
            self.log(&msg);
            self.status(ActorId::Hero(i), "se levanta", Tone::Pos);
        }
        amount
    }

    /// 5e: a 0 PV cae inconsciente (salvo daño masivo: lo que sobra >= PV máximos = muere). Caído, cada daño es un
    /// fallo de salvación, dos si el atacante está al lado (ese golpe es crítico), y deja de estar estable.
    pub(crate) fn hero_damage(&mut self, i: usize, dmg: i32, src: Option<ActorId>) {
        self.heroes[i].action = None;
        if self.heroes[i].dead {
            return;
        }
        let src_pos = src.and_then(|s| self.ch(s)).map(|c| c.pos);
        let src_name = src.and_then(|s| self.source_name(s)).unwrap_or_else(|| "daño".to_string());
        // daño por turno (un estado) a un caído: según la regla de la partida `dot_death_saves`
        let counts = self.rules.dot_death_saves || !matches!(src, Some(ActorId::Buff(_)));
        let h = &mut self.heroes[i];
        if h.ch.hp <= 0 {
            if dmg > 0 && counts {
                let n = if src_pos.is_some_and(|p| adjacent(p, h.ch.pos)) { 2 } else { 1 };
                h.stable = false;
                h.saves[1] += n;
                let msg = format!(
                    "{} recibe daño estando caído ({src_name}): {n} fallo{} ({}/3)",
                    h.ch.name,
                    if n > 1 { "s" } else { "" },
                    h.saves[1].min(3)
                );
                let dies = h.saves[1] >= 3;
                self.log(&msg);
                self.status(ActorId::Hero(i), "fallo", Tone::Neg);
                if dies {
                    self.hero_die(i, src);
                }
            }
            return;
        }
        if dmg > 0 {
            self.status(ActorId::Hero(i), &dmg.to_string(), Tone::Neg);
        }
        let h = &mut self.heroes[i];
        h.ch.hp -= dmg;
        if h.ch.hp <= 0 {
            let massive = -h.ch.hp >= h.ch.ht;
            h.ch.hp = 0;
            if massive {
                self.hero_die(i, src);
                return;
            }
            h.saves = [0, 0];
            h.stable = false;
            let msg = format!("{} cae inconsciente", h.ch.name);
            self.log(&msg);
            self.status(ActorId::Hero(i), "inconsciente", Tone::Neg);
        }
    }

    /// Sale del planificador con lo suyo: el piso sigue para los demás héroes; sin héroes vivos, nada avanza.
    /// ponytail: el muerto queda de espectador; volver con un personaje nuevo llega con la red (R6)
    pub(crate) fn hero_die(&mut self, i: usize, src: Option<ActorId>) {
        let cause = src.and_then(|s| self.source_name(s)).map_or(String::new(), |n| format!(" ({n})"));
        let h = &mut self.heroes[i];
        h.dead = true;
        h.ch.hp = 0;
        let s = &mut self.floors.get_mut(&self.depth).expect("piso en contexto").scheduler;
        s.remove(ActorId::Hero(i));
        s.remove(ActorId::Regen(i));
        s.remove(ActorId::Hunger(i));
        for b in &self.heroes[i].ch.buffs {
            s.remove(ActorId::Buff(b.uid));
        }
        let msg = if self.heroes.len() == 1 {
            format!("Has muerto{cause}. R = nueva partida")
        } else {
            format!("{} muere{cause}", self.heroes[i].ch.name)
        };
        self.log(&msg);
    }

    /// Al inicio de su turno: d20, 10+ éxito. 3 éxitos = estable, 3 fallos = muere, 20 = vuelve con 1 PV, 1 = dos
    /// fallos. Se aplica cuando el dado se muestra (pending).
    fn death_save(&mut self, i: usize) {
        let (nat, _) = dice::d20(&mut self.rng, 0);
        self.roll_event("Muerte CD10", nat, 1, 20, Some(nat >= 10));
        self.defer(Pending::DeathSave { hero: i, nat });
    }

    pub(crate) fn apply_death_save(&mut self, i: usize, nat: i32) {
        if !self.heroes[i].is_downed() || self.heroes[i].stable {
            return;
        }
        if nat == 20 {
            self.heal(i, 1);
            return;
        }
        let h = &mut self.heroes[i];
        if nat >= 10 {
            h.saves[0] += 1;
        } else {
            h.saves[1] += if nat == 1 { 2 } else { 1 };
        }
        let [s, f] = h.saves;
        let msg = format!("{}: salvación contra la muerte {nat} ({s} éxitos, {f} fallos)", h.ch.name);
        self.log(&msg);
        if f >= 3 {
            self.hero_die(i, None);
        } else if s >= 3 {
            self.heroes[i].stable = true;
            let msg = format!("{} está estable", self.heroes[i].ch.name);
            self.log(&msg);
            self.status(ActorId::Hero(i), "estable", Tone::Pos);
        }
    }

    /// Medicina (5e): d20 + SAB (+ competencia) contra CD 10 sobre un aliado caído al lado. Una acción.
    fn stabilize(&mut self, i: usize, ally: usize) {
        if !self.heroes[ally].is_downed() || self.heroes[ally].stable {
            return;
        }
        let (nat, _) = dice::d20(&mut self.rng, 0);
        let b = self.skill_bonus(i, "pdri:medicine");
        let ok = nat + b >= 10;
        self.roll_event("Medicina CD10", nat + b, 1 + b, 20 + b, Some(ok));
        let cell = self.heroes[ally].ch.pos;
        self.effect(Fx::Anim { who: ActorId::Hero(i), kind: "operate", cell });
        self.defer(Pending::Stabilize { hero: i, ally, ok });
        self.spend(ActorId::Hero(i), TIME_TO_REST);
    }

    pub(crate) fn apply_stabilize(&mut self, i: usize, ally: usize, ok: bool) {
        let (me, other) = (self.heroes[i].ch.name.clone(), self.heroes[ally].ch.name.clone());
        if ok && self.heroes[ally].is_downed() {
            self.heroes[ally].stable = true;
            self.log(&format!("{me} estabiliza a {other}"));
            self.status(ActorId::Hero(ally), "estable", Tone::Pos);
        } else if !ok {
            self.log(&format!("{me} no logra estabilizar a {other}"));
        }
    }

    /// buffs/Regeneration.java: +1 PV cada 10 de tiempo. Caído, sólo si está estable (despierta).
    pub(crate) fn regen_act(&mut self, i: usize) {
        let h = &self.heroes[i];
        if h.is_alive() && h.ch.hp < h.ch.ht && !h.is_starving() && (h.ch.hp > 0 || h.stable) {
            self.heal(i, 1);
        }
        self.spend(ActorId::Regen(i), REGENERATION_DELAY);
    }

    /// buffs/Hunger.java: +10 de hambre cada 10 de tiempo (Pícaro cada 12). Hambriento a 260, famélico a 360: ahí deja
    /// de regenerar y pierde 1 PV con 30% de probabilidad cada vez.
    pub(crate) fn hunger_act(&mut self, i: usize) {
        let h = &self.heroes[i];
        if h.is_alive() {
            if h.is_starving() {
                if self.rng.float() < 0.3 {
                    self.hero_damage(i, 1, Some(ActorId::Hunger(i))); // se puede morir de hambre (como en Java)
                }
            } else {
                let new = h.hunger + HUNGER_STEP;
                if new >= STARVING {
                    self.tell(i, "¡Te mueres de hambre!");
                    self.heroes[i].action = None;
                } else if new >= HUNGRY && HUNGRY > h.hunger {
                    self.tell(i, "Tienes hambre");
                }
                self.heroes[i].hunger = new.min(STARVING);
            }
        }
        let step = self.heroes[i].cls.get("hunger_step").and_then(Value::as_f64).unwrap_or(HUNGER_STEP);
        self.spend(ActorId::Hunger(i), step);
    }

    /// Mete un ítem en la mochila (apila si se puede). Si no hay lugar, lo devuelve.
    pub(crate) fn collect(&mut self, i: usize, item: Item) -> Result<(), Item> {
        let h = &mut self.heroes[i];
        if item.id == GOLD {
            h.gold += item.quantity;
            return Ok(());
        }
        if item.stackable()
            && let Some(same) = h.backpack.iter_mut().find(|b| b.id == item.id)
        {
            same.quantity += item.quantity;
            return Ok(());
        }
        if h.backpack.len() >= BACKPACK_SIZE {
            return Err(item);
        }
        h.backpack.push(item);
        Ok(())
    }

    // --- armas a distancia ---
    /// Arco equipado con flechas, enemigo vivo a la vista, dentro del alcance largo y con línea de tiro.
    pub(crate) fn can_shoot(&self, i: usize, e: ActorId) -> bool {
        let h = &self.heroes[i];
        let (Some(_), Some(w)) = (h.ammo(), h.weapon.as_ref()) else { return false };
        let Some(tp) = self.ch(e).filter(|_| self.alive(e)).map(|c| c.pos) else { return false };
        h.fov[tp as usize] && distance(h.ch.pos, tp) <= Hero::range_of(w).1 && self.shot(h.ch.pos, tp).0 == tp
    }

    /// Hero.shoot: ataque 5e con el arma a distancia (`ranged` y `cover` mientras dura).
    fn ranged_attack(&mut self, i: usize, enemy: ActorId, weapon: Item, cover: i32) -> bool {
        let epos = self.ch(enemy).map_or(-1, |c| c.pos);
        if cover > 0 && self.sees(epos) {
            let name = self.ch(enemy).map_or(String::new(), |c| c.name.clone());
            self.log(&format!("{name} tiene media cobertura (-{cover})"));
        }
        (self.heroes[i].ranged, self.heroes[i].cover) = (Some(weapon), cover);
        let hit = self.attack(ActorId::Hero(i), enemy);
        (self.heroes[i].ranged, self.heroes[i].cover) = (None, 0);
        hit
    }

    /// Lo que pasa cuando la tirada ya se vio (si se ve): el estado del arma, y el proyectil cae, se pierde o vuelve.
    fn after_shot(&mut self, i: usize, missile: Item, on_hit: Value, enemy: Option<ActorId>, hit: bool, stop: i32) {
        let p = Pending::Land { hero: i, missile, on_hit, enemy, hit, stop };
        if self.sees(stop) { self.pending.push(p) } else { self.apply_land(p) }
    }

    pub(crate) fn apply_land(&mut self, p: Pending) {
        let Pending::Land { hero, missile, on_hit, enemy, hit, stop } = p else { return };
        if let (true, Some(e), Some(o)) = (hit, enemy, on_hit.as_array()) {
            // [estado, duración (null = la suya), nivel, CD de la salvación]
            if self.alive(e) {
                let id = o[0].as_str().unwrap_or("");
                self.add_buff(e, id, o[1].as_f64(), o[2].as_i64().unwrap_or(0) as i32, o[3].as_i64().unwrap_or(0) as i32);
            }
        }
        if missile.flag("returns") {
            // el bumerán vuelve a la mano (si la mochila está llena, cae a los pies)
            // ponytail: sin proyectil en vuelo (falta el efecto de misiles, screens/particles.py)
            let pos = self.heroes[hero].ch.pos;
            if let Err(m) = self.collect(hero, missile) {
                self.floor_mut().level.drop(m, pos, HeapKind::Heap);
            }
        } else if !hit {
            self.floor_mut().level.drop(missile, stop, HeapKind::Heap);
        }
    }

    /// Item.cast + MissileWeapon.onThrow: lanza una unidad. Si se frena en un mob, es un ataque a distancia; contra un
    /// aliado o el suelo, cae ahí. Si no se puede, avisa y no gasta tiempo.
    fn throw(&mut self, i: usize, slot: Slot, cell: i32) {
        let h = &self.heroes[i];
        let pos = h.ch.pos;
        let Some(item) = h.item(slot).filter(|it| it.flag("thrown") && matches!(slot, Slot::Pack(_))) else { return };
        if cell == pos {
            return;
        }
        let range = Hero::range_of(item).1;
        if !h.fov[cell as usize] {
            self.tell(i, "No ves esa casilla");
            return;
        }
        let (stop, cover) = self.shot(pos, cell);
        if stop == pos {
            return;
        }
        if distance(pos, stop) > range {
            self.tell(i, "Está demasiado lejos");
            return;
        }
        let Some(one) = self.take_one(i, slot) else { return };
        let me = ActorId::Hero(i);
        self.effect(Fx::Anim { who: me, kind: "attack", cell: stop });
        let enemy = self.find_char(stop);
        let hit = matches!(enemy, Some(ActorId::Mob(_))) && self.ranged_attack(i, enemy.unwrap(), one.clone(), cover);
        let dly = one.data.get("dly").and_then(Value::as_f64).unwrap_or(1.0);
        let on_hit = one.data.get("on_hit").cloned().unwrap_or(Value::Null);
        self.after_shot(i, one, on_hit, enemy, hit, stop);
        self.spend(me, TIME_TO_THROW * dly);
    }

    /// Dispara el arco: gasta una flecha (si acierta se pierde; si falla, cae junto al enemigo).
    fn shoot(&mut self, i: usize, e: ActorId) {
        let (Some(k), Some(bow)) = (self.heroes[i].ammo(), self.heroes[i].weapon.clone()) else { return };
        let Some(arrow) = self.take_one(i, Slot::Pack(k)) else { return };
        let epos = self.ch(e).map_or(-1, |c| c.pos);
        let cover = self.shot(self.heroes[i].ch.pos, epos).1;
        let hit = self.ranged_attack(i, e, bow.clone(), cover);
        let on_hit = bow.data.get("on_hit").cloned().unwrap_or(Value::Null);
        self.after_shot(i, arrow, on_hit, Some(e), hit, epos);
        self.spend(ActorId::Hero(i), bow.data.get("dly").and_then(Value::as_f64).unwrap_or(1.0));
    }

    /// Saca una unidad del ítem (de una pila, o el ítem entero).
    fn take_one(&mut self, i: usize, slot: Slot) -> Option<Item> {
        let h = &mut self.heroes[i];
        match slot {
            Slot::Weapon => h.weapon.take(),
            Slot::Armor => h.armor.take(),
            Slot::Pack(k) if h.backpack.get(k)?.quantity > 1 => Some(h.backpack[k].split(1)),
            Slot::Pack(k) => Some(h.backpack.remove(k)),
        }
    }

    /// Hero.actOpenChest + Heap.open: lo de adentro queda en el suelo como un montón normal.
    /// ponytail: la tumba no invoca espectros ni los restos malditos hacen daño (faltan los Wraith)
    fn open_heap(&mut self, i: usize, cell: i32) {
        let Some(heap) = self.floor_mut().level.heaps.get_mut(&cell) else { return };
        let msg = match heap.kind {
            HeapKind::Heap => return,
            HeapKind::Tomb => "Abres la tumba",
            HeapKind::Skeleton => "Revisas los restos",
            HeapKind::Chest => "Abres el cofre",
        };
        heap.kind = HeapKind::Heap;
        if heap.items.is_empty() {
            self.floor_mut().level.heaps.remove(&cell);
            self.tell(i, "Está vacío");
        } else {
            self.tell(i, msg);
        }
        self.effect(Fx::Anim { who: ActorId::Hero(i), kind: "operate", cell });
        self.spend(ActorId::Hero(i), TIME_TO_OPEN);
    }

    fn pick_up(&mut self, i: usize) {
        let pos = self.heroes[i].ch.pos;
        let Some(heap) = self.level().heaps.get(&pos) else { return };
        if heap.kind != HeapKind::Heap {
            self.open_heap(i, pos); // cofre, tumba o restos bajo los pies: primero se abren
            return;
        }
        let item = heap.items[0].clone();
        let msg = if item.id == GOLD { format!("Recoges {} de oro", item.quantity) } else { format!("Recoges {}", self.item_name(&item)) };
        if self.collect(i, item).is_err() {
            self.tell(i, "Tu mochila está llena");
            return;
        }
        let heaps = &mut self.floor_mut().level.heaps;
        let heap = heaps.get_mut(&pos).unwrap();
        heap.items.remove(0);
        if heap.items.is_empty() {
            heaps.remove(&pos);
        }
        self.tell(i, &msg);
        self.spend(ActorId::Hero(i), TIME_TO_PICK_UP);
    }

    /// Una acción sobre un ítem pedida desde la UI, en el turno del héroe.
    fn use_item(&mut self, i: usize, slot: Slot, verb: Verb, target: Option<Slot>) {
        let me = ActorId::Hero(i);
        let pos = self.heroes[i].ch.pos;
        let Some(item) = self.heroes[i].item(slot).cloned() else { return };
        match verb {
            Verb::Eat if item.category() == "food" => {
                self.take_one(i, slot);
                let h = &mut self.heroes[i];
                h.hunger = (h.hunger - item.data["energy"].as_f64().unwrap_or(0.0)).max(0.0);
                self.tell(i, &format!("Comes {}", text(&item.data["name"])));
                self.effect(Fx::Anim { who: me, kind: "operate", cell: pos });
                self.spend(me, TIME_TO_EAT);
            }
            Verb::Drink | Verb::Read if matches!(item.category(), "potion" | "scroll") => {
                let need = item.str("target").to_string();
                let target_item = target.and_then(|t| self.heroes[i].item(t).cloned());
                if !need.is_empty() {
                    let ok = target_item.as_ref().is_some_and(|t| t.id != item.id && self.target_ok(&need, t));
                    if !ok {
                        return;
                    }
                }
                // al sacar una unidad de la mochila los índices de detrás pueden correrse: el objetivo se busca después
                let target = target_item.and_then(|t| {
                    let after = if let (Slot::Pack(a), Some(Slot::Pack(b))) = (slot, target) {
                        if b > a && item.quantity == 1 { Some(Slot::Pack(b - 1)) } else { Some(Slot::Pack(b)) }
                    } else {
                        target
                    };
                    after.filter(|s| self.heroes[i].item(*s).is_some_and(|x| x.id == t.id))
                });
                self.take_one(i, slot);
                self.known.insert(item.id.clone());
                self.item_effect(i, item.str("effect"), target);
                let kind = if verb == Verb::Read { "read" } else { "operate" };
                self.effect(Fx::Anim { who: me, kind, cell: pos });
                self.spend(me, TIME_TO_USE);
            }
            Verb::Equip if item.is_equipment() => {
                let Slot::Pack(k) = slot else { return };
                let armor = item.category() == "armor";
                let h = &mut self.heroes[i];
                let old = if armor { &mut h.armor } else { &mut h.weapon };
                if let Some(o) = old.as_mut().filter(|o| o.cursed) {
                    o.cursed_known = true;
                    let o = o.clone();
                    let name = self.item_name(&o);
                    self.tell(i, &format!("No puedes quitarte {name}: está maldito"));
                    return;
                }
                let mut new = h.backpack.remove(k);
                let old = if armor { &mut h.armor } else { &mut h.weapon };
                new.level_known = true; // ponytail: en Java se identifica con el uso, no al equiparlo
                new.cursed_known = true;
                let cursed = new.cursed;
                if let Some(o) = old.replace(new) {
                    h.backpack.push(o);
                }
                let name = self.item_name(&item);
                self.tell(i, &format!("Te equipas {name}{}", if cursed { ": ¡está maldito!" } else { "" }));
                self.spend(me, TIME_TO_EQUIP * if armor { 2.0 } else { 1.0 });
            }
            Verb::Unequip | Verb::Drop if item.cursed && !matches!(slot, Slot::Pack(_)) => {
                let h = &mut self.heroes[i];
                if let Some(it) = if slot == Slot::Weapon { h.weapon.as_mut() } else { h.armor.as_mut() } {
                    it.cursed_known = true;
                }
                let name = self.item_name(&item);
                let what = if verb == Verb::Drop { "soltar" } else { "quitarte" };
                self.tell(i, &format!("No puedes {what} {name}: está maldito"));
            }
            Verb::Unequip if !matches!(slot, Slot::Pack(_)) => {
                if self.heroes[i].backpack.len() >= BACKPACK_SIZE {
                    self.tell(i, "Tu mochila está llena");
                    return;
                }
                let it = self.take_one(i, slot).unwrap();
                self.heroes[i].backpack.push(it);
                self.spend(me, TIME_TO_EQUIP);
            }
            Verb::Drop => {
                let it = match slot {
                    Slot::Pack(k) => self.heroes[i].backpack.remove(k),
                    _ => self.take_one(i, slot).unwrap(),
                };
                self.floor_mut().level.drop(it, pos, HeapKind::Heap);
                self.spend(me, TIME_TO_DROP);
            }
            _ => {}
        }
    }

    /// Qué ítems acepta un pergamino con objetivo (`target` en data/items.json).
    pub fn target_ok(&self, need: &str, item: &Item) -> bool {
        match need {
            "unknown" => !self.is_known(item),
            "upgradable" => item.is_equipment() || item.flag("upgradable"),
            _ => false,
        }
    }

    /// Efectos de pociones y pergaminos, por nombre (data/items.json: `effect`).
    fn item_effect(&mut self, i: usize, effect: &str, target: Option<Slot>) {
        match effect {
            "heal" => {
                let ht = self.heroes[i].ch.ht;
                self.heal(i, ht);
                self.tell(i, "Tus heridas se cierran");
            }
            "strength" => {
                let s = &mut self.heroes[i].abilities[0];
                *s = (*s + 1).min(20);
                let msg = format!("Te sientes más fuerte: FUE {s}");
                self.tell(i, &msg);
            }
            "experience" => {
                let h = &self.heroes[i];
                let missing = h.max_exp() - h.exp;
                self.earn_exp(i, missing);
            }
            "identify" => {
                let Some(slot) = target else { return };
                let id = {
                    let t = self.target_mut(i, slot);
                    t.level_known = true;
                    t.cursed_known = true;
                    t.id.clone()
                };
                self.known.insert(id);
                let name = self.item_name(&self.heroes[i].item(slot).unwrap().clone());
                self.tell(i, &format!("Es {name}"));
            }
            "upgrade" => {
                let Some(slot) = target else { return };
                let t = self.target_mut(i, slot);
                t.level += 1;
                t.cursed = false;
                t.cursed_known = true;
                let name = text(&t.data["name"]);
                self.tell(i, &format!("Tu {name} brilla"));
            }
            "remove_curse" => {
                let h = &mut self.heroes[i];
                let mut any = false;
                for it in h.weapon.iter_mut().chain(h.armor.iter_mut()).chain(h.backpack.iter_mut()) {
                    if it.cursed {
                        it.cursed = false;
                        it.cursed_known = true;
                        any = true;
                    }
                }
                self.tell(i, if any { "Una luz limpia tu equipo" } else { "Sientes una calma extraña" });
            }
            "magic_mapping" => {
                for c in 0..LENGTH {
                    let l = &mut self.floor_mut().level;
                    if l.discoverable[c as usize] {
                        l.mapped[c as usize] = true;
                        if l.secret[c as usize] {
                            l.discover(c);
                        }
                    }
                }
                self.observe();
                self.view_changed = true;
                self.tell(i, "Recuerdas el mapa de este piso");
            }
            "teleport" => {
                let mut rng = self.rng.clone();
                let pos = self.level().random_respawn_cell(&mut rng, |c| self.find_char(c).is_some());
                self.rng = rng;
                if pos == -1 {
                    self.tell(i, "No pasa nada");
                    return;
                }
                self.place_char(ActorId::Hero(i), pos);
                self.heroes[i].action = None;
                self.tell(i, "Apareces en otro lugar");
            }
            _ => {}
        }
    }

    fn target_mut(&mut self, i: usize, slot: Slot) -> &mut Item {
        let h = &mut self.heroes[i];
        match slot {
            Slot::Weapon => h.weapon.as_mut().unwrap(),
            Slot::Armor => h.armor.as_mut().unwrap(),
            Slot::Pack(k) => &mut h.backpack[k],
        }
    }

    /// Nombre de un ítem con mayúscula (títulos de ventanas).
    pub fn item_title(&self, item: &Item) -> String {
        capitalize(&self.item_name(item))
    }
}
