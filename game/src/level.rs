//! Level.java (create) + RegularLevel.java + SewerLevel.java (`game/level.py`): generación de las Alcantarillas,
//! pisos 1-4. Port 1:1 del prototipo: cada tirada del rng va en el mismo orden, así la misma seed da el mismo piso.
use crate::fov;
use crate::geom::{Rect, HEIGHT, LENGTH, NEIGHBOURS4, NEIGHBOURS8, NEIGHBOURS9, WIDTH};
use crate::painters::{self, fill, fill_room};
use crate::patch;
use crate::rng::Rng;
use crate::room::{self, Door, DoorType, Room, RoomType, SPECIALS};
use crate::terrain::{self as t, Feeling};

pub const MIN_ROOM_SIZE: i32 = 7;
pub const MAX_ROOM_SIZE: i32 = 9;

pub fn boss_level(depth: i32) -> bool {
    matches!(depth, 5 | 10 | 15 | 20 | 25)
}

/// Seed del rng de un piso: estable entre procesos y peers.
pub fn level_seed(seed: i64, depth: i32) -> i64 {
    seed * 1000 + depth as i64
}

pub struct Level {
    pub depth: i32,
    pub boss: bool,
    pub boss_next: bool,
    pub rng: Rng,
    pub feeling: Feeling,
    pub entrance: i32,
    pub exit: i32,
    pub secret_doors: i32,
    /// Tumbas del cementerio: (celda, tiene ítem). Se llenan con los ítems (R5).
    pub tombs: Vec<(i32, bool)>,
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    pub room_entrance: usize,
    pub room_exit: usize,
    pub map: Vec<u8>,
    pub passable: Vec<bool>,
    pub los_blocking: Vec<bool>,
    pub flamable: Vec<bool>,
    pub secret: Vec<bool>,
    pub solid: Vec<bool>,
    pub avoid: Vec<bool>,
    pub water: Vec<bool>,
    pub pit: Vec<bool>,
    /// La celda toca algo que no es pared y no es todo abismo (para buscar secretos).
    pub discoverable: Vec<bool>,
    /// Lo que se ve ahora (unión de las vistas de los héroes), lo que se vio alguna vez, lo revelado por magia.
    pub visible: Vec<bool>,
    pub visited: Vec<bool>,
    pub mapped: Vec<bool>,
}

const SECRET_TRAPS: [u8; 8] = [
    t::SECRET_TOXIC_TRAP,
    t::SECRET_FIRE_TRAP,
    t::SECRET_PARALYTIC_TRAP,
    t::SECRET_POISON_TRAP,
    t::SECRET_ALARM_TRAP,
    t::SECRET_LIGHTNING_TRAP,
    t::SECRET_GRIPPING_TRAP,
    t::SECRET_SUMMONING_TRAP,
];

impl Level {
    pub fn new(seed: i64, depth: i32) -> Self {
        let n = LENGTH as usize;
        let mut level = Self {
            depth,
            boss: boss_level(depth),
            boss_next: boss_level(depth + 1),
            rng: Rng::new(level_seed(seed, depth)),
            feeling: Feeling::None,
            entrance: -1,
            exit: -1,
            secret_doors: 0,
            tombs: Vec::new(),
            rooms: Vec::new(),
            doors: Vec::new(),
            room_entrance: 0,
            room_exit: 0,
            map: vec![t::WALL; n],
            passable: Vec::new(),
            los_blocking: Vec::new(),
            flamable: Vec::new(),
            secret: Vec::new(),
            solid: Vec::new(),
            avoid: Vec::new(),
            water: Vec::new(),
            pit: Vec::new(),
            discoverable: Vec::new(),
            visible: vec![false; n],
            visited: vec![false; n],
            mapped: vec![false; n],
        };
        level.create();
        level
    }

    // --- Level.create ---

