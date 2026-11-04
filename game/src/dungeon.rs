//! Dungeon.java (`game/dungeon.py`): estado de una partida (seed, pisos, héroes, mobs, planificadores, mensajes).
//! Varios héroes, cada uno en su piso (`Hero::depth`). `depth` es el piso "en contexto": el que se está procesando, o el
//! del héroe local (`hero`, el que maneja la UI) fuera de `process()`.
//! El combate 5e (iniciativa, ataque d20 vs CA, críticos, daño) vive aquí porque toca a atacante y objetivo.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::actor::{ActorId, Scheduler, CALM_ROUNDS, TICK};
use crate::char::{adjacent, distance, Char, TRAIL_MAX};
use crate::dice;
use crate::fov::MAX_DISTANCE;
use crate::geom::NEIGHBOURS9;
use crate::hero::Hero;
use crate::items::{make_gold, Generator, HeapKind, POTION_LOOKS, SCROLL_LOOKS};
use crate::level::Level;
use crate::mobs::{pick_mob, Mob, State};
use crate::pathfinder;
use crate::registry;
use crate::rng::Rng;
use crate::terrain as t;

/// Dungeon.posNeeded / souNeeded: [piso límite, cantidad acumulada esperada, ...].
const STRENGTH_QUOTA: [i32; 10] = [4, 2, 9, 4, 14, 6, 19, 8, 24, 9];
const UPGRADE_QUOTA: [i32; 10] = [5, 3, 10, 6, 15, 9, 20, 12, 25, 13];

/// El piso 5 (Goo) queda fuera.
pub const MAX_DEPTH: i32 = 4;
/// Level.TIME_TO_RESPAWN.
pub const TIME_TO_RESPAWN: f64 = 50.0;
const MAX_MESSAGES: usize = 50;
const MAX_DICE_EVENTS: usize = 20;
const MAX_FX: usize = 40;

/// Una tirada que el héroe local ve (la UI la anima). ok = salvación superada / fallada; None en ataques y daño.
#[derive(Clone, Debug)]
pub struct DiceEvent {
    pub label: String,
    pub value: i32,
    pub lo: i32,
    pub hi: i32,
    pub ok: Option<bool>,
}

/// Color del texto flotante (CharSprite.DEFAULT / POSITIVE / NEGATIVE / WARNING / NEUTRAL).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Default,
    Pos,
    Neg,
    Warn,
    Neutral,
}

/// Efectos para el render del piso local, que los consume.
#[derive(Clone, Debug)]
pub enum Fx {
    /// Ataque / operar / leer: el sprite se gira hacia la celda y reproduce la animación.
    Anim { who: ActorId, kind: &'static str, cell: i32 },
    /// Texto flotante sobre el personaje (CharSprite.showStatus).
    Status { who: ActorId, text: String, tone: Tone },
    /// Un mob murió: ya salió de la partida, por eso viaja con sus datos (queda el cadáver).
    Die { who: ActorId, cell: i32, stats: Value },
    /// Casilla revisada al buscar (CheckedCell).
    Checked(i32),
    /// Pasto alto pisado.
    Leaves(i32),
}

/// Lo que espera a que la UI muestre el dado. Sin rng nuevo al resolver: el orden de tiradas no cambia.
#[derive(Clone, Debug)]
pub enum Pending {
    Attack { attacker: ActorId, target: ActorId, hit: bool, dmg: i32, detail: String, simple: String },
    DeathSave { hero: usize, nat: i32 },
    Stabilize { hero: usize, ally: usize, ok: bool },
}

pub struct Floor {
    pub level: Level,
    pub mobs: Vec<Mob>,
    pub scheduler: Scheduler,
}

pub struct Dungeon {
    pub seed: i64,
    /// rng de juego (combate, IA); cada nivel genera con el suyo.
    pub rng: Rng,
    pub depth: i32,
    pub messages: Vec<String>,
    pub pending: Vec<Pending>,
    /// Tiradas que el héroe local ve; la UI las consume.
    pub dice_events: Vec<DiceEvent>,
    /// Efectos del piso local; el render los consume.
    pub fx: Vec<Fx>,
    /// Opción de desarrollador: el log muestra el cálculo completo de cada tirada.
    pub verbose_rolls: bool,
    /// Pisos visitados: al volver siguen sus mobs y lo explorado.
    pub floors: BTreeMap<i32, Floor>,
    pub heroes: Vec<Hero>,
    /// El héroe local (el que maneja la UI).
    pub hero: usize,
    /// La vista del piso local cambió (la UI vuelve a subir la niebla y lo baja).
    pub view_changed: bool,
    /// Pociones y pergaminos identificados, y con qué apariencia (nombre, índice en items.png) se ve cada uno.
    pub known: BTreeSet<String>,
    pub looks: BTreeMap<String, (&'static str, i32)>,
    generator: Generator,
    potions_of_strength: i32,
    scrolls_of_upgrade: i32,
    next_mob: u32,
    classes: Map<String, Value>,
    pub(crate) items: Map<String, Value>,
    mobs_data: Map<String, Value>,
    bestiary: Map<String, Value>,
}

fn load(name: &str) -> Map<String, Value> {
    registry::load(name).unwrap_or_else(|e| panic!("{e}"))
}

impl Dungeon {
    pub fn new(seed: i64, hero_class: &str) -> Self {
        let mut d = Self {
            seed,
            rng: Rng::new(seed),
            depth: 0,
            messages: Vec::new(),
            pending: Vec::new(),
            dice_events: Vec::new(),
            fx: Vec::new(),
            verbose_rolls: false,
            floors: BTreeMap::new(),
            heroes: Vec::new(),
            hero: 0,
            view_changed: true,
            known: BTreeSet::new(),
            looks: BTreeMap::new(),
            generator: Generator::default(),
            potions_of_strength: 0,
            scrolls_of_upgrade: 0,
            next_mob: 0,
            classes: load("classes"),
            items: load("items"),
            mobs_data: load("mobs"),
            bestiary: load("bestiary"),
        };
        // apariencias sin identificar: lo primero que gasta el rng de la partida (como el prototipo)
        // ponytail: si un mod agrega más de 12 pociones o pergaminos faltan apariencias
        for cat in ["potion", "scroll"] {
            let mut order: Vec<usize> = (0..12).collect();
            d.rng.shuffle(&mut order);
            let ids: Vec<String> = d.items.iter().filter(|(_, v)| v["category"] == cat).map(|(k, _)| k.clone()).collect();
            for (id, k) in ids.into_iter().zip(order) {
                let look = if cat == "potion" { (POTION_LOOKS[k][0], 56 + k as i32) } else { SCROLL_LOOKS[k] };
                d.looks.insert(id, look);
            }
        }
        d.add_hero(hero_class);
        d
    }

