//! utils/PathFinder.java (`game/pathfinder.py`): BFS en 8 direcciones sobre el mapa plano.
use std::collections::VecDeque;

fn dirs(w: i32) -> [i32; 8] {
    [-1, 1, -w, w, -w - 1, -w + 1, w - 1, w + 1]
}

/// Distancias desde `to` hasta encontrar `from` (que no necesita ser transitable). None si no hay camino.
/// Igual que el prototipo, `from` puede anotarse más de una vez antes de salir de la cola (decide el paso).
fn distance_map(from: i32, to: i32, passable: &[bool], w: i32) -> Option<Vec<i32>> {
    let len = passable.len() as i32;
    if from == to {
        return None;
    }
    let mut dist = vec![i32::MAX; len as usize];
    dist[to as usize] = 0;
    let mut queue = VecDeque::from([to]);
    while let Some(step) = queue.pop_front() {
        if step == from {
            return Some(dist);
        }
        let nd = dist[step as usize] + 1;
        for d in dirs(w) {
            let n = step + d;
            if n == from || ((0..len).contains(&n) && passable[n as usize] && dist[n as usize] > nd) {
                dist[n as usize] = nd;
                queue.push_back(n);
            }
        }
    }
    None
}

/// Siguiente celda desde `from` hacia `to`, o -1.
/// `w` = ancho del mapa (`passable` tiene una celda por casilla).
pub fn get_step(from: i32, to: i32, passable: &[bool], w: i32) -> i32 {
    let len = passable.len() as i32;
    let Some(dist) = distance_map(from, to, passable, w) else { return -1 };
    let (mut best, mut min) = (from, dist[from as usize]);
    for d in dirs(w) {
        let n = from + d;
        if (0..len).contains(&n) && dist[n as usize] < min {
            (best, min) = (n, dist[n as usize]);
        }
    }
    best
}