    fn create(&mut self) {
        // ponytail: Java agrega aquí ítems obligatorios que consumen rng; llegan con los ítems (R5)
        if self.depth > 1 {
            let c = self.rng.int(10);
            if c == 0 && !self.boss_next {
                self.feeling = Feeling::Chasm;
            } else if c == 1 {
                self.feeling = Feeling::Water;
            } else if c == 2 {
                self.feeling = Feeling::Grass;
            }
        }
        loop {
            let base = if self.feeling == Feeling::Chasm { t::CHASM } else { t::WALL };
            self.map = vec![base; LENGTH as usize];
            if self.build() {
                break;
            }
        }
        self.decorate();
        self.build_flag_maps();
        self.clean_walls();
    }

    // --- RegularLevel.build ---

    fn build(&mut self) -> bool {
        self.tombs.clear();
        if !self.init_rooms() {
            return false;
        }
        let n = self.rooms.len();
        let min_distance = (n as f64).sqrt() as i64;
        let big = |r: &Room| r.rect.width() >= 4 && r.rect.height() >= 4;
        let mut retry = 0;
        let (entrance, exit) = loop {
            let entrance = loop {
                let i = self.rng.pick(n).unwrap();
                if big(&self.rooms[i]) {
                    break i;
                }
            };
            let exit = loop {
                let i = self.rng.pick(n).unwrap();
                if i != entrance && big(&self.rooms[i]) {
                    break i;
                }
            };
            room::build_distance_map(&mut self.rooms, exit);
            let distance = self.rooms[entrance].distance;
            if retry > 10 {
                return false;
            }
            retry += 1;
            if distance >= min_distance {
                break (entrance, exit);
            }
        };
        self.rooms[entrance].kind = RoomType::Entrance;
        self.rooms[exit].kind = RoomType::Exit;
        self.room_entrance = entrance;
        self.room_exit = exit;

        // Dos caminos entrada -> salida; el segundo se desvía porque el primero se encarece.
        let mut connected = vec![entrance];
        for second in [false, true] {
            room::build_distance_map(&mut self.rooms, exit);
            let Some(path) = room::build_path(&self.rooms, entrance, exit) else { return false };
            let mut cur = entrance;
            for &next in &path {
                room::connect(&mut self.rooms, cur, next);
                cur = next;
                if !connected.contains(&cur) {
                    connected.push(cur);
                }
            }
            if !second {
                let d = self.rooms[entrance].distance;
                for &r in &path {
                    self.rooms[r].price = d;
                }
            }
        }

        let n_connected = (n as f64 * self.rng.float_range(0.5, 0.7)) as usize;
        while connected.len() < n_connected {
            let cr = connected[self.rng.pick(connected.len()).unwrap()];
            let other = self.rng.pick(self.rooms[cr].neigbours.len()).map(|i| self.rooms[cr].neigbours[i]);
            if let Some(o) = other
                && !connected.contains(&o)
            {
                room::connect(&mut self.rooms, cr, o);
                connected.push(o);
            }
        }

        // Tienda: no aparece en pisos 1-4. ponytail: sin salas especiales todavía (lista vacía, como el prototipo)
        self.assign_room_type();
        self.paint();
        self.paint_water();
        self.paint_grass();
        self.place_traps();
        true
    }

    fn init_rooms(&mut self) -> bool {
        self.rooms.clear();
        self.doors.clear();
        self.split(Rect::new(0, 0, WIDTH - 1, HEIGHT - 1));
        if self.rooms.len() < 8 {
            return false;
        }
        for i in 0..self.rooms.len() - 1 {
            for j in i + 1..self.rooms.len() {
                room::add_neigbour(&mut self.rooms, i, j);
            }
        }
        true
    }