    /// Suma un héroe a la partida, en la entrada del piso 1.
    pub fn add_hero(&mut self, hero_class: &str) -> usize {
        let hero = Hero::new(&self.classes, &self.items, hero_class);
        if let Some(known) = hero.cls.get("known").and_then(Value::as_array) {
            self.known.extend(known.iter().filter_map(Value::as_str).map(str::to_string));
        }
        self.heroes.push(hero);
        let i = self.heroes.len() - 1;
        self.go_to(i, 1);
        i
    }

    // --- acceso ---
    pub fn floor(&self) -> &Floor {
        &self.floors[&self.depth]
    }

    pub fn floor_mut(&mut self) -> &mut Floor {
        self.floors.get_mut(&self.depth).expect("piso en contexto")
    }

    pub fn level(&self) -> &Level {
        &self.floor().level
    }

    pub fn local(&self) -> &Hero {
        &self.heroes[self.hero]
    }

    pub fn mob(&self, id: u32) -> Option<&Mob> {
        self.floor().mobs.iter().find(|m| m.id == id)
    }

    pub fn mob_mut(&mut self, id: u32) -> Option<&mut Mob> {
        self.floor_mut().mobs.iter_mut().find(|m| m.id == id)
    }

    pub(crate) fn remove_mob(&mut self, id: u32) -> Option<Mob> {
        let f = self.floor_mut();
        f.scheduler.remove(ActorId::Mob(id));
        let i = f.mobs.iter().position(|m| m.id == id)?;
        Some(f.mobs.remove(i))
    }

    pub fn ch(&self, a: ActorId) -> Option<&Char> {
        match a {
            ActorId::Hero(i) => Some(&self.heroes[i].ch),
            ActorId::Mob(id) => self.mob(id).map(|m| &m.ch),
            _ => None,
        }
    }

    pub fn ch_mut(&mut self, a: ActorId) -> Option<&mut Char> {
        match a {
            ActorId::Hero(i) => Some(&mut self.heroes[i].ch),
            ActorId::Mob(id) => self.mob_mut(id).map(|m| &mut m.ch),
            _ => None,
        }
    }

    pub fn alive(&self, a: ActorId) -> bool {
        match a {
            ActorId::Hero(i) => self.heroes[i].is_alive(),
            ActorId::Mob(id) => self.mob(id).is_some_and(|m| m.ch.hp > 0),
            _ => false,
        }
    }

    fn downed(&self, a: ActorId) -> bool {
        matches!(a, ActorId::Hero(i) if self.heroes[i].is_downed())
    }

    /// Héroes del piso en contexto (vivos o no).
    pub fn here(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.heroes.len()).filter(|&i| self.heroes[i].depth == self.depth)
    }

