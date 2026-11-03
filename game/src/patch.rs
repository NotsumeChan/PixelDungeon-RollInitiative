//! Patch.java: ruido con densidad `seed` suavizado `n_gen` veces por autómata celular.
//! Una celda vive con >= 4 vecinas vivas (si ya vivía) o >= 5 (si no).
use crate::geom::{HEIGHT, LENGTH, WIDTH};
use crate::rng::Rng;

pub fn generate(rng: &mut Rng, seed: f64, n_gen: u32) -> Vec<bool> {
    let (w, h) = (WIDTH as usize, HEIGHT as usize);
    let mut off: Vec<bool> = (0..LENGTH).map(|_| rng.random() < seed).collect();
    let mut cur = vec![false; LENGTH as usize];
    for _ in 0..n_gen {
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let p = x + y * w;
                let count = [p - w - 1, p - w, p - w + 1, p - 1, p + 1, p + w - 1, p + w, p + w + 1]
                    .iter()
                    .filter(|&&q| off[q])
                    .count();
                cur[p] = if off[p] { count >= 4 } else { count >= 5 };
            }
        }
        std::mem::swap(&mut off, &mut cur);
    }
    off
}