    /// Partición recursiva del mapa en salas de 7..9 (aprox).
    fn split(&mut self, rect: Rect) {
        let (w, h) = (rect.width(), rect.height());
        let vertical = |s: &mut Self, rect: Rect| {
            let vw = s.rng.int_range(rect.left + 3, rect.right - 3);
            s.split(Rect::new(rect.left, rect.top, vw, rect.bottom));
            s.split(Rect::new(vw, rect.top, rect.right, rect.bottom));
        };
        let horizontal = |s: &mut Self, rect: Rect| {
            let vh = s.rng.int_range(rect.top + 3, rect.bottom - 3);
            s.split(Rect::new(rect.left, rect.top, rect.right, vh));
            s.split(Rect::new(rect.left, vh, rect.right, rect.bottom));
        };
        if w > MAX_ROOM_SIZE && h < MIN_ROOM_SIZE {
            return vertical(self, rect);
        }
        if h > MAX_ROOM_SIZE && w < MIN_ROOM_SIZE {
            return horizontal(self, rect);
        }
        // Java: Math.random() y división ENTERA 49 / square. El random() se tira siempre que se llega aquí.
        let r = self.rng.random();
        let q = (MIN_ROOM_SIZE * MIN_ROOM_SIZE / rect.square()) as f64;
        if (r <= q && w <= MAX_ROOM_SIZE && h <= MAX_ROOM_SIZE) || w < MIN_ROOM_SIZE || h < MIN_ROOM_SIZE {
            self.rooms.push(Room::new(rect));
        } else if self.rng.float() < (w - 2) as f64 / (w + h - 4) as f64 {
            vertical(self, rect);
        } else {
            horizontal(self, rect);
        }
    }

    fn assign_room_type(&mut self) {
        for i in 0..self.rooms.len() {
            if self.rooms[i].kind == RoomType::Null && self.rooms[i].connected.len() == 1 && self.rng.int(2) == 0 {
                let r = &self.rooms[i];
                let candidates: Vec<usize> = r
                    .neigbours
                    .iter()
                    .copied()
                    .filter(|&n| {
                        let k = self.rooms[n].kind;
                        !r.is_connected(n) && !SPECIALS.contains(&k) && k != RoomType::Pit
                    })
                    .collect();
                if candidates.len() > 1 {
                    let c = candidates[self.rng.pick(candidates.len()).unwrap()];
                    room::connect(&mut self.rooms, i, c);
                }
            }
        }
        let mut count = 0;
        for i in 0..self.rooms.len() {
            if self.rooms[i].kind == RoomType::Null {
                let c = self.rooms[i].connected.len() as i32;
                if c == 0 {
                } else if self.rng.int(c * c) == 0 {
                    self.rooms[i].kind = RoomType::Standard;
                    count += 1;
                } else {
                    self.rooms[i].kind = RoomType::Tunnel;
                }
            }
        }
        while count < 4 {
            if let Some(r) = self.random_room(RoomType::Tunnel, 1) {
                self.rooms[r].kind = RoomType::Standard;
                count += 1;
            }
        }
    }

    // --- pintado ---

    pub fn tunnel_tile(&self) -> u8 {
        if self.feeling == Feeling::Chasm { t::EMPTY_SP } else { t::EMPTY }
    }

    fn paint(&mut self) {
        for i in 0..self.rooms.len() {
            if self.rooms[i].kind != RoomType::Null {
                self.place_doors(i);
                painters::paint(self, i);
            } else if self.feeling == Feeling::Chasm && self.rng.int(2) == 0 {
                fill_room(self, i, 0, t::WALL);
            }
        }
        for i in 0..self.rooms.len() {
            self.paint_doors(i);
        }
    }

    fn place_doors(&mut self, i: usize) {
        for k in 0..self.rooms[i].connected.len() {
            let (n, door) = self.rooms[i].connected[k];
            if door.is_some() {
                continue;
            }
            let x = self.rooms[i].rect.intersect(&self.rooms[n].rect);
            let d = if x.width() == 0 {
                Door::new(x.left, self.rng.int_range(x.top + 1, x.bottom))
            } else {
                Door::new(self.rng.int_range(x.left + 1, x.right), x.top)
            };
            self.doors.push(d);
            let id = self.doors.len() - 1;
            self.rooms[i].connected[k].1 = Some(id);
            if let Some(e) = self.rooms[n].connected.iter_mut().find(|(r, _)| *r == i) {
                e.1 = Some(id);
            }
        }
    }