    /// Personajes del piso en contexto con su celda: héroes y después mobs.
    pub fn chars(&self) -> Vec<(ActorId, i32)> {
        let heroes = self.here().map(|i| (ActorId::Hero(i), self.heroes[i].ch.pos));
        heroes.chain(self.floor().mobs.iter().map(|m| (ActorId::Mob(m.id), m.ch.pos))).collect()
    }

    pub fn find_char(&self, cell: i32) -> Option<ActorId> {
        self.chars().into_iter().find(|&(a, p)| p == cell && self.alive(a)).map(|(a, _)| a)
    }

    /// Lo que pasa en `cell` se muestra (piso del héroe local y a la vista): espera al dado (pending).
    pub fn sees(&self, cell: i32) -> bool {
        self.depth == self.local().depth && self.level().visible[cell as usize]
    }

    /// Una tirada para la animación de dados (sólo la del piso local).
    pub(crate) fn roll_event(&mut self, label: &str, value: i32, lo: i32, hi: i32, ok: Option<bool>) {
        if self.depth != self.local().depth {
            return;
        }
        self.dice_events.push(DiceEvent { label: label.to_string(), value, lo, hi, ok });
        let extra = self.dice_events.len().saturating_sub(MAX_DICE_EVENTS);
        self.dice_events.drain(..extra);
    }

    /// Un efecto para el render (sólo se dibuja el piso del héroe local).
    pub(crate) fn effect(&mut self, fx: Fx) {
        if self.depth != self.local().depth {
            return;
        }
        self.fx.push(fx);
        let extra = self.fx.len().saturating_sub(MAX_FX);
        self.fx.drain(..extra);
    }

    /// Texto flotante sobre el personaje, si se ve.
    pub(crate) fn status(&mut self, who: ActorId, text: &str, tone: Tone) {
        if self.ch(who).is_some_and(|c| c.pos >= 0 && self.level().visible[c.pos as usize]) {
            self.effect(Fx::Status { who, text: text.to_string(), tone });
        }
    }

    pub fn log(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
        let extra = self.messages.len().saturating_sub(MAX_MESSAGES);
        self.messages.drain(..extra);
    }

    // --- pisos ---
    pub(crate) fn descend(&mut self, i: usize) {
        if self.heroes[i].depth >= MAX_DEPTH {
            self.log("Fin de la demo: el piso 5 no está incluido");
            return;
        }
        self.go_to(i, self.heroes[i].depth + 1);
    }

    pub(crate) fn ascend(&mut self, i: usize) {
        if self.heroes[i].depth <= 1 {
            self.log("No puedes salir de la mazmorra");
            return;
        }
        self.go_to(i, self.heroes[i].depth - 1);
    }

    /// Cambia de piso a un héroe (InterlevelScene). Un piso sin héroes vivos no se procesa (su tiempo no avanza).
    /// El héroe se lleva lo suyo (la regeneración).
    fn go_to(&mut self, i: usize, depth: i32) {
        let old = self.heroes[i].depth;
        let going_down = depth > old;
        self.heroes[i].depth = depth;
        if old != 0 {
            self.depth = old;
            let s = &mut self.floor_mut().scheduler;
            s.remove(ActorId::Hero(i));
            s.remove(ActorId::Regen(i));
            s.remove(ActorId::Hunger(i));
            self.observe(); // el piso que deja ya no ve lo que veía este héroe
        }
        let h = &mut self.heroes[i];
        h.ch.pos = -1; // que free_near no lo cuente como ocupando la escalera
        h.action = None;
        h.visible_enemies.clear();
        h.ch.trail.clear(); // cambia de piso: aparece en la escalera, no camina hasta ahí
        self.pending.clear();
        if self.floors.contains_key(&depth) {
            self.depth = depth;
            let cell = if going_down { self.level().entrance } else { self.level().exit };
            self.heroes[i].ch.pos = self.free_near(cell);
            self.floor_mut().scheduler.add(ActorId::Hero(i), 0.0);
        } else {
            let level = Level::new(self.seed, depth);
            self.heroes[i].ch.pos = level.entrance;
            self.floors.insert(depth, Floor { level, mobs: Vec::new(), scheduler: Scheduler::default() });
            self.depth = depth;
            self.generator.reset();
            self.floor_mut().scheduler.add(ActorId::Hero(i), 0.0); // primero: en empates actúa antes que nadie
            self.create_mobs();
            self.create_items();
            self.floor_mut().scheduler.add(ActorId::Respawner, 0.0);
        }
        let s = &mut self.floor_mut().scheduler;
        s.add(ActorId::Regen(i), 0.0);
        s.add(ActorId::Hunger(i), 0.0);
        self.observe();
        let msg =
            if self.heroes.len() == 1 { format!("Piso {depth}") } else { format!("{}: piso {depth}", self.heroes[i].ch.name) };
        self.log(&msg);
    }

