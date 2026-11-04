//! actors/hero/Hero.java + HeroClass.java + buffs/Regeneration.java (`game/hero.py`), con el combate de D&D 5e.
//! Las clases son datos (`data/classes.json`). R3: moverse, atacar cuerpo a cuerpo, esperar, buscar, escaleras,
//! estabilizar a un aliado caído, salvaciones contra la muerte, regeneración, XP y niveles.
//! ponytail: sin hambre, inventario, rasgos de clase ni conjuros; llegan en R5
use serde_json::{Map, Value};

use crate::actor::ActorId;
use crate::char::{adjacent, Char};
use crate::dice;
use crate::dungeon::{Dungeon, Pending};
use crate::geom::{LENGTH, NEIGHBOURS9};
use crate::items::{armor_ac, Item};
use crate::registry::text;

pub const TIME_TO_REST: f64 = 1.0;
pub const TIME_TO_SEARCH: f64 = 2.0;
pub const REGENERATION_DELAY: f64 = 10.0;

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
    pub action: Option<Action>,
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
}

impl Hero {
    pub fn new(classes: &Map<String, Value>, items: &Map<String, Value>, class_key: &str) -> Self {
        let cls = classes.get(class_key).unwrap_or_else(|| panic!("clase desconocida: {class_key}")).clone();
        let item = |k: &str| cls.get(k).and_then(Value::as_str).and_then(|id| Item::new(items, id));
        let mut h = Self {
            ch: Char::new(-1, text(&cls["name"])),
            class_key: class_key.to_string(),
            abilities: dice::DEFAULT_SCORES,
            lvl: 1,
            exp: 0,
            awareness: 0.0,
            weapon: item("start_weapon"),
            armor: item("start_armor"),
            action: None,
            animating: false,
            visible_enemies: Vec::new(),
            depth: 0,
            fov: vec![false; LENGTH as usize],
            dead: false,
            saves: [0, 0],
            stable: false,
            cls,
        };
        h.ch.base_speed_ft = h.cls.get("speed_ft").and_then(Value::as_i64).unwrap_or(30) as i32;
        // PV de nivel 1 en 5e: dado de golpe al máximo + modificador de CON
        h.ch.ht = (h.hit_die() + h.ability_mod("CON")).max(1);
        h.ch.hp = h.ch.ht;
        h.update_awareness();
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

    pub fn proficient_weapon(&self) -> bool {
        self.weapon.as_ref().is_none_or(|w| self.class_has("weapon_prof", &w.id) || self.class_has("weapon_prof", w.str("kind")))
    }

    pub fn proficient_armor(&self) -> bool {
        self.armor.as_ref().is_none_or(|a| a.str("kind") == "clothing" || self.class_has("armor_prof", a.str("kind")))
    }

    /// Puntos de FUE que faltan para usar bien el ítem (0 si alcanza).
    pub fn str_missing(&self, item: Option<&Item>) -> i32 {
        item.map_or(0, |i| (i.int("STR") - self.score("STR")).max(0))
    }

    // --- combate 5e ---
    pub fn attack_mod(&self) -> i32 {
        if self.weapon.as_ref().is_some_and(|w| w.flag("finesse")) {
            return self.ability_mod("STR").max(self.ability_mod("DEX"));
        }
        self.ability_mod("STR")
    }

    pub fn armor_class(&self) -> i32 {
        armor_ac(self.armor.as_ref(), self.ability_mod("DEX"))
    }

    pub fn attack_bonus(&self) -> i32 {
        let prof = if self.proficient_weapon() { self.proficiency() } else { 0 };
        self.attack_mod() + prof - self.str_missing(self.weapon.as_ref())
    }

    /// Desventaja propia: arma o armadura demasiado pesada, o armadura sin competencia.
    pub fn disadvantaged(&self) -> bool {
        self.str_missing(self.weapon.as_ref()) > 0 || self.str_missing(self.armor.as_ref()) > 0 || !self.proficient_armor()
    }

    pub fn attack_delay(&self) -> f64 {
        self.weapon.as_ref().and_then(|w| w.data.get("dly")).and_then(Value::as_f64).unwrap_or(1.0)
    }

    /// Del original: armadura muy pesada = más lento fuera de combate.
    pub fn speed(&self) -> f64 {
        let enc = self.armor.as_ref().map_or(0, |a| a.int("STR") - self.score("STR"));
        if enc > 0 { self.ch.base_speed / 1.3f64.powi(enc) } else { self.ch.base_speed }
    }

    /// 5e: armadura pesada sin la FUE que pide = -10 ft.
    pub fn speed_ft(&self) -> i32 {
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

    pub fn max_exp(&self) -> i32 {
        5 + self.lvl * 5
    }

    fn update_awareness(&mut self) {
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
        self.action.is_none() && !self.is_downed() // caído: los turnos pasan solos
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
            Some(a) if seen && a != ActorId::Hero(i) => Action::Attack(a),
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
            Action::Attack(e) => {
                let target = if self.alive(e) { self.ch(e).map(|c| c.pos) } else { None };
                match target {
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
            Action::Help(j) => {
                let ally = self.heroes[j].ch.pos;
                if adjacent(pos, ally) {
                    self.heroes[i].action = None;
                    self.stabilize(i, j);
                } else if !(self.heroes[j].is_downed() && self.hero_get_closer(i, ally)) {
                    self.heroes[i].action = None;
                }
            }
            Action::Descend(c) | Action::Ascend(c) if pos == c => {
                self.heroes[i].action = None;
                if matches!(action, Action::Descend(_)) { self.descend(i) } else { self.ascend(i) }
            }
            Action::Move(c) | Action::Descend(c) | Action::Ascend(c) => {
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
            self.floor().mobs.iter().filter(|m| fov[m.ch.pos as usize]).map(|m| ActorId::Mob(m.id)).collect();
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
            if self.level().secret[p as usize] && (intentional || self.rng.float() < self.heroes[i].awareness) {
                self.floor_mut().level.discover(p);
                found = true;
            }
        }
        if found {
            self.log("Encuentras algo oculto");
            self.observe();
        } else if intentional {
            self.log("Buscas, pero no encuentras nada");
        }
    }

    pub(crate) fn earn_exp(&mut self, i: usize, exp: i32) {
        self.heroes[i].exp += exp;
        while self.heroes[i].exp >= self.heroes[i].max_exp() {
            let h = &mut self.heroes[i];
            h.exp -= h.max_exp();
            h.lvl += 1;
            // 5e: promedio del dado de golpe + CON (mínimo 1) por nivel
            let gain = (h.hit_die() / 2 + 1 + h.ability_mod("CON")).max(1);
            h.ch.ht += gain;
            if h.ch.hp > 0 {
                h.ch.hp += gain; // caído (XP repartida) no se levanta por subir de nivel
            }
            if h.lvl < 10 {
                h.update_awareness();
            }
            let msg = format!("Subes a nivel {}: +{gain} PV, competencia +{}", h.lvl, h.proficiency());
            self.log(&msg);
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
        let src_name = src.and_then(|s| self.ch(s)).map_or("daño".to_string(), |c| c.name.clone());
        let h = &mut self.heroes[i];
        if h.ch.hp <= 0 {
            if dmg > 0 {
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
                if dies {
                    self.hero_die(i, src);
                }
            }
            return;
        }
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
        }
    }

    /// Sale del planificador con lo suyo: el piso sigue para los demás héroes; sin héroes vivos, nada avanza.
    /// ponytail: el muerto queda de espectador; volver con un personaje nuevo llega con la red (R6)
    pub(crate) fn hero_die(&mut self, i: usize, src: Option<ActorId>) {
        let cause = src.and_then(|s| self.ch(s)).map_or(String::new(), |c| format!(" ({})", c.name));
        let h = &mut self.heroes[i];
        h.dead = true;
        h.ch.hp = 0;
        let s = &mut self.floor_mut().scheduler;
        s.remove(ActorId::Hero(i));
        s.remove(ActorId::Regen(i));
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
        }
    }

    /// Medicina (5e): d20 + SAB contra CD 10 sobre un aliado caído al lado. Una acción.
    /// ponytail: sin competencia en habilidades todavía (Medicina sólo suma SAB)
    fn stabilize(&mut self, i: usize, ally: usize) {
        if !self.heroes[ally].is_downed() || self.heroes[ally].stable {
            return;
        }
        let (nat, _) = dice::d20(&mut self.rng, 0);
        let ok = nat + self.heroes[i].ability_mod("WIS") >= 10;
        self.defer(Pending::Stabilize { hero: i, ally, ok });
        self.spend(ActorId::Hero(i), TIME_TO_REST);
    }

    pub(crate) fn apply_stabilize(&mut self, i: usize, ally: usize, ok: bool) {
        let (me, other) = (self.heroes[i].ch.name.clone(), self.heroes[ally].ch.name.clone());
        if ok && self.heroes[ally].is_downed() {
            self.heroes[ally].stable = true;
            self.log(&format!("{me} estabiliza a {other}"));
        } else if !ok {
            self.log(&format!("{me} no logra estabilizar a {other}"));
        }
    }

    /// buffs/Regeneration.java: +1 PV cada 10 de tiempo. Caído, sólo si está estable (despierta).
    pub(crate) fn regen_act(&mut self, i: usize) {
        let h = &self.heroes[i];
        if h.is_alive() && h.ch.hp < h.ch.ht && (h.ch.hp > 0 || h.stable) {
            self.heal(i, 1);
        }
        self.spend(ActorId::Regen(i), REGENERATION_DELAY);
    }
}
