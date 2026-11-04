//! actors/Char.java (`game/char.py`): lo común de héroes y mobs. El combate 5e (que necesita la partida) está en
//! dungeon.rs; lo propio de cada uno en hero.rs y mobs.rs.
use crate::geom::WIDTH;

/// Pasos que se guardan para el render (más que el movimiento de un turno de combate).
pub const TRAIL_MAX: usize = 16;

/// Level.distance: distancia de Chebyshev (las diagonales cuentan 1).
pub fn distance(a: i32, b: i32) -> i32 {
    ((a % WIDTH) - (b % WIDTH)).abs().max(((a / WIDTH) - (b / WIDTH)).abs())
}

pub fn adjacent(a: i32, b: i32) -> bool {
    distance(a, b) == 1
}

pub struct Char {
    pub pos: i32,
    pub name: String,
    pub ht: i32,
    pub hp: i32,
    /// Velocidad de PD (fuera de combate: tiempo por paso = 1 / speed).
    pub base_speed: f64,
    /// Velocidad 5e en pies (en combate: casillas de 5 ft por turno).
    pub base_speed_ft: i32,
    // turno de combate 5e: movimiento restante, acción disponible, iniciativa
    pub moves: i32,
    pub has_action: bool,
    pub init: i32,
    /// El próximo spend() es un paso (gasta movimiento, no la acción).
    pub stepped: bool,
    /// Se quedó sin movimiento: su turno termina.
    pub no_moves: bool,
    pub flying: bool,
    /// Celdas por las que pasó desde que el render miró: el sprite las recorre una por una (sin cortar esquinas
    /// cuando hizo varios pasos de una vez). Las consume el render; tope TRAIL_MAX.
    pub trail: Vec<i32>,
    /// Estados alterados (buffs.rs).
    pub buffs: Vec<crate::buffs::Buff>,
}

impl Char {
    pub fn new(pos: i32, name: String) -> Self {
        Self {
            pos,
            name,
            ht: 1,
            hp: 1,
            base_speed: 1.0,
            base_speed_ft: 30,
            moves: 0,
            has_action: true,
            init: 0,
            stepped: false,
            no_moves: false,
            flying: false,
            trail: Vec::new(),
            buffs: Vec::new(),
        }
    }

    /// Turno de combate 5e: movimiento (speed_ft / 5 casillas) + 1 acción.
    pub fn start_turn(&mut self, speed_ft: i32) {
        self.moves = speed_ft / 5;
        self.has_action = true;
        self.stepped = false;
        self.no_moves = false;
    }

    pub fn turn_over(&self) -> bool {
        !self.has_action || self.no_moves
    }
}