    /// `cell`, o una vecina libre y transitable (otro héroe o un mob ya está en la escalera).
    fn free_near(&self, cell: i32) -> i32 {
        NEIGHBOURS9
            .iter()
            .map(|n| cell + n)
            .find(|&c| self.level().passable[c as usize] && self.find_char(c).is_none())
            .unwrap_or(cell) // ponytail: todo ocupado, comparte celda hasta que alguien se mueva
    }

    /// Dungeon.observe: la vista de cada héroe del piso; `level.visible` es la unión (lo que se dibuja).
    pub fn observe(&mut self) {
        let hs: Vec<usize> = self.here().collect();
        let pos: Vec<i32> = hs.iter().map(|&i| self.heroes[i].ch.pos).collect();
        let views = self.floor_mut().level.observe(&pos, MAX_DISTANCE);
        for (i, v) in hs.into_iter().zip(views) {
            self.heroes[i].fov = v;
        }
        if self.depth == self.local().depth {
            self.view_changed = true;
        }
    }

    fn n_mobs(depth: i32, rng: &mut Rng) -> usize {
        (2 + depth % 5 + rng.int(3)) as usize
    }

    /// Celda libre de una sala STANDARD fuera de la vista; -1 si no hay.
    fn respawn_cell(&self, rng: &mut Rng) -> i32 {
        self.level().random_respawn_cell(rng, |c| self.find_char(c).is_some())
    }

    pub(crate) fn random_destination(&mut self) -> i32 {
        self.floors[&self.depth].level.random_destination(&mut self.rng)
    }

    /// RegularLevel.createMobs, con el rng del nivel (mismo nivel = mismos mobs).
    fn create_mobs(&mut self) {
        let mut rng = self.level().rng.clone();
        for _ in 0..Self::n_mobs(self.depth, &mut rng) {
            let key = pick_mob(&self.bestiary, &mut rng, self.depth);
            let mut pos = -1;
            while pos == -1 {
                pos = self.respawn_cell(&mut rng);
            }
            self.add_mob(&key, pos, State::Sleeping);
        }
        self.floor_mut().level.rng = rng;
    }

    /// Dungeon.chance: garantiza ~N ítems clave antes de cierto piso.
    fn quota(&self, rng: &mut Rng, quota: &[i32], number: i32) -> bool {
        for q in quota.chunks(2) {
            if self.depth <= q[0] {
                return rng.float() < (q[1] - number) as f64 / (q[0] - self.depth + 1) as f64;
            }
        }
        false
    }

    /// Level.create (ítems obligatorios) + RegularLevel.createItems + tumbas, con el rng del nivel.
    /// ponytail: el mimic (caso 5 en pisos > 1) sale como cofre normal hasta que exista
    fn create_items(&mut self) {
        let mut rng = self.level().rng.clone();
        let (depth, cur_str) = (self.depth, self.local().score("STR"));
        let mut to_spawn = vec![self.generator.random_of("food", &self.items, &mut rng, depth, cur_str)];
        if self.quota(&mut rng, &STRENGTH_QUOTA, self.potions_of_strength) {
            to_spawn.push(self.new_item("pdri:potion_strength"));
            self.potions_of_strength += 1;
        }
        if self.quota(&mut rng, &UPGRADE_QUOTA, self.scrolls_of_upgrade) {
            to_spawn.push(self.new_item("pdri:scroll_upgrade"));
            self.scrolls_of_upgrade += 1;
        }
        let mut n = 3;
        while rng.float() < 0.4 {
            n += 1;
        }
        for _ in 0..n {
            let t = rng.int(20);
            let kind = if t == 0 { HeapKind::Skeleton } else if t <= 5 { HeapKind::Chest } else { HeapKind::Heap };
            let item = self.generator.random(&self.items, &mut rng, depth, cur_str);
            let cell = self.level().random_drop_cell(&mut rng);
            self.floor_mut().level.drop(item, cell, kind);
        }
        for item in to_spawn {
            let cell = self.level().random_drop_cell(&mut rng);
            self.floor_mut().level.drop(item, cell, HeapKind::Heap);
        }
        for (pos, has_item) in self.level().tombs.clone() {
            let item = if has_item {
                self.generator.random(&self.items, &mut rng, depth, cur_str)
            } else {
                make_gold(&self.items, &mut rng, depth)
            };
            self.floor_mut().level.drop(item, pos, HeapKind::Tomb);
        }
        self.floor_mut().level.rng = rng;
    }

    fn add_mob(&mut self, key: &str, pos: i32, state: State) {
        let stats = self.mobs_data.get(key).unwrap_or_else(|| panic!("mob desconocido: {key}")).clone();
        let id = self.next_mob;
        self.next_mob += 1;
        let f = self.floor_mut();
        f.mobs.push(Mob::new(id, key, stats, pos, state));
        f.scheduler.add(ActorId::Mob(id), 0.0);
    }