    fn paint_doors(&mut self, i: usize) {
        for k in 0..self.rooms[i].connected.len() {
            let (n, door) = self.rooms[i].connected[k];
            if self.join_rooms(i, n) {
                continue;
            }
            let d = self.doors[door.expect("puerta sin colocar")];
            let pos = (d.x + d.y * WIDTH) as usize;
            self.map[pos] = match d.kind {
                DoorType::Empty => t::EMPTY,
                DoorType::Tunnel => self.tunnel_tile(),
                DoorType::Regular if self.depth <= 1 => t::DOOR,
                DoorType::Regular => {
                    let roll = if self.depth < 6 { self.rng.int(12 - self.depth) } else { self.rng.int(6) };
                    if roll == 0 {
                        self.secret_doors += 1;
                        t::SECRET_DOOR
                    } else {
                        t::DOOR
                    }
                }
                DoorType::Unlocked => t::DOOR,
                DoorType::Hidden => {
                    self.secret_doors += 1;
                    t::SECRET_DOOR
                }
                DoorType::Barricade => {
                    if self.rng.int(3) == 0 { t::BOOKSHELF } else { t::BARRICADE }
                }
                DoorType::Locked => t::LOCKED_DOOR,
            };
        }
    }

    /// Dos STANDARD contiguas y chicas se fusionan abriendo la pared compartida.
    fn join_rooms(&mut self, r: usize, n: usize) -> bool {
        if self.rooms[r].kind != RoomType::Standard || self.rooms[n].kind != RoomType::Standard {
            return false;
        }
        let (a, b) = (self.rooms[r].rect, self.rooms[n].rect);
        let mut w = a.intersect(&b);
        if w.left == w.right {
            if w.bottom - w.top < 3 || w.height() == a.height().max(b.height()) || a.width() + b.width() > MAX_ROOM_SIZE {
                return false;
            }
            w.top += 1;
            w.right += 1;
            fill(self, w.left, w.top, 1, w.height(), t::EMPTY);
        } else {
            if w.right - w.left < 3 || w.width() == a.width().max(b.width()) || a.height() + b.height() > MAX_ROOM_SIZE {
                return false;
            }
            w.left += 1;
            w.bottom += 1;
            fill(self, w.left, w.top, w.width(), 1, t::EMPTY);
        }
        true
    }

    fn paint_water(&mut self) {
        let density = if self.feeling == Feeling::Water { 0.60 } else { 0.45 };
        let lake = patch::generate(&mut self.rng, density, 5);
        for (c, wet) in self.map.iter_mut().zip(lake) {
            if *c == t::EMPTY && wet {
                *c = t::WATER;
            }
        }
    }

    fn paint_grass(&mut self) {
        let density = if self.feeling == Feeling::Grass { 0.60 } else { 0.40 };
        let mut grass = patch::generate(&mut self.rng, density, 4);
        if self.feeling == Feeling::Grass {
            for r in &self.rooms {
                if !matches!(r.kind, RoomType::Null | RoomType::Passage | RoomType::Tunnel) {
                    let r = r.rect;
                    for (x, y) in [(r.left + 1, r.top + 1), (r.right - 1, r.top + 1), (r.left + 1, r.bottom - 1), (r.right - 1, r.bottom - 1)] {
                        grass[(x + y * WIDTH) as usize] = true;
                    }
                }
            }
        }
        for i in (WIDTH + 1)..(LENGTH - WIDTH - 1) {
            let u = i as usize;
            if self.map[u] == t::EMPTY && grass[u] {
                let count = 1 + NEIGHBOURS8.iter().filter(|&&n| grass[(i + n) as usize]).count();
                self.map[u] = if self.rng.float() < count as f64 / 12.0 { t::HIGH_GRASS } else { t::GRASS };
            }
        }
    }

