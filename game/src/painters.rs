//! painters/*.java (`game/painters.py`): cada tipo de sala se pinta sobre `level.map`.
//! Hoy: STANDARD (con sus 6 variantes), ENTRANCE, EXIT, TUNNEL. Las salas especiales llegan después.
use crate::level::Level;
use crate::room::{DoorType, RoomType};
use crate::terrain::{self as t, Feeling};

pub fn set_cell(level: &mut Level, x: i32, y: i32, v: u8) {
    level.map[(x + y * level.width) as usize] = v;
}

pub fn fill(level: &mut Level, x: i32, y: i32, w: i32, h: i32, v: u8) {
    for j in y..y + h {
        let start = (x + j * level.width).max(0);
        let end = (start + w.max(0)).min(level.len());
        for c in start..end {
            level.map[c as usize] = v;
        }
    }
}

/// El rect de la sala incluye sus paredes; `m` = margen hacia adentro.
pub fn fill_room(level: &mut Level, room: usize, m: i32, v: u8) {
    let r = level.rooms[room].rect;
    fill(level, r.left + m, r.top + m, r.width() + 1 - m * 2, r.height() + 1 - m * 2, v);
}

/// Desde una puerta (en la pared) avanza `n` celdas hacia el interior pintando `v`.
fn draw_inside(level: &mut Level, room: usize, door: usize, n: i32, v: u8) {
    let r = level.rooms[room].rect;
    let d = level.doors[door];
    let (sx, sy) = if d.x == r.left {
        (1, 0)
    } else if d.x == r.right {
        (-1, 0)
    } else if d.y == r.top {
        (0, 1)
    } else {
        (0, -1)
    };
    let (mut x, mut y) = (d.x + sx, d.y + sy);
    for _ in 0..n {
        set_cell(level, x, y, v);
        x += sx;
        y += sy;
    }
}

/// `range(a, b, step)` de Python con step 1 o -1.
fn prange(a: i32, b: i32, step: i32) -> impl Iterator<Item = i32> {
    let n = if step > 0 { (b - a).max(0) } else { (a - b).max(0) };
    (0..n).map(move |k| a + k * step)
}

fn doors_of(level: &Level, room: usize) -> Vec<usize> {
    level.rooms[room].connected.iter().map(|(_, d)| d.expect("puerta sin colocar")).collect()
}

fn set_doors(level: &mut Level, room: usize, kind: DoorType) {
    for d in doors_of(level, room) {
        level.doors[d].set(kind);
    }
}

pub fn paint(level: &mut Level, room: usize) {
    match level.rooms[room].kind {
        RoomType::Standard => paint_standard(level, room),
        RoomType::Entrance => level.entrance = stairs_room(level, room, t::ENTRANCE),
        RoomType::Exit => level.exit = stairs_room(level, room, t::EXIT),
        RoomType::Tunnel => paint_tunnel(level, room),
        k => panic!("sala sin pintor: {k:?}"),
    }
}

// --- EntrancePainter / ExitPainter ---

fn stairs_room(level: &mut Level, room: usize, tile: u8) -> i32 {
    fill_room(level, room, 0, t::WALL);
    fill_room(level, room, 1, t::EMPTY);
    set_doors(level, room, DoorType::Regular);
    let pos = level.rooms[room].random(&mut level.rng, 1, level.width);
    level.map[pos as usize] = tile;
    pos
}

// --- TunnelPainter ---