    /// Level.respawner: cada 50 de tiempo, si faltan mobs, aparece uno (despierto) fuera de la vista.
    fn respawn_act(&mut self) {
        if self.floor().mobs.len() < Self::n_mobs(self.depth, &mut self.rng) {
            let mut rng = self.rng.clone();
            let pos = self.respawn_cell(&mut rng);
            self.rng = rng;
            if pos != -1 {
                let key = pick_mob(&self.bestiary, &mut self.rng, self.depth);
                self.add_mob(&key, pos, State::Wandering);
            }
        }
        self.spend(ActorId::Respawner, TIME_TO_RESPAWN);
    }

    /// El héroe vivo más cercano que el mob ve; si no ve a ninguno, el más cercano (para seguir su rastro). Prefiere
    /// a los que están en pie. En empate, el primero (determinista).
    pub(crate) fn choose_enemy(&self, fov: &[bool], pos: i32) -> Option<usize> {
        let hs: Vec<usize> = self.here().filter(|&h| self.heroes[h].is_alive()).collect();
        let seen: Vec<usize> = hs.iter().copied().filter(|&h| fov[self.heroes[h].ch.pos as usize]).collect();
        let pool = if seen.is_empty() { hs } else { seen };
        pool.into_iter().min_by_key(|&h| (self.heroes[h].is_downed(), distance(pos, self.heroes[h].ch.pos)))
    }

    /// Dungeon.findPath: un paso hacia `to` evitando a las criaturas que se ven.
    pub(crate) fn find_path(&self, flying: bool, from: i32, to: i32, passable: &[bool], visible: &[bool]) -> i32 {
        let level = self.level();
        if adjacent(from, to) {
            let t = to as usize;
            return if self.find_char(to).is_none() && (passable[t] || level.avoid[t]) { to } else { -1 };
        }
        let mut p: Vec<bool> =
            if flying { passable.iter().zip(&level.avoid).map(|(a, b)| *a || *b).collect() } else { passable.to_vec() };
        for (_, c) in self.chars() {
            if c >= 0 && visible[c as usize] {
                p[c as usize] = false;
            }
        }
        pathfinder::get_step(from, to, &p)
    }

    // --- movimiento ---
    /// Las puertas se abren al entrar y se cierran al salir (una sola criatura por celda).
    pub(crate) fn move_char(&mut self, a: ActorId, step: i32) {
        let old = self.ch(a).unwrap().pos;
        let level = &mut self.floor_mut().level;
        let s = step as usize;
        debug_assert!(adjacent(old, step) && (level.passable[s] || level.avoid[s]), "paso inválido {old} -> {step}");
        let before = (level.map[old as usize], level.map[step as usize]);
        level.leave(old);
        level.enter(step);
        let changed = (level.map[old as usize], level.map[step as usize]) != before;
        let c = self.ch_mut(a).unwrap();
        c.pos = step;
        c.trail.push(step);
        if c.trail.len() > TRAIL_MAX {
            c.trail.remove(0);
        }
        c.stepped = true; // el próximo spend() es movimiento (combate 5e)
        let flying = c.flying;
        if changed {
            self.observe(); // se abrió o cerró una puerta: la vista cambia
        }
        if !flying {
            self.press(step);
        }
    }

    /// Lleva a alguien a una celda lejana (teletransporte): sin rastro de pasos, el sprite aparece ahí.
    pub(crate) fn place_char(&mut self, a: ActorId, cell: i32) {
        let old = self.ch(a).unwrap().pos;
        let level = &mut self.floor_mut().level;
        level.leave(old);
        level.enter(cell);
        let c = self.ch_mut(a).unwrap();
        c.pos = cell;
        c.trail.clear();
        let flying = c.flying;
        self.observe();
        if !flying {
            self.press(cell);
        }
    }

    /// Level.press: el pasto alto se pisa. ponytail: sin trampas todavía (R5)
    fn press(&mut self, cell: i32) {
        if self.level().map[cell as usize] == t::HIGH_GRASS {
            self.floor_mut().level.set(cell, t::GRASS);
            if self.level().visible[cell as usize] {
                self.effect(Fx::Leaves(cell));
            }
            self.observe(); // el pasto alto tapaba la vista
        }
    }

    // --- tiempo ---
    /// Fuera de combate, tiempo de PD. En combate: lo que sigue a un paso gasta 1 casilla de movimiento; cualquier otra
    /// cosa gasta la acción y termina el turno.
    pub(crate) fn spend(&mut self, a: ActorId, t: f64) {
        let in_combat = self.floor().scheduler.in_combat(a);
        let Some(c) = self.ch_mut(a) else {
            self.floor_mut().scheduler.spend(a, t);
            return;
        };
        let stepped = std::mem::take(&mut c.stepped);
        if !in_combat {
            self.floor_mut().scheduler.spend(a, t);
        } else if c.no_moves {
        } else if stepped {
            c.moves -= 1;
        } else {
            c.has_action = false;
        }
    }

