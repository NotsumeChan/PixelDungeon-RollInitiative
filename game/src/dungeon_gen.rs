//! Generador propio de zonas (`"gen": "pdri:dungeon"` en `world.json`, M1b): salas rectangulares unidas por pasillos,
//! sin código de PD. Salas al azar sin tocarse; un árbol que las une a todas (cada sala con la más cercana ya unida)
//! más algunos pasillos extra para que haya vueltas; cada pasillo sale por una puerta en la pared que mira a la otra
//! sala y busca su camino por fuera de las salas (BFS). Entrada en una sala, salida en la más lejana a pie. Agua y
//! pasto con ruido suavizado; trampas ocultas desde z = 2. Usa los tiles de siempre (`terrain`).
use std::collections::VecDeque;

use crate::geom::Rect;
use crate::level::{Level, SECRET_TRAPS};
use crate::rng::Rng;
use crate::room::{Room, RoomType};
use crate::terrain as t;

/// Arma el mapa de `level` (vacío, todo pared, con su tamaño y su rng). false = no salió: se reintenta con el rng
/// como quedó (así el próximo intento es otro).
pub fn build(level: &mut Level) -> bool {
    let mut rng = level.rng.clone();
    let ok = try_build(level, &mut rng);
    level.rng = rng;
    ok
}

fn try_build(level: &mut Level, rng: &mut Rng) -> bool {
    let (w, h) = (level.width, level.height);
    let rects = place_rooms(rng, w, h);
    if rects.len() < 3 {
        return false;
    }
    let inside = |c: i32| rects.iter().any(|r| contains(r, c % w, c / w));
    for r in &rects {
        for y in r.top + 1..r.bottom {
            for x in r.left + 1..r.right {
                level.map[(x + y * w) as usize] = t::EMPTY;
            }
        }
    }
    for (a, b) in links(rng, &rects) {
        let (da, oa) = door(rng, &rects[a], &rects[b], w);
        let (db, ob) = door(rng, &rects[b], &rects[a], w);
        let Some(path) = corridor(oa, ob, w, h, &inside) else { continue };
        for c in path {
            level.map[c as usize] = t::EMPTY;
        }
        level.map[da as usize] = t::DOOR;
        level.map[db as usize] = t::DOOR;
    }
    // entrada en la primera sala; salida en la más lejana a pie (si alguna no se alcanza, se reintenta)
    let start = rects[0];
    level.entrance = random_in(rng, &start, w);
    let dist = walk(level.entrance, &level.map, w, h);
    let far = (1..rects.len()).max_by_key(|&i| dist[center(&rects[i], w) as usize]);
    let Some(far) = far.filter(|&i| rects.iter().all(|r| dist[center(r, w) as usize] != u32::MAX) && i > 0) else { return false };
    level.exit = random_in(rng, &rects[far], w);
    level.map[level.entrance as usize] = t::ENTRANCE;
    level.map[level.exit as usize] = t::EXIT;
    level.rooms = rects
        .iter()
        .enumerate()
        .map(|(i, &r)| {
            let mut room = Room::new(r);
            room.kind = if i == 0 { RoomType::Entrance } else if i == far { RoomType::Exit } else { RoomType::Standard };
            room
        })
        .collect();
    (level.room_entrance, level.room_exit) = (0, far);
    paint_noise(level, rng, t::WATER, 0.35);
    paint_noise(level, rng, t::GRASS, 0.40);
    if level.zone.z > 1 {
        for _ in 0..rng.int_range(1, rects.len() as i32 + level.zone.z) {
            let c = rng.int(w * h) as usize;
            if level.map[c] == t::EMPTY {
                level.map[c] = SECRET_TRAPS[rng.int(SECRET_TRAPS.len() as i32) as usize];
            }
        }
    }
    true
}

fn contains(r: &Rect, x: i32, y: i32) -> bool {
    r.left <= x && x <= r.right && r.top <= y && y <= r.bottom
}

fn center(r: &Rect, w: i32) -> i32 {
    (r.left + r.right) / 2 + (r.top + r.bottom) / 2 * w
}

fn random_in(rng: &mut Rng, r: &Rect, w: i32) -> i32 {
    rng.int_range(r.left + 1, r.right) + rng.int_range(r.top + 1, r.bottom) * w
}

/// Salas (con sus paredes) que no se tocan: queda al menos una casilla entre dos, para los pasillos.
fn place_rooms(rng: &mut Rng, w: i32, h: i32) -> Vec<Rect> {
    let want = ((w * h) / 100).clamp(3, 16) as usize;
    let mut rects: Vec<Rect> = Vec::new();
    for _ in 0..want * 12 {
        if rects.len() >= want {
            break;
        }
        let (rw, rh) = (rng.int_range(4, 9), rng.int_range(4, 8)); // interior
        if rw + 4 > w || rh + 4 > h {
            continue;
        }
        let (x, y) = (rng.int_range(1, w - rw - 2), rng.int_range(1, h - rh - 2));
        let r = Rect::new(x, y, x + rw + 1, y + rh + 1);
        let apart = |o: &Rect| r.right + 1 < o.left || o.right + 1 < r.left || r.bottom + 1 < o.top || o.bottom + 1 < r.top;
        if rects.iter().all(apart) {
            rects.push(r);
        }
    }
    rects
}

