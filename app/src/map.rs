//! El piso como instancias (DrawLevel + BuildFog de `screens/gameScreen.py`): agua, tiles y la niebla.
use game::level::Level;

use crate::atlas::Atlas;
use crate::gpu::{Sprite, FOG_SIZE};

pub const TILE: f32 = 16.0;
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const WATER_SPEED: f32 = 5.0; // px/s hacia abajo (GameScene: water.offset(0, -5 * elapsed))

// FogOfWar.java: un color por esquina de celda (RGBA), interpolado por el filtro LINEAR de la textura
const FOG_VISIBLE: [u8; 4] = [0, 0, 0, 0];
const FOG_VISITED: [u8; 4] = [0x11, 0x11, 0x11, 0xcc];
const FOG_MAPPED: [u8; 4] = [0x44, 0x22, 0x11, 0xcc];
const FOG_INVISIBLE: [u8; 4] = [0, 0, 0, 0xff];

fn uv(atlas: &Atlas, x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    let (aw, ah) = (atlas.w as f32, atlas.h as f32);
    [x / aw, y / ah, (x + w) / aw, (y + h) / ah]
}

/// Agua (water0.png en mosaico) debajo de todo, bajando `off` px, recortada al mapa. Los tiles de agua (48..63)
/// son transparentes donde hay agua.
pub fn water(out: &mut Vec<Sprite>, atlas: &Atlas, sheet: [u32; 4], level: &Level, ox: f32, oy: f32, off: f32) {
    let (sx, sy, ww, wh) = (sheet[0] as f32, sheet[1] as f32, sheet[2] as f32, sheet[3] as f32);
    let (mw, mh) = (level.width as f32 * TILE, level.height as f32 * TILE);
    let mut y = -wh + off.floor() % wh;
    while y < mh {
        let (top, bottom) = (y.max(0.0), (y + wh).min(mh));
        if bottom > top {
            let mut x = 0.0;
            while x < mw {
                out.push(Sprite {
                    pos: [ox + x, oy + top],
                    size: [ww, bottom - top],
                    uv: uv(atlas, sx, sy + (top - y), ww, bottom - top),
                    color: WHITE,
                });
                x += ww;
            }
        }
        y += wh;
    }
}

/// Un tile por celda: el valor de terreno es el índice en tiles0.png (16 por fila).
pub fn tiles(out: &mut Vec<Sprite>, atlas: &Atlas, sheet: [u32; 4], level: &Level, ox: f32, oy: f32) {
    for i in 0..level.len() {
        let t = level.map[i as usize] as u32;
        let (tx, ty) = (sheet[0] + (t % 16) * 16, sheet[1] + (t / 16) * 16);
        out.push(Sprite {
            pos: [ox + (i % level.width) as f32 * TILE, oy + (i / level.width) as f32 * TILE],
            size: [TILE, TILE],
            uv: uv(atlas, tx as f32, ty as f32, TILE, TILE),
            color: WHITE,
        });
    }
}

/// Esquinas de la niebla: visible si las celdas (hasta 4, las del mapa) que tocan la esquina se ven, recordada si se
/// vieron... (el borde puede ser transitable, M1c: ya no queda negro). Se sube como textura de FOG_SIZE x FOG_SIZE; el mapa usa la esquina de arriba a la izquierda,
/// (ancho + 1) x (alto + 1); lo demás queda negro.
pub fn fog_corners(level: &Level) -> Vec<u8> {
    let (w, h) = (level.width, level.height);
    let mut rgba = Vec::with_capacity((FOG_SIZE * FOG_SIZE * 4) as usize);
    for i in 0..FOG_SIZE as i32 {
        for j in 0..FOG_SIZE as i32 {
            let c = if i > h || j > w {
                FOG_INVISIBLE
            } else {
                let corner: Vec<usize> = [(j, i), (j, i - 1), (j - 1, i), (j - 1, i - 1)]
                    .into_iter()
                    .filter(|&(x, y)| (0..w).contains(&x) && (0..h).contains(&y))
                    .map(|(x, y)| (x + y * w) as usize)
                    .collect();
                if corner.iter().all(|&c| level.visible[c]) {
                    FOG_VISIBLE
                } else if corner.iter().all(|&c| level.visited[c]) {
                    FOG_VISITED
                } else if corner.iter().all(|&c| level.mapped[c]) {
                    FOG_MAPPED
                } else {
                    FOG_INVISIBLE
                }
            };
            rgba.extend_from_slice(&c);
        }
    }
    rgba
}

/// El quad de la niebla: cubre el mapa y cada esquina cae en el centro de su texel (así LINEAR interpola entre
/// esquinas a lo largo de cada tile, como el original).
pub fn fog_quad(level: &Level, ox: f32, oy: f32) -> Sprite {
    let f = FOG_SIZE as f32;
    let uv = [0.5 / f, 0.5 / f, (level.width as f32 + 0.5) / f, (level.height as f32 + 0.5) / f];
    Sprite { pos: [ox, oy], size: [level.width as f32 * TILE, level.height as f32 * TILE], uv, color: WHITE }
}
