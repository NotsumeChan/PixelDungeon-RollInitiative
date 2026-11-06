//! Línea de tiro propia (Bresenham): por dónde pasa un proyectil o un conjuro.
use crate::level::Level;

/// La celda donde se detiene algo que va de `from` a `to`: en la primera criatura (`stops(celda)`) o en lo que tapa la
/// vista; contra una pared queda en la celda anterior. Si devuelve `to`, el camino está libre.
pub fn cast(level: &Level, from: i32, to: i32, stops: impl Fn(i32) -> bool) -> i32 {
    let w = level.width;
    let (mut x, mut y, x1, y1) = (from % w, from / w, to % w, to / w);
    let (dx, dy, sx, sy) = ((x1 - x).abs(), -(y1 - y).abs(), (x1 - x).signum(), (y1 - y).signum());
    let (mut err, mut prev) = (dx + dy, from);
    while (x, y) != (x1, y1) {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
        let cell = x + y * w;
        let c = cell as usize;
        if !level.passable[c] && !level.avoid[c] {
            return prev;
        }
        if level.los_blocking[c] || stops(cell) {
            return cell;
        }
        prev = cell;
    }
    to
}