    /// En combate, sin movimiento restante no camina y su turno termina (la acción en curso sigue el próximo).
    pub(crate) fn can_step(&mut self, a: ActorId) -> bool {
        let in_combat = self.floor().scheduler.in_combat(a);
        let c = self.ch_mut(a).unwrap();
        if in_combat && c.moves <= 0 {
            c.no_moves = true;
            return false;
        }
        true
    }

    // --- planificador ---
    /// Corre los turnos de cada piso con héroes vivos. Se frena mientras haya algo visto sin resolver (lo resuelve
    /// `resolve_pending`). Al final queda en contexto el piso del héroe local.
    pub fn process(&mut self) {
        let mut depths = Vec::new();
        for h in &self.heroes {
            if h.is_alive() && !depths.contains(&h.depth) {
                depths.push(h.depth);
            }
        }
        for d in depths {
            self.depth = d;
            self.process_floor();
        }
        self.depth = self.local().depth;
    }

    fn process_floor(&mut self) {
        let depth = self.depth;
        loop {
            if !self.pending.is_empty() || self.depth != depth || !self.here().any(|h| self.heroes[h].is_alive()) {
                return;
            }
            let s = &self.floor().scheduler;
            if s.actors.is_empty() {
                return;
            }
            if s.order.is_some() {
                if !self.combat_step() {
                    return;
                }
                continue;
            }
            let Some(cur) = s.current() else { return };
            if self.waiting(cur) {
                return;
            }
            let now = s.time(cur);
            self.floor_mut().scheduler.now = now;
            self.act(cur);
            // si el héroe cambió de piso, el combate se decide igual en el piso que dejó (como el prototipo)
            let ctx = std::mem::replace(&mut self.depth, depth);
            if self.any_fighting() {
                self.start_combat();
            }
            self.depth = ctx;
        }
    }

    /// El héroe del piso local al que le toca actuar (el planificador espera su acción), o None.
    pub fn blocking(&self) -> Option<usize> {
        match self.floor().scheduler.current() {
            Some(ActorId::Hero(i)) => Some(i),
            _ => None,
        }
    }

    fn act(&mut self, a: ActorId) {
        match a {
            ActorId::Hero(i) => self.hero_act(i),
            ActorId::Mob(id) => self.mob_act(id),
            ActorId::Regen(i) => self.regen_act(i),
            ActorId::Hunger(i) => self.hunger_act(i),
            ActorId::Respawner => self.respawn_act(),
        }
    }

    fn waiting(&self, a: ActorId) -> bool {
        matches!(a, ActorId::Hero(i) if self.heroes[i].waiting())
    }

    fn any_fighting(&self) -> bool {
        self.floor().scheduler.actors.iter().any(|&(a, _)| matches!(a, ActorId::Mob(id) if self.mob(id).is_some_and(Mob::fighting)))
    }

    // --- combate 5e ---
    fn initiative_bonus(&self, a: ActorId) -> i32 {
        match a {
            ActorId::Hero(i) => self.heroes[i].initiative_bonus(),
            ActorId::Mob(id) => self.mob(id).map_or(0, Mob::initiative_bonus),
            _ => 0,
        }
    }

    /// d20 + bono (DES). Queda en `init` para ordenar a los que se suman después.
    fn roll_initiative(&mut self, a: ActorId) {
        let (nat, _) = dice::d20(&mut self.rng, 0);
        let b = self.initiative_bonus(a);
        let init = nat + b;
        let c = self.ch_mut(a).unwrap();
        c.init = init;
        if let ActorId::Hero(_) = a {
            let label = format!("Iniciativa {}", c.name);
            self.roll_event(&label, init, 1 + b, 20 + b, None);
        }
    }

    fn start_turn(&mut self, a: ActorId) {
        let ft = match a {
            ActorId::Hero(i) => self.heroes[i].speed_ft(),
            _ => self.ch(a).map_or(30, |c| c.base_speed_ft),
        };
        if let Some(c) = self.ch_mut(a) {
            c.start_turn(ft);
        }
    }

    fn by_initiative(&self, mut chars: Vec<ActorId>) -> Vec<ActorId> {
        chars.sort_by_key(|&a| -self.ch(a).map_or(0, |c| c.init)); // estable: en empate, el agregado primero
        chars
    }

