//! Lógica del juego, sin render ni ventana (corre headless: host y herramientas).
//! Port de `game/` del prototipo (rama `python-prototipe`).

pub mod actor;
pub mod ballistica;
pub mod buffs;
pub mod char;
pub mod dice;
pub mod dungeon;
pub mod fov;
pub mod geom;
pub mod hero;
pub mod items;
pub mod level;
pub mod mobs;
pub mod painters;
pub mod pathfinder;
pub mod patch;
pub mod registry;
pub mod rng;
pub mod spells;
pub mod room;
pub mod terrain;
pub mod traps;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
