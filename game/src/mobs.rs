//! Mobs de las Alcantarillas: actors/mobs/*.java + Bestiary.java (`game/mobs.py`). Los mobs son datos
//! (`data/mobs.json`, `data/bestiary.json`) + este Mob genérico. IA SLEEPING / WANDERING / HUNTING (Mob.java).
//! ponytail: sin habilidades especiales (Swarm se divide, Skeleton explota, Thief roba) ni FLEEING; llegan en R5
use serde_json::{Map, Value};

use crate::actor::{ActorId, TICK};
use crate::char::{adjacent, distance, Char};
use crate::dungeon::{Dungeon, Fx, Tone};
use crate::items::{make_gold, HeapKind, GOLD};
use crate::fov::{self, MAX_DISTANCE};
use crate::geom::WIDTH;
use crate::registry::text;
use crate::rng::Rng;

pub const TIME_TO_WAKE_UP: f64 = 1.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Sleeping,
    Wandering,
    Hunting,
}

impl State {
    /// AiState.status (ventana de examinar).
    pub fn name(self) -> &'static str {
        match self {
            State::Sleeping => "Está durmiendo",
            State::Wandering => "Está deambulando",
            State::Hunting => "Te está cazando",
        }
    }
}

fn int(s: &Value, key: &str) -> i32 {
    s.get(key).and_then(Value::as_i64).unwrap_or(0) as i32
}

/// Conversión PD -> 5e: la evasión y la reducción de daño del original se vuelven CA (base 8).
/// Rat 10, Gnoll 11, Crab 12, Swarm 10, Skeleton 15, Thief 15.
pub fn mob_ac(s: &Value) -> i32 {
    8 + (int(s, "defense") + int(s, "dr")).div_euclid(2)
}

/// Conversión PD -> 5e: precisión del original / 2. Rat +4, Gnoll +5, el resto +6.
pub fn mob_attack_bonus(s: &Value) -> i32 {
    int(s, "attack").div_euclid(2)
}

/// Bestiary.java: un mob del piso según los pesos de `data/bestiary.json`.
pub fn pick_mob(bestiary: &Map<String, Value>, rng: &mut Rng, depth: i32) -> String {
    let table = bestiary[&depth.to_string()].as_array().expect("bestiario");
    let weights: Vec<f64> = table.iter().map(|e| e[1].as_f64().unwrap_or(0.0)).collect();
    table[rng.chances(&weights)][0].as_str().unwrap_or("").to_string()
}

pub struct Mob {
    pub id: u32,
    pub ch: Char,
    pub key: String,
    pub stats: Value,
    pub state: State,
    pub target: i32,
    pub enemy: Option<usize>,
    pub enemy_seen: bool,
    pub alerted: bool,
    pub fov: Vec<bool>,
}

impl Mob {
    pub fn new(id: u32, key: &str, stats: Value, pos: i32, state: State) -> Self {
        let mut ch = Char::new(pos, text(&stats["name"]));
        ch.ht = int(&stats, "HT");
        ch.hp = ch.ht;
        ch.base_speed = stats.get("speed").and_then(Value::as_f64).unwrap_or(1.0);
        ch.base_speed_ft = stats.get("speed_ft").and_then(Value::as_i64).unwrap_or(30) as i32;
        ch.flying = stats.get("flying").and_then(Value::as_bool).unwrap_or(false);
        Self { id, ch, key: key.to_string(), stats, state, target: -1, enemy: None, enemy_seen: false, alerted: false, fov: Vec::new() }
    }

    pub fn armor_class(&self) -> i32 {
        mob_ac(&self.stats)
    }

    pub fn attack_bonus(&self) -> i32 {
        mob_attack_bonus(&self.stats)
    }

    /// Mod de DES 5e del mob (`initiative` en data/mobs.json).
    pub fn initiative_bonus(&self) -> i32 {
        int(&self.stats, "initiative")
    }

    /// Mete al piso en combate: caza a un héroe que ve.
    pub fn fighting(&self) -> bool {
        self.state == State::Hunting && self.enemy_seen && self.ch.hp > 0
    }