    /// Todos los personajes del piso tiran iniciativa y se ordenan.
    fn start_combat(&mut self) {
        let chars: Vec<ActorId> =
            self.floor().scheduler.actors.iter().map(|e| e.0).filter(|&a| a.in_rounds() && self.alive(a)).collect();
        for &a in &chars {
            self.roll_initiative(a);
        }
        let order = self.by_initiative(chars);
        let seen: Vec<String> = order
            .iter()
            .filter_map(|&a| self.ch(a))
            .filter(|c| self.level().visible[c.pos as usize])
            .map(|c| format!("{} {}", c.name, c.init))
            .collect();
        self.log(&format!("¡Combate! Iniciativa: {}", seen.join(", ")));
        let first = order[0];
        let s = &mut self.floor_mut().scheduler;
        (s.turn, s.round, s.calm) = (0, 1, 0);
        s.order = Some(order);
        self.start_turn(first);
    }

    /// Un paso de la ronda. false = hay que esperar (al héroe al que le toca).
    fn combat_step(&mut self) -> bool {
        let s = &self.floor().scheduler;
        let order = s.order.as_ref().unwrap();
        if s.turn >= order.len() {
            self.end_round();
            return true;
        }
        let c = order[s.turn];
        if s.contains(c) && !self.ch(c).is_none_or(Char::turn_over) {
            if self.waiting(c) {
                return false;
            }
            self.act(c);
            return true;
        }
        let s = &mut self.floor_mut().scheduler;
        s.turn += 1;
        if let Some(&next) = s.order.as_ref().unwrap().get(s.turn) {
            self.start_turn(next);
        }
        true
    }

    /// Fin de ronda: estados y timers avanzan un TICK (1 ronda = 1 turno de PD); entran los personajes nuevos con su
    /// iniciativa; CALM_ROUNDS rondas sin nadie peleando = fin del combate.
    fn end_round(&mut self) {
        let end = self.floor().scheduler.now + TICK;
        loop {
            let s = &self.floor().scheduler;
            let mut due: Option<(ActorId, f64)> = None;
            for &(a, t) in &s.actors {
                if !a.in_rounds() && t <= end && due.is_none_or(|(_, dt)| t < dt) {
                    due = Some((a, t));
                }
            }
            let Some((a, t)) = due else { break };
            let s = &mut self.floor_mut().scheduler;
            s.now = s.now.max(t);
            self.act(a);
        }
        let s = &self.floor().scheduler;
        let old: Vec<ActorId> = s.order.as_ref().unwrap().iter().copied().filter(|&a| s.contains(a)).collect();
        let new: Vec<ActorId> =
            s.actors.iter().map(|e| e.0).filter(|&a| a.in_rounds() && !old.contains(&a) && self.alive(a)).collect();
        for &a in &new {
            self.roll_initiative(a);
        }
        // ponytail: el que entra a mitad de combate actúa desde la ronda siguiente (5e: desde su iniciativa)
        let order = self.by_initiative([old, new].concat());
        let fighting = self.any_fighting();
        let s = &mut self.floor_mut().scheduler;
        s.now = end;
        s.calm = if fighting { 0 } else { s.calm + 1 };
        if s.calm >= CALM_ROUNDS || order.is_empty() {
            for &a in &order {
                s.set_time(a, end); // vuelven al tiempo de PD todos parejos
            }
            s.order = None;
            if !order.is_empty() {
                self.log("Fin del combate");
            }
            return;
        }
        s.turn = 0;
        s.round += 1;
        let first = order[0];
        s.order = Some(order);
        self.start_turn(first);
    }

    fn armor_class(&self, a: ActorId) -> i32 {
        match a {
            ActorId::Hero(i) => self.heroes[i].armor_class(),
            ActorId::Mob(id) => self.mob(id).map_or(10, Mob::armor_class),
            _ => 10,
        }
    }

    fn attack_bonus(&self, a: ActorId) -> i32 {
        match a {
            ActorId::Hero(i) => self.heroes[i].attack_bonus(),
            ActorId::Mob(id) => self.mob(id).map_or(0, Mob::attack_bonus),
            _ => 0,
        }
    }

    /// 1 = ventaja, -1 = desventaja, 0 = normal. Ventaja contra quien no lo vio venir o está caído.
    fn advantage(&self, att: ActorId, target: ActorId) -> i32 {
        let unaware = matches!(target, ActorId::Mob(id) if self.mob(id).is_some_and(|m| !m.enemy_seen));
        let mut adv = if unaware || self.downed(target) { 1 } else { 0 };
        if matches!(att, ActorId::Hero(i) if self.heroes[i].disadvantaged()) {
            adv -= 1; // ventaja y desventaja juntas se anulan
        }
        adv.clamp(-1, 1)
    }

