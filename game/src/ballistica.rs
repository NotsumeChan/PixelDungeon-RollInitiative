//! mechanics/Ballistica.java (`game/ballistica.py`): línea de tiro de proyectiles y conjuros.
use crate::geom::WIDTH;
use crate::level::Level;

/// Ballistica.cast(from, to, magic=false, hitChars=true): la celda donde se detiene algo que va de `from` a `to`.
/// Se frena en la primera criatura (`stops(celda)`) o en lo que bloquea la vista; contra una pared queda en la celda
/// anterior. Si devuelve `to`, el camino está libre.
pub fn cast(level: &Level, from: i32, to: i32, stops: impl Fn(i32) -> bool) -> i32 {
    let (x0, y0, x1, y1) = (from % WIDTH, from / WIDTH, to % WIDTH, to / WIDTH);
    let (dx, dy) = (x1 - x0, y1 - y0);
    let (sx, sy) = (if dx > 0 { 1 } else { -1 }, if dy > 0 { 1 } else { -1 });
    let (dx, dy) = (dx.abs(), dy.abs());
    let (step_a, step_b, da, db) = if dx > dy { (sx, sy * WIDTH, dx, dy) } else { (sy * WIDTH, sx, dy, dx) };
    let (mut cell, mut err) = (from, da / 2);
    while cell != to {
        let prev = cell;
        cell += step_a;
        err += db;
        if err >= da {
            err -= da;
            cell += step_b;
        }
        let c = cell as usize;
        if !level.passable[c] && !level.avoid[c] {
            return prev;
        }
        if level.los_blocking[c] || stops(cell) {
            return cell;
        }
    }
    to
}