    pub fn exp_for(&self, hero_lvl: i32) -> i32 {
        if hero_lvl <= int(&self.stats, "max_lvl") { int(&self.stats, "exp") } else { 0 }
    }

    /// El daño conserva el rango del original (NormalIntRange); con crítico se tira dos veces.
    /// (daño, texto para el log, rango del original).
    pub fn roll_damage(&self, rng: &mut Rng, crit: bool) -> (i32, String, (i32, i32)) {
        let d = &self.stats["damage"];
        let (lo, hi) = (d[0].as_i64().unwrap_or(1) as i32, d[1].as_i64().unwrap_or(1) as i32);
        let rolls: Vec<i32> = (0..if crit { 2 } else { 1 }).map(|_| rng.normal_int_range(lo, hi)).collect();
        let list: Vec<String> = rolls.iter().map(i32::to_string).collect();
        (rolls.iter().sum(), format!("{lo}-{hi}({})", list.join("+")), (lo, hi))
    }
}

impl Dungeon {
    /// Mob.act + AiState.
    pub(crate) fn mob_act(&mut self, id: u32) {
        let Some(m) = self.mob(id) else { return };
        let pos = m.ch.pos;
        let fov = fov::cast_shadow(pos % WIDTH, pos / WIDTH, &self.level().los_blocking, MAX_DISTANCE);
        let enemy = self.choose_enemy(&fov, pos);
        let in_fov = enemy.is_some_and(|e| self.heroes[e].is_alive() && fov[self.heroes[e].ch.pos as usize]);
        let m = self.mob_mut(id).unwrap();
        m.fov = fov;
        m.enemy = enemy;
        let just_alerted = std::mem::take(&mut m.alerted);
        let state = m.state;
        match state {
            State::Sleeping => self.mob_sleeping(id, in_fov),
            State::Wandering => self.mob_wandering(id, in_fov, just_alerted),
            State::Hunting => self.mob_hunting(id, in_fov),
        }
    }

    /// Distancia al enemigo + su sigilo (+2 si vuela): cuanto más, menos probable que el mob lo note.
    fn notice(&self, id: u32) -> i32 {
        let m = self.mob(id).unwrap();
        let h = &self.heroes[m.enemy.unwrap()];
        distance(m.ch.pos, h.ch.pos) + h.stealth() + if h.ch.flying { 2 } else { 0 }
    }

    fn mob_sleeping(&mut self, id: u32, in_fov: bool) {
        if in_fov && self.rng.int(self.notice(id)) == 0 {
            let target = self.heroes[self.mob(id).unwrap().enemy.unwrap()].ch.pos;
            let m = self.mob_mut(id).unwrap();
            m.enemy_seen = true;
            m.state = State::Hunting;
            m.target = target;
            self.spend(ActorId::Mob(id), TIME_TO_WAKE_UP);
        } else {
            self.mob_mut(id).unwrap().enemy_seen = false;
            self.spend(ActorId::Mob(id), TICK);
        }
    }

    fn mob_wandering(&mut self, id: u32, in_fov: bool, just_alerted: bool) {
        let noticed = in_fov && (just_alerted || {
            let m = self.mob(id).unwrap();
            let h = &self.heroes[m.enemy.unwrap()];
            let n = distance(m.ch.pos, h.ch.pos) / 2 + h.stealth();
            self.rng.int(n) == 0
        });
        if noticed {
            // sin gastar tiempo: el planificador lo vuelve a elegir y actúa ya cazando (igual que Java)
            let target = self.heroes[self.mob(id).unwrap().enemy.unwrap()].ch.pos;
            let m = self.mob_mut(id).unwrap();
            m.enemy_seen = true;
            m.state = State::Hunting;
            m.target = target;
            return;
        }
        let m = self.mob_mut(id).unwrap();
        m.enemy_seen = false;
        let (target, speed) = (m.target, m.ch.base_speed);
        if target != -1 && self.mob_get_closer(id, target) {
            self.spend(ActorId::Mob(id), 1.0 / speed);
        } else {
            let dest = self.random_destination();
            self.mob_mut(id).unwrap().target = dest;
            self.spend(ActorId::Mob(id), TICK);
        }
    }