    fn place_traps(&mut self) {
        let n = if self.depth <= 1 { 0 } else { self.rng.int_range(1, self.rooms.len() as i32 + self.depth) };
        for _ in 0..n {
            let pos = self.rng.int(LENGTH) as usize;
            if self.map[pos] == t::EMPTY {
                self.map[pos] = SECRET_TRAPS[self.rng.chances(&[1.0; 8])];
            }
        }
    }

    // --- SewerLevel.decorate ---

    fn decorate(&mut self) {
        let w = WIDTH as usize;
        for i in 0..w {
            if self.map[i] == t::WALL && self.map[i + w] == t::WATER && self.rng.int(4) == 0 {
                self.map[i] = t::WALL_DECO;
            }
        }
        for i in w..LENGTH as usize - w {
            if self.map[i] == t::WALL && self.map[i - w] == t::WALL && self.map[i + w] == t::WATER && self.rng.int(2) == 0 {
                self.map[i] = t::WALL_DECO;
            }
        }
        for i in w + 1..LENGTH as usize - w - 1 {
            if self.map[i] == t::EMPTY {
                let m = &self.map;
                let count = [m[i + 1], m[i - 1], m[i + w], m[i - w]].iter().filter(|&&c| c == t::WALL).count() as i32;
                if self.rng.int(16) < count * count {
                    self.map[i] = t::EMPTY_DECO;
                }
            }
        }
        loop {
            let pos = self.rooms[self.room_entrance].random(&mut self.rng, 0);
            if pos != self.entrance {
                self.map[pos as usize] = t::SIGN;
                break;
            }
        }
    }

    // --- Level.buildFlagMaps / cleanWalls ---

    fn build_flag_maps(&mut self) {
        let f: Vec<u16> = self.map.iter().map(|&c| t::flags(c)).collect();
        let has = |bit: u16| f.iter().map(|x| x & bit != 0).collect::<Vec<bool>>();
        self.passable = has(t::PASSABLE);
        self.los_blocking = has(t::LOS_BLOCKING);
        self.flamable = has(t::FLAMABLE);
        self.secret = has(t::SECRET);
        self.solid = has(t::SOLID);
        self.avoid = has(t::AVOID);
        self.water = has(t::LIQUID);
        self.pit = has(t::PIT);

        // El borde del mapa nunca es transitable.
        for i in 0..LENGTH {
            let (x, y) = (i % WIDTH, i / WIDTH);
            if x == 0 || x == WIDTH - 1 || y == 0 || y == HEIGHT - 1 {
                self.passable[i as usize] = false;
                self.avoid[i as usize] = false;
            }
        }

        // En el lugar y en orden (como el prototipo): una celda ya cosida se lee como vecina de la siguiente.
        for i in WIDTH..LENGTH - WIDTH {
            let u = i as usize;
            if self.water[u] {
                let mut tile = t::WATER_TILES;
                for (j, n) in NEIGHBOURS4.iter().enumerate() {
                    if t::flags(self.map[(i + n) as usize]) & t::UNSTITCHABLE != 0 {
                        tile += 1 << j;
                    }
                }
                self.map[u] = tile;
            }
            let up = (i - WIDTH) as usize;
            if self.pit[u] && !self.pit[up] {
                let c = self.map[up];
                self.map[u] = if c == t::EMPTY_SP || c == t::STATUE_SP {
                    t::CHASM_FLOOR_SP
                } else if self.water[up] {
                    t::CHASM_WATER
                } else if t::flags(c) & t::UNSTITCHABLE != 0 {
                    t::CHASM_WALL
                } else {
                    t::CHASM_FLOOR
                };
            }
        }
    }

    fn clean_walls(&mut self) {
        self.discoverable = (0..LENGTH)
            .map(|i| {
                let near: Vec<usize> =
                    NEIGHBOURS9.iter().map(|n| i + n).filter(|&c| (0..LENGTH).contains(&c)).map(|c| c as usize).collect();
                near.iter().any(|&c| self.map[c] != t::WALL && self.map[c] != t::WALL_DECO)
                    && near.iter().any(|&c| !self.pit[c])
            })
            .collect();
    }

