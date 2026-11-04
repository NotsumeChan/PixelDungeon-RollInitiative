//! actors/Actor.java (`game/actor.py`): planificador por tiempo, y rondas de combate 5e por iniciativa.
//! Los actores son ids: el planificador guarda tiempos y el orden; qué hace cada uno vive en `Dungeon` (dungeon.rs).

pub const TICK: f64 = 1.0;
/// Rondas seguidas sin ningún mob cazando a la vista para salir del combate.
pub const CALM_ROUNDS: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActorId {
    Hero(usize),
    Mob(u32),
    /// Regeneración del héroe (buffs/Regeneration.java): viaja con él entre pisos.
    Regen(usize),
    /// Level.respawner: uno por piso.
    Respawner,
}

impl ActorId {
    /// true = en combate actúa por iniciativa (personajes); false = sigue el tiempo (estados, timers).
    pub fn in_rounds(self) -> bool {
        matches!(self, ActorId::Hero(_) | ActorId::Mob(_))
    }
}

/// Fuera de combate (Actor.process): actúa el de menor tiempo; en empate, el agregado primero.
/// En combate (`order`), rondas 5e: cada personaje, por iniciativa, hace su turno (movimiento + 1 acción); al final
/// de la ronda los estados y timers avanzan un TICK. Uno por piso.
#[derive(Default)]
pub struct Scheduler {
    /// (actor, tiempo), en el orden en que se agregaron (desempate).
    pub actors: Vec<(ActorId, f64)>,
    pub now: f64,
    /// Combate: personajes por iniciativa (None = fuera de combate).
    pub order: Option<Vec<ActorId>>,
    /// Índice en `order` de a quién le toca.
    pub turn: usize,
    pub round: u32,
    /// Rondas seguidas sin nadie peleando.
    pub calm: u32,
}

impl Scheduler {
    pub fn add(&mut self, a: ActorId, delay: f64) {
        self.actors.push((a, self.now + delay));
    }

    pub fn remove(&mut self, a: ActorId) {
        self.actors.retain(|e| e.0 != a);
    }

    pub fn contains(&self, a: ActorId) -> bool {
        self.actors.iter().any(|e| e.0 == a)
    }

    pub fn time(&self, a: ActorId) -> f64 {
        self.actors.iter().find(|e| e.0 == a).map_or(0.0, |e| e.1)
    }

    pub fn set_time(&mut self, a: ActorId, t: f64) {
        if let Some(e) = self.actors.iter_mut().find(|e| e.0 == a) {
            e.1 = t;
        }
    }

    pub fn spend(&mut self, a: ActorId, t: f64) {
        if let Some(e) = self.actors.iter_mut().find(|e| e.0 == a) {
            e.1 += t;
        }
    }

    /// A quién le toca: en combate, al de la iniciativa; si no, al de menor tiempo (el primero en empate).
    pub fn current(&self) -> Option<ActorId> {
        if let Some(o) = &self.order {
            return o.get(self.turn).copied();
        }
        let mut best: Option<(ActorId, f64)> = None;
        for &(a, t) in &self.actors {
            if best.is_none_or(|(_, bt)| t < bt) {
                best = Some((a, t));
            }
        }
        best.map(|b| b.0)
    }

    pub fn in_combat(&self, a: ActorId) -> bool {
        self.order.as_ref().is_some_and(|o| o.contains(&a))
    }
}