    fn mob_hunting(&mut self, id: u32, in_fov: bool) {
        let m = self.mob_mut(id).unwrap();
        m.enemy_seen = in_fov;
        let (pos, enemy) = (m.ch.pos, m.enemy);
        let epos = enemy.map(|e| self.heroes[e].ch.pos);
        if in_fov && adjacent(pos, epos.unwrap()) {
            self.attack(ActorId::Mob(id), ActorId::Hero(enemy.unwrap()));
            self.spend(ActorId::Mob(id), 1.0); // attackDelay
            return;
        }
        let m = self.mob_mut(id).unwrap();
        if in_fov {
            m.target = epos.unwrap();
        }
        let (target, speed) = (m.target, m.ch.base_speed);
        if target != -1 && self.mob_get_closer(id, target) {
            self.spend(ActorId::Mob(id), 1.0 / speed);
        } else {
            self.spend(ActorId::Mob(id), TICK);
            let dest = self.random_destination();
            let m = self.mob_mut(id).unwrap();
            m.state = State::Wandering;
            m.target = dest;
        }
    }

    fn mob_get_closer(&mut self, id: u32, target: i32) -> bool {
        if !self.can_step(ActorId::Mob(id)) {
            return true; // sin movimiento en este turno de combate: el spend() siguiente no gasta nada
        }
        let m = self.mob(id).unwrap();
        let step = self.find_path(m.ch.flying, m.ch.pos, target, &self.level().passable, &m.fov);
        if step == -1 {
            return false;
        }
        self.move_char(ActorId::Mob(id), step);
        true
    }

    /// Mob.damage: lo despierta y lo alerta.
    pub(crate) fn mob_damage(&mut self, id: u32, dmg: i32, src: Option<ActorId>) {
        let Some(m) = self.mob_mut(id) else { return };
        if m.state == State::Sleeping {
            m.state = State::Wandering;
        }
        m.alerted = true;
        if m.ch.hp <= 0 {
            return;
        }
        if dmg > 0 {
            self.status(ActorId::Mob(id), &dmg.to_string(), Tone::Neg); // número de daño
        }
        let m = self.mob_mut(id).unwrap();
        m.ch.hp -= dmg;
        if m.ch.hp <= 0 {
            self.mob_die(id, src);
        }
    }

    /// XP repartida entre los héroes vivos del piso (redondeo hacia arriba) y +1 al que dio el último golpe.
    fn mob_die(&mut self, id: u32, src: Option<ActorId>) {
        let Some(m) = self.remove_mob(id) else { return };
        self.effect(Fx::Die { who: ActorId::Mob(id), cell: m.ch.pos, stats: m.stats.clone() }); // el render deja el cadáver
        if self.level().visible[m.ch.pos as usize] {
            self.log(&format!("{} muere", m.ch.name));
        }
        let alive: Vec<usize> = self.here().filter(|&h| self.heroes[h].is_alive()).collect();
        let n = alive.len() as i32;
        for h in alive {
            let mut share = (m.exp_for(self.heroes[h].lvl) + n - 1) / n;
            if share > 0 && src == Some(ActorId::Hero(h)) && n > 1 {
                share += 1;
            }
            self.earn_exp(h, share);
        }
        // Mob.dropLoot: sólo si los héroes no los superan por mucho (cuenta el de menor nivel)
        if let Some(loot) = m.stats.get("loot") {
            let low = self.here().map(|h| self.heroes[h].lvl).min().unwrap_or(0);
            if low <= int(&m.stats, "max_lvl") + 2 && self.rng.float() < loot[1].as_f64().unwrap_or(0.0) {
                let id = loot[0].as_str().unwrap_or(GOLD);
                let item = if id == GOLD { make_gold(&self.items, &mut self.rng, self.depth) } else { self.new_item(id) };
                self.floor_mut().level.drop(item, m.ch.pos, HeapKind::Heap);
            }
        }
    }
}
