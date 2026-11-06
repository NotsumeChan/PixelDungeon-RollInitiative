//! Caminos propios: BFS en 8 direcciones sobre el mapa plano. Se expande desde el destino hasta llegar a quien camina;
//! el paso es su vecina más cerca del destino. Con 8 direcciones la diagonal cuesta lo mismo que el paso recto y hay
//! muchos caminos igual de cortos: entre ellos gana la vecina más pegada a la recta hasta el destino (sin esto,
//! subía en diagonal y zigzagueaba). Todo en enteros: igual en todas las máquinas.
use std::collections::VecDeque;

/// Las 8 vecinas de `c` en un mapa de ancho `w` y `n` celdas, sin salirse ni dar la vuelta por los bordes.
fn neighbours(c: i32, w: i32, n: i32) -> impl Iterator<Item = i32> {
    [-w - 1, -w, -w + 1, -1, 1, w - 1, w, w + 1]
        .into_iter()
        .map(move |d| c + d)
        .filter(move |&m| (0..n).contains(&m) && ((m % w) - (c % w)).abs() <= 1)
}

/// Siguiente celda desde `from` hacia `to` por lo transitable (`to` no necesita serlo: un mob, un cofre), o -1 si no
/// hay camino. `w` = ancho del mapa (`passable` tiene una celda por casilla).
pub fn get_step(from: i32, to: i32, passable: &[bool], w: i32) -> i32 {
    let n = passable.len() as i32;
    if from == to || !(0..n).contains(&from) || !(0..n).contains(&to) {
        return -1;
    }
    let mut dist = vec![u32::MAX; n as usize];
    dist[to as usize] = 0;
    let mut queue = VecDeque::from([to]);
    'bfs: while let Some(c) = queue.pop_front() {
        for m in neighbours(c, w, n) {
            if dist[m as usize] != u32::MAX || !(m == from || passable[m as usize]) {
                continue;
            }
            dist[m as usize] = dist[c as usize] + 1;
            if m == from {
                break 'bfs;
            }
            queue.push_back(m);
        }
    }
    if dist[from as usize] == u32::MAX {
        return -1;
    }
    let (fx, fy, dx, dy) = (from % w, from / w, to % w - from % w, to / w - from / w);
    // (pasos que faltan, lejanía de la recta from -> to (producto cruz), distancia al destino al cuadrado)
    let key = |m: i32| {
        let (x, y) = (m % w - fx, m / w - fy);
        (dist[m as usize], (dx * y - dy * x).abs(), (dx - x).pow(2) + (dy - y).pow(2))
    };
    neighbours(from, w, n).filter(|&m| dist[m as usize] < dist[from as usize]).min_by_key(|&m| key(m)).unwrap_or(-1)
}