fn paint_tunnel(level: &mut Level, room: usize) {
    let floor = level.tunnel_tile();
    let (cx, cy) = level.rooms[room].center(&mut level.rng);
    let r = level.rooms[room].rect;
    let doors: Vec<_> = doors_of(level, room).into_iter().map(|d| level.doors[d]).collect();
    if r.width() > r.height() || (r.width() == r.height() && level.rng.int(2) == 0) {
        let (mut from, mut to) = (r.right - 1, r.left + 1);
        for d in &doors {
            let step = if d.y < cy { 1 } else { -1 };
            if d.x == r.left {
                from = r.left + 1;
                for i in prange(d.y, cy, step) {
                    set_cell(level, from, i, floor);
                }
            } else if d.x == r.right {
                to = r.right - 1;
                for i in prange(d.y, cy, step) {
                    set_cell(level, to, i, floor);
                }
            } else {
                from = from.min(d.x);
                to = to.max(d.x);
                for i in prange(d.y + step, cy, step) {
                    set_cell(level, d.x, i, floor);
                }
            }
        }
        for i in from..=to {
            set_cell(level, i, cy, floor);
        }
    } else {
        let (mut from, mut to) = (r.bottom - 1, r.top + 1);
        for d in &doors {
            let step = if d.x < cx { 1 } else { -1 };
            if d.y == r.top {
                from = r.top + 1;
                for i in prange(d.x, cx, step) {
                    set_cell(level, i, from, floor);
                }
            } else if d.y == r.bottom {
                to = r.bottom - 1;
                for i in prange(d.x, cx, step) {
                    set_cell(level, i, to, floor);
                }
            } else {
                from = from.min(d.y);
                to = to.max(d.y);
                for i in prange(d.x + step, cx, step) {
                    set_cell(level, i, d.y, floor);
                }
            }
        }
        for i in from..=to {
            set_cell(level, cx, i, floor);
        }
    }
    set_doors(level, room, DoorType::Tunnel);
}

// --- StandardPainter ---

fn paint_standard(level: &mut Level, room: usize) {
    fill_room(level, room, 0, t::WALL);
    set_doors(level, room, DoorType::Regular);

    if !level.boss && level.rng.int(5) == 0 {
        let c = level.rng.int(6);
        let r = level.rooms[room].rect;
        let (w, h) = (r.width(), r.height());
        // Mismo encadenado que el prototipo (el switch de Java con fall-through, pasado a elif).
        if c == 0 && level.feeling != Feeling::Grass {
            if w.min(h) >= 4 && w.max(h) >= 6 {
                return graveyard(level, room);
            }
        } else if c == 0 || c == 1 {
            if level.zone.z > 1 {
                return burned(level, room);
            }
        } else if c == 2 {
            if w.max(h) >= 4 {
                return striped(level, room);
            }
        } else if c == 3 {
            if w >= 6 && h >= 6 {
                return study(level, room);
            }
        } else if c == 4 && level.feeling != Feeling::Water {
            if level.rooms[room].connected.len() == 2 && w >= 4 && h >= 4 {
                return bridge(level, room);
            }
        } else if (c == 4 || c == 5) && !level.boss && !level.boss_next && w.min(h) >= 5 {
            return fissure(level, room);
        }
    }
    fill_room(level, room, 1, t::EMPTY);
}

fn burned(level: &mut Level, room: usize) {
    const PICK: [u8; 5] = [t::EMPTY, t::FIRE_TRAP, t::SECRET_FIRE_TRAP, t::INACTIVE_TRAP, t::EMBERS];
    let r = level.rooms[room].rect;
    for y in r.top + 1..r.bottom {
        for x in r.left + 1..r.right {
            let v = PICK[level.rng.int(5) as usize];
            level.map[(x + y * level.width) as usize] = v;
        }
    }
}

fn graveyard(level: &mut Level, room: usize) {
    let r = level.rooms[room].rect;
    fill(level, r.left + 1, r.top + 1, r.width() - 1, r.height() - 1, t::GRASS);
    let (w, h) = (r.width() - 1, r.height() - 1);
    let n_graves = w.max(h) / 2;
    let index = level.rng.int(n_graves);
    let shift = level.rng.int(2);
    for i in 0..n_graves {
        let pos = if w > h {
            r.left + 1 + shift + i * 2 + (r.top + 2 + level.rng.int(h - 2)) * level.width
        } else {
            (r.left + 2 + level.rng.int(w - 2)) + (r.top + 1 + shift + i * 2) * level.width
        };
        // ponytail: tumbas como (celda, tiene ítem); el drop real (ítem / oro) llega con los ítems (R5)
        level.tombs.push((pos, i == index));
    }
}

fn striped(level: &mut Level, room: usize) {
    let r = level.rooms[room].rect;
    fill(level, r.left + 1, r.top + 1, r.width() - 1, r.height() - 1, t::EMPTY_SP);
    if r.width() > r.height() {
        for x in (r.left + 2..r.right).step_by(2) {
            fill(level, x, r.top + 1, 1, r.height() - 1, t::HIGH_GRASS);
        }
    } else {
        for y in (r.top + 2..r.bottom).step_by(2) {
            fill(level, r.left + 1, y, r.width() - 1, 1, t::HIGH_GRASS);
        }
    }
}