/// Qué salas se unen: un árbol (cada sala nueva con la más cercana ya unida) y ~1 de cada 4 con otra más.
fn links(rng: &mut Rng, rects: &[Rect]) -> Vec<(usize, usize)> {
    let d = |a: usize, b: usize| {
        let (ra, rb) = (&rects[a], &rects[b]);
        ((ra.left + ra.right) - (rb.left + rb.right)).abs() + ((ra.top + ra.bottom) - (rb.top + rb.bottom)).abs()
    };
    let mut done = vec![0];
    let mut out = Vec::new();
    while done.len() < rects.len() {
        let (a, b) = done
            .iter()
            .flat_map(|&a| (0..rects.len()).filter(|b| !done.contains(b)).map(move |b| (a, b)))
            .min_by_key(|&(a, b)| d(a, b))
            .expect("quedan salas");
        out.push((a, b));
        done.push(b);
    }
    for a in 0..rects.len() {
        if rng.int(4) == 0
            && let Some(b) = (0..rects.len()).filter(|&b| b != a && !out.contains(&(a, b)) && !out.contains(&(b, a))).min_by_key(|&b| d(a, b))
        {
            out.push((a, b));
        }
    }
    out
}

/// Puerta en la pared de `r` que mira hacia `to` (sin esquinas) y la casilla de afuera, donde arranca el pasillo.
fn door(rng: &mut Rng, r: &Rect, to: &Rect, w: i32) -> (i32, i32) {
    let (dx, dy) = ((to.left + to.right) - (r.left + r.right), (to.top + to.bottom) - (r.top + r.bottom));
    let (x, y, ox, oy) = if dx.abs() >= dy.abs() {
        let x = if dx > 0 { r.right } else { r.left };
        (x, rng.int_range(r.top + 1, r.bottom), dx.signum(), 0)
    } else {
        let y = if dy > 0 { r.bottom } else { r.top };
        (rng.int_range(r.left + 1, r.right), y, 0, dy.signum())
    };
    (x + y * w, (x + ox) + (y + oy) * w)
}

/// Camino en 4 direcciones de `a` a `b` por fuera de las salas y del borde del mapa (BFS), o None.
fn corridor(a: i32, b: i32, w: i32, h: i32, inside: &impl Fn(i32) -> bool) -> Option<Vec<i32>> {
    let ok = |c: i32| (c % w) > 0 && (c % w) < w - 1 && (c / w) > 0 && (c / w) < h - 1 && !inside(c);
    if !ok(a) || !ok(b) {
        return None;
    }
    let mut from = vec![-1; (w * h) as usize];
    from[a as usize] = a;
    let mut queue = VecDeque::from([a]);
    while let Some(c) = queue.pop_front() {
        if c == b {
            let mut path = vec![b];
            while *path.last()? != a {
                path.push(from[*path.last()? as usize]);
            }
            return Some(path);
        }
        for n in [c - w, c + 1, c + w, c - 1] {
            if (0..w * h).contains(&n) && from[n as usize] < 0 && ok(n) {
                from[n as usize] = c;
                queue.push_back(n);
            }
        }
    }
    None
}

/// Distancias a pie (4 direcciones) desde `start` por lo que no es pared.
fn walk(start: i32, map: &[u8], w: i32, h: i32) -> Vec<u32> {
    let mut dist = vec![u32::MAX; (w * h) as usize];
    dist[start as usize] = 0;
    let mut queue = VecDeque::from([start]);
    while let Some(c) = queue.pop_front() {
        for n in [c - w, c + 1, c + w, c - 1] {
            if (0..w * h).contains(&n) && dist[n as usize] == u32::MAX && map[n as usize] != t::WALL {
                dist[n as usize] = dist[c as usize] + 1;
                queue.push_back(n);
            }
        }
    }
    dist
}

/// Manchas de `tile` sobre el piso vacío: ruido con densidad `density`, suavizado dos veces (una casilla queda si la
/// mayoría de su vecindario también).
fn paint_noise(level: &mut Level, rng: &mut Rng, tile: u8, density: f64) {
    let (w, h) = (level.width, level.height);
    let mut on: Vec<bool> = (0..w * h).map(|_| rng.float() < density).collect();
    for _ in 0..2 {
        on = (0..w * h)
            .map(|c| {
                let (x, y) = (c % w, c / w);
                let n = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (x + dx, y + dy))).filter(|&(x, y)| (0..w).contains(&x) && (0..h).contains(&y) && on[(x + y * w) as usize]).count();
                n >= 5
            })
            .collect();
    }
    for (c, _) in on.iter().enumerate().filter(|e| *e.1) {
        if level.map[c] == t::EMPTY {
            level.map[c] = tile;
        }
    }
}