    /// (daño, texto para el log, evento de dado para la animación). crit = dados dobles.
    fn roll_damage(&mut self, a: ActorId, crit: bool) -> (i32, String, Option<DiceEvent>) {
        match a {
            ActorId::Hero(i) => {
                let h = &self.heroes[i];
                let m = h.attack_mod();
                let Some(w) = &h.weapon else { return (1 + m, format!("1{m:+}"), None) }; // sin armas: 1 + FUE
                let (total, text, rolled, sides) = dice::roll(&mut self.rng, &crate::items::weapon_dice(w), crit);
                let n = rolled.len() as i32;
                let value = rolled.iter().sum();
                let ev = DiceEvent { label: format!("{n}d{sides}"), value, lo: n, hi: n * sides, ok: None };
                (total + m, format!("{text}{m:+}"), Some(ev))
            }
            ActorId::Mob(id) => {
                let m = self.floors.get(&self.depth).and_then(|f| f.mobs.iter().find(|m| m.id == id));
                let Some(m) = m else { return (0, String::new(), None) };
                let (dmg, text, (lo, hi)) = m.roll_damage(&mut self.rng, crit);
                let k = if crit { 2 } else { 1 };
                let ev = DiceEvent { label: format!("{lo}-{hi}"), value: dmg, lo: lo * k, hi: hi * k, ok: None };
                (dmg, text, Some(ev))
            }
            _ => (0, String::new(), None),
        }
    }

    /// Tirada de ataque: d20 + bono contra la CA. 20 natural = crítico (dados dobles), 1 natural = falla.
    pub(crate) fn attack(&mut self, att: ActorId, enemy: ActorId) -> bool {
        let adv = self.advantage(att, enemy);
        let (nat, roll_text) = dice::d20(&mut self.rng, adv);
        let (bonus, ac) = (self.attack_bonus(att), self.armor_class(enemy));
        let hit = nat == 20 || (nat != 1 && nat + bonus >= ac);
        let (a, e) = (self.ch(att).unwrap(), self.ch(enemy).unwrap());
        let (apos, epos, aname, ename) = (a.pos, e.pos, a.name.clone(), e.name.clone());
        let line = format!("{aname}: d20 {roll_text}{bonus:+}={} vs CA {ac}", nat + bonus);
        let seen = self.sees(apos) || self.sees(epos);
        self.effect(Fx::Anim { who: att, kind: "attack", cell: epos });
        if seen {
            self.roll_event("d20", nat, 1, 20, None);
        }
        if !hit {
            if seen {
                let p = Pending::Attack {
                    attacker: att,
                    target: enemy,
                    hit: false,
                    dmg: 0,
                    detail: format!("{line}, falla"),
                    simple: format!("{aname} falla"),
                };
                self.pending.push(p);
            }
            return false;
        }
        let crit = nat == 20 || (self.downed(enemy) && adjacent(apos, epos)); // 5e: caído y al lado = crítico
        let (dmg, dmg_text, event) = self.roll_damage(att, crit);
        let dmg = dmg.max(0);
        if !seen {
            self.damage(enemy, dmg, Some(att));
            return true;
        }
        if let Some(e) = event {
            self.roll_event(&e.label, e.value, e.lo, e.hi, None);
        }
        let tag = if crit { "CRITICO " } else { "" };
        let p = Pending::Attack {
            attacker: att,
            target: enemy,
            hit: true,
            dmg,
            detail: format!("{line}, {tag}{dmg_text}={dmg}"),
            simple: format!("{aname} golpea a {ename}: {dmg}{}", if crit { " (critico)" } else { "" }),
        };
        self.pending.push(p);
        true
    }

    pub fn damage(&mut self, target: ActorId, dmg: i32, src: Option<ActorId>) {
        match target {
            ActorId::Hero(i) => self.hero_damage(i, dmg, src),
            ActorId::Mob(id) => self.mob_damage(id, dmg, src),
            _ => {}
        }
    }

    // --- lo que espera al dado ---
    /// Agrega a pending. Fuera del piso local no hay dado que mirar: se aplica ya.
    pub(crate) fn defer(&mut self, p: Pending) {
        self.pending.push(p);
        if self.depth != self.local().depth {
            self.resolve_pending();
        }
    }

    /// Aplica lo que la UI ya mostró. Sin rng nuevo: el orden de tiradas no cambia (determinista).
    pub fn resolve_pending(&mut self) {
        for p in std::mem::take(&mut self.pending) {
            match p {
                Pending::Attack { attacker, target, hit, dmg, detail, simple } => {
                    self.log(if self.verbose_rolls { &detail } else { &simple });
                    if hit {
                        self.damage(target, dmg, Some(attacker));
                    } else {
                        self.status(target, "esquiva", Tone::Neutral);
                    }
                }
                Pending::DeathSave { hero, nat } => self.apply_death_save(hero, nat),
                Pending::Stabilize { hero, ally, ok } => self.apply_stabilize(hero, ally, ok),
            }
        }
    }
}