fn study(level: &mut Level, room: usize) {
    let r = level.rooms[room].rect;
    fill(level, r.left + 1, r.top + 1, r.width() - 1, r.height() - 1, t::BOOKSHELF);
    fill(level, r.left + 2, r.top + 2, r.width() - 3, r.height() - 3, t::EMPTY_SP);
    for d in doors_of(level, room) {
        let d = level.doors[d];
        if d.x == r.left {
            set_cell(level, d.x + 1, d.y, t::EMPTY);
        } else if d.x == r.right {
            set_cell(level, d.x - 1, d.y, t::EMPTY);
        } else if d.y == r.top {
            set_cell(level, d.x, d.y + 1, t::EMPTY);
        } else if d.y == r.bottom {
            set_cell(level, d.x, d.y - 1, t::EMPTY);
        }
    }
    let (cx, cy) = level.rooms[room].center(&mut level.rng);
    set_cell(level, cx, cy, t::PEDESTAL);
}

fn bridge(level: &mut Level, room: usize) {
    let r = level.rooms[room].rect;
    let chasm = !level.boss && !level.boss_next && level.rng.int(3) == 0;
    fill(level, r.left + 1, r.top + 1, r.width() - 1, r.height() - 1, if chasm { t::CHASM } else { t::WATER });
    let ds = doors_of(level, room);
    let (i1, i2) = (ds[0], ds[1]);
    let (d1, d2) = (level.doors[i1], level.doors[i2]);
    let sp = t::EMPTY_SP;
    // {d1.x, d2.x} == {left, right} del prototipo
    let both = |a: i32, b: i32, p: i32, q: i32| (a == p && b == q) || (a == q && b == p);
    if both(d1.x, d2.x, r.left, r.right) {
        let s = r.width() / 2;
        draw_inside(level, room, i1, s, sp);
        draw_inside(level, room, i2, s, sp);
        let cx = level.rooms[room].center(&mut level.rng).0; // gasta las dos tiradas del centro
        fill(level, cx, d1.y.min(d2.y), 1, (d1.y - d2.y).abs() + 1, sp);
    } else if both(d1.y, d2.y, r.top, r.bottom) {
        let s = r.height() / 2;
        draw_inside(level, room, i1, s, sp);
        draw_inside(level, room, i2, s, sp);
        let cy = level.rooms[room].center(&mut level.rng).1;
        fill(level, d1.x.min(d2.x), cy, (d1.x - d2.x).abs() + 1, 1, sp);
    } else if d1.x == d2.x {
        let x = if d1.x == r.left { r.left + 1 } else { r.right - 1 };
        fill(level, x, d1.y.min(d2.y), 1, (d1.y - d2.y).abs() + 1, sp);
    } else if d1.y == d2.y {
        let y = if d1.y == r.top { r.top + 1 } else { r.bottom - 1 };
        fill(level, d1.x.min(d2.x), y, (d1.x - d2.x).abs() + 1, 1, sp);
    } else if d1.y == r.top || d1.y == r.bottom {
        draw_inside(level, room, i1, (d1.y - d2.y).abs(), sp);
        draw_inside(level, room, i2, (d1.x - d2.x).abs(), sp);
    } else if d1.x == r.left || d1.x == r.right {
        draw_inside(level, room, i1, (d1.x - d2.x).abs(), sp);
        draw_inside(level, room, i2, (d1.y - d2.y).abs(), sp);
    }
}

fn fissure(level: &mut Level, room: usize) {
    let r = level.rooms[room].rect;
    fill(level, r.left + 1, r.top + 1, r.width() - 1, r.height() - 1, t::EMPTY);
    for y in r.top + 2..r.bottom - 1 {
        for x in r.left + 2..r.right - 1 {
            let v = (y - r.top).min(r.bottom - y);
            let h = (x - r.left).min(r.right - x);
            if v.min(h) > 2 || level.rng.int(2) == 0 {
                set_cell(level, x, y, t::CHASM);
            }
        }
    }
}
