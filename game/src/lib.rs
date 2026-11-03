//! Lógica del juego, sin render ni ventana (corre headless: host y herramientas).
//! Port de `game/` del prototipo (rama `python-prototipe`).

pub mod geom;
pub mod level;
pub mod painters;
pub mod patch;
pub mod registry;
pub mod rng;
pub mod room;
pub mod terrain;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
