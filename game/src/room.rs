//! Room.java + Graph.java (`game/room.py` del prototipo). Las salas y puertas viven en vectores del nivel y se
//! referencian por índice; `connected` conserva el orden de inserción (como el dict de Python): decide el orden
//! de las tiradas, así que nunca cambiarlo por un HashMap.
use std::collections::VecDeque;

use crate::geom::Rect;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomType {
    Null,
    Standard,
    Entrance,
    Exit,
    BossExit,
    Tunnel,
    Passage,
    Shop,
    Blacksmith,
    Treasury,
    Armory,
    Library,
    Laboratory,
    Vault,
    Traps,
    Storage,
    MagicWell,
    Garden,
    Crypt,
    Statue,
    Pool,
    RatKing,
    WeakFloor,
    Pit,
    Altar,
}

/// Orden original de Room.SPECIALS (las salas especiales llegan después; hoy sólo se usa para excluirlas).
pub const SPECIALS: [RoomType; 14] = [
    RoomType::Armory,
    RoomType::WeakFloor,
    RoomType::MagicWell,
    RoomType::Crypt,
    RoomType::Pool,
    RoomType::Garden,
    RoomType::Library,
    RoomType::Treasury,
    RoomType::Traps,
    RoomType::Storage,
    RoomType::Statue,
    RoomType::Laboratory,
    RoomType::Vault,
    RoomType::Altar,
];

/// El orden importa: `Door::set` sólo "sube" de tipo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoorType {
    Empty,
    Tunnel,
    Regular,
    Unlocked,
    Hidden,
    Barricade,
    Locked,
}

#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub x: i32,
    pub y: i32,
    pub kind: DoorType,
}

impl Door {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y, kind: DoorType::Empty }
    }

    pub fn set(&mut self, t: DoorType) {
        if t > self.kind {
            self.kind = t;
        }
    }
}

/// Las paredes son el borde del rect; el interior va de left+1 a right-1.
#[derive(Clone, Debug)]
pub struct Room {
    pub rect: Rect,
    pub kind: RoomType,
    pub neigbours: Vec<usize>,
    /// (sala, puerta) en orden de conexión. La puerta (índice en `Level::doors`) se crea al pintar.
    pub connected: Vec<(usize, Option<usize>)>,
    pub distance: i64,
    pub price: i64,
}

impl Room {
    pub fn new(rect: Rect) -> Self {
        Self { rect, kind: RoomType::Null, neigbours: Vec::new(), connected: Vec::new(), distance: 0, price: 1 }
    }

    pub fn is_connected(&self, other: usize) -> bool {
        self.connected.iter().any(|(r, _)| *r == other)
    }

    /// Celda al azar del interior, a `m` celdas de la pared.
    pub fn random(&self, rng: &mut Rng, m: i32, w: i32) -> i32 {
        let r = self.rect;
        let x = rng.int_range(r.left + 1 + m, r.right - m);
        let y = rng.int_range(r.top + 1 + m, r.bottom - m);
        x + y * w
    }

    pub fn inside(&self, p: i32, w: i32) -> bool {
        let (x, y) = (p % w, p / w);
        let r = self.rect;
        r.left < x && x < r.right && r.top < y && y < r.bottom
    }

    /// Centro; con lado impar se elige al azar una de las dos celdas. Siempre tira en orden x, y.
    pub fn center(&self, rng: &mut Rng) -> (i32, i32) {
        let r = self.rect;
        let x = (r.left + r.right) / 2 + if r.width() & 1 != 0 { rng.int(2) } else { 0 };
        let y = (r.top + r.bottom) / 2 + if r.height() & 1 != 0 { rng.int(2) } else { 0 };
        (x, y)
    }
}

/// Vecinas = comparten una pared de al menos 3 celdas.
pub fn add_neigbour(rooms: &mut [Room], a: usize, b: usize) {
    let i = rooms[a].rect.intersect(&rooms[b].rect);
    if (i.width() == 0 && i.height() >= 3) || (i.height() == 0 && i.width() >= 3) {
        rooms[a].neigbours.push(b);
        rooms[b].neigbours.push(a);
    }
}

pub fn connect(rooms: &mut [Room], a: usize, b: usize) {
    if !rooms[a].is_connected(b) {
        rooms[a].connected.push((b, None));
        rooms[b].connected.push((a, None));
    }
}

/// Graph.buildDistanceMap: BFS ponderado por `price` desde `focus` sobre las vecinas (no las conectadas).
pub fn build_distance_map(rooms: &mut [Room], focus: usize) {
    for r in rooms.iter_mut() {
        r.distance = i32::MAX as i64;
    }
    rooms[focus].distance = 0;
    let mut queue = VecDeque::from([focus]);
    while let Some(node) = queue.pop_front() {
        let (d, p) = (rooms[node].distance, rooms[node].price);
        for k in 0..rooms[node].neigbours.len() {
            let edge = rooms[node].neigbours[k];
            if rooms[edge].distance > d + p {
                rooms[edge].distance = d + p;
                queue.push_back(edge);
            }
        }
    }
}

/// Graph.buildPath: baja por el mapa de distancias de `start` a `end`. None si no hay camino.
pub fn build_path(rooms: &[Room], start: usize, end: usize) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut room = start;
    while room != end {
        let mut best = rooms[room].distance;
        let mut next = None;
        for &edge in &rooms[room].neigbours {
            if rooms[edge].distance < best {
                best = rooms[edge].distance;
                next = Some(edge);
            }
        }
        room = next?;
        path.push(room);
    }
    Some(path)
}
