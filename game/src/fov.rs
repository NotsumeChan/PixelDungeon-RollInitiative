//! mechanics/ShadowCaster.java (`game/fov.py`): campo de visión por shadowcasting en 8 sectores.
use crate::geom::{HEIGHT, LENGTH, WIDTH};

pub const MAX_DISTANCE: i32 = 8;

const SECTORS: [(i32, i32, i32, i32); 8] = [
    (1, 1, 0, 0),
    (-1, 1, 0, 0),
    (1, -1, 0, 0),
    (-1, -1, 0, 0),
    (0, 0, 1, 1),
    (0, 0, -1, 1),
    (0, 0, 1, -1),
    (0, 0, -1, -1),
];

/// Hasta qué q llega la fila p con radio d (recorta el cuadrado a un círculo). Math.round de Java = floor(x + 0.5).
fn limits(d: i32) -> Vec<i32> {
    let mut l = vec![0];
    for j in 1..=d {
        let r = (d as f64 * (j as f64 / (d as f64 + 0.5)).asin().cos() + 0.5).floor() as i32;
        l.push(j.min(r));
    }
    l
}

/// Celdas visibles desde (x, y). distance <= MAX_DISTANCE.
pub fn cast_shadow(x: i32, y: i32, los_blocking: &[bool], distance: i32) -> Vec<bool> {
    let mut fov = vec![false; LENGTH as usize];
    fov[(y * WIDTH + x) as usize] = true;
    let lim = limits(distance);
    for s in SECTORS {
        scan_sector(&mut fov, los_blocking, &lim, distance, x, y, s);
    }
    fov
}

fn scan_sector(fov: &mut [bool], los: &[bool], lim: &[i32], distance: i32, cx: i32, cy: i32, s: (i32, i32, i32, i32)) {
    let (m1, m2, m3, m4) = s;
    // Obstáculos como intervalos de ángulo [a1, a2]; sólo cuentan los de filas anteriores (limit).
    let mut obs: Vec<(f64, f64)> = Vec::new();
    let mut limit = 0;
    for p in 1..=distance {
        let dq2 = 0.5 / p as f64;
        for q in 0..=lim[p as usize] {
            let x = cx + q * m1 + p * m3;
            let y = cy + p * m2 + q * m4;
            if (0..HEIGHT).contains(&y) && (0..WIDTH).contains(&x) {
                let a0 = q as f64 / p as f64;
                let (a1, a2) = (a0 - dq2, a0 + dq2);
                let pos = (y * WIDTH + x) as usize;
                let blocked = |a: f64| obs[..limit].iter().any(|&(o1, o2)| o1 <= a && a <= o2);
                if !(blocked(a0) && blocked(a1) && blocked(a2)) {
                    fov[pos] = true;
                }
                if los[pos] {
                    if obs.len() > limit && a1 <= obs[obs.len() - 1].1 {
                        obs.last_mut().unwrap().1 = a2; // se une al obstáculo anterior de esta fila
                    } else {
                        obs.push((a1, a2));
                    }
                }
            }
        }
        limit = obs.len();
    }
}