    // --- visión (Level.updateFieldOfView + Dungeon.observe) ---

    /// Recalcula `visible` desde cada posición (la unión) y lo suma a `visited`. Devuelve la vista de cada una.
    /// ponytail: sin estados que cambien la vista (ceguera, visión mental); llegan con los estados (R5)
    pub fn observe(&mut self, positions: &[i32], distance: i32) -> Vec<Vec<bool>> {
        let views: Vec<Vec<bool>> =
            positions.iter().map(|&p| fov::cast_shadow(p % WIDTH, p / WIDTH, &self.los_blocking, distance)).collect();
        for i in 0..LENGTH as usize {
            self.visible[i] = views.iter().any(|v| v[i]);
            self.visited[i] |= self.visible[i];
        }
        views
    }

    // --- cambios de terreno en juego ---

    /// Level.set: cambia una celda y sus flags (no recalcula el cosido del agua).
    pub fn set(&mut self, pos: i32, terrain: u8) {
        let (i, f) = (pos as usize, t::flags(terrain));
        self.map[i] = terrain;
        self.passable[i] = f & t::PASSABLE != 0;
        self.los_blocking[i] = f & t::LOS_BLOCKING != 0;
        self.flamable[i] = f & t::FLAMABLE != 0;
        self.secret[i] = f & t::SECRET != 0;
        self.solid[i] = f & t::SOLID != 0;
        self.avoid[i] = f & t::AVOID != 0;
        self.pit[i] = f & t::PIT != 0;
        self.water[i] = f & t::LIQUID != 0;
    }

    /// Alguien pisa `pos`: la puerta se abre (Door.enter).
    pub fn enter(&mut self, pos: i32) {
        if self.map[pos as usize] == t::DOOR {
            self.set(pos, t::OPEN_DOOR);
        }
    }

    /// Alguien sale de `pos`: la puerta se cierra (Door.leave; hay una sola criatura por celda).
    pub fn leave(&mut self, pos: i32) {
        if self.map[pos as usize] == t::OPEN_DOOR {
            self.set(pos, t::DOOR);
        }
    }

    /// Un secreto encontrado (puerta / trampa) pasa a su versión visible.
    pub fn discover(&mut self, pos: i32) {
        self.set(pos, t::discover(self.map[pos as usize]));
    }

    // --- utilidades ---

    pub fn random_room(&mut self, kind: RoomType, tries: u32) -> Option<usize> {
        for _ in 0..tries {
            let r = self.rng.pick(self.rooms.len())?;
            if self.rooms[r].kind == kind {
                return Some(r);
            }
        }
        None
    }
}

/// Volcado ASCII para revisar a mano, igual al del prototipo (`python -m game.level <seed> <piso>`).
pub fn to_ascii(level: &Level) -> String {
    let ch = |c: u8| -> char {
        if (t::WATER_TILES..=t::WATER).contains(&c) {
            return '~';
        }
        let f = t::flags(c);
        if f & t::PIT != 0 {
            return ' ';
        }
        if f & t::SECRET != 0 {
            return if c == t::SECRET_DOOR { 'S' } else { '^' };
        }
        match c {
            t::WALL => '#',
            t::WALL_DECO => '%',
            t::EMPTY | t::EMPTY_DECO => '.',
            t::EMPTY_SP => ',',
            t::GRASS => '"',
            t::HIGH_GRASS => '&',
            t::DOOR => '+',
            t::ENTRANCE => '<',
            t::EXIT => '>',
            t::SIGN => '?',
            t::EMBERS => ':',
            t::INACTIVE_TRAP => '_',
            t::BOOKSHELF => 'B',
            t::PEDESTAL => 'P',
            t::FIRE_TRAP => '^',
            t::BARRICADE => '=',
            t::LOCKED_DOOR => 'L',
            _ => '*',
        }
    };
    (0..HEIGHT)
        .map(|y| (0..WIDTH).map(|x| ch(level.map[(x + y * WIDTH) as usize])).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
