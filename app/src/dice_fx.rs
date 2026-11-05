//! Animación de dados (`screens/diceFx.py`): cada tirada que el héroe ve se muestra en un recuadro (números al azar y
//! luego el resultado). Mínimo = borde rojo y vibración; máximo = borde dorado, chispas y vibración. Tres tamaños:
//! salvación (decisiva) grande en el centro, d20 de ataque mediano, el resto (daño...) chico abajo a la derecha.
//! El azar de aquí es sólo visual: el resultado real ya lo decidió `game` con su rng.
use std::collections::VecDeque;

use game::dungeon::DiceEvent;
use game::rng::Rng;

use crate::atlas::Atlas;
use crate::font::Font;
use crate::gpu::Sprite;

const BOX_W: f32 = 34.0;
const BOX_H: f32 = 20.0;
/// Distancia al borde derecho / inferior de los recuadros chico y mediano.
const BOX_MARGIN: (f32, f32) = (4.0, 29.0);
/// Cada cuánto cambia el número mientras gira.
const TICK_TIME: f32 = 0.05;
const SHAKE_TIME: f32 = 0.3;
const SHAKE_PX: i32 = 2;
/// Con más tiradas en cola, todo va al doble de rápido.
const FAST_QUEUE: usize = 5;
/// El recuadro decisivo va centrado, esto por encima del centro (no tapa al héroe).
const DECIDE_RISE: f32 = 12.0;
const RED: [f32; 4] = [220.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 1.0];
const GOLD: [f32; 4] = [1.0, 200.0 / 255.0, 40.0 / 255.0, 1.0];
const ORANGE: [f32; 4] = [1.0, 120.0 / 255.0, 20.0 / 255.0, 1.0];
const GREEN: [f32; 4] = [60.0 / 255.0, 200.0 / 255.0, 60.0 / 255.0, 1.0];
const FRAME: [f32; 4] = [120.0 / 255.0, 110.0 / 255.0, 90.0 / 255.0, 1.0];
const BG: [f32; 4] = [20.0 / 255.0, 18.0 / 255.0, 16.0 / 255.0, 210.0 / 255.0];
const LABEL: [f32; 4] = [136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0, 1.0];
const WHITE: [f32; 4] = [1.0; 4];

/// Tamaño de un recuadro: escala, segundos girando, segundos mostrando, mínimo mostrando si hay cola.
#[derive(Clone, Copy, PartialEq)]
struct Tier {
    scale: f32,
    roll: f32,
    show: f32,
    min_show: f32,
}

const SMALL: Tier = Tier { scale: 1.0, roll: 0.35, show: 1.2, min_show: 0.7 };
const D20: Tier = Tier { scale: 2.0, roll: 0.35, show: 1.2, min_show: 0.7 };
/// Salvaciones: gira más (suspenso) y el resultado queda más tiempo.
const DECIDE: Tier = Tier { scale: 3.0, roll: 0.7, show: 1.8, min_show: 1.2 };

fn tier(e: &DiceEvent) -> Tier {
    if e.ok.is_some() {
        DECIDE
    } else if e.hi - e.lo == 19 {
        D20
    } else {
        SMALL
    }
}

pub struct DiceFx {
    queue: VecDeque<DiceEvent>,
    current: Option<DiceEvent>,
    tier: Tier,
    t: f32,
    shown: i32,
    tick: f32,
    /// [x, y, vx, vy, vida] en pixeles de escala 1, desde el centro del recuadro.
    sparks: Vec<[f32; 5]>,
    /// Centro del último recuadro dibujado y su escala (las chispas lo siguen).
    anchor: (f32, f32, f32),
    rng: Rng,
}

impl DiceFx {
    pub fn new(seed: i64) -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            tier: SMALL,
            t: 0.0,
            shown: 0,
            tick: 0.0,
            sparks: Vec::new(),
            anchor: (0.0, 0.0, 1.0),
            rng: Rng::new(seed),
        }
    }

    pub fn feed(&mut self, events: &mut Vec<DiceEvent>) {
        self.queue.extend(events.drain(..));
    }

    /// true mientras falte mostrar algún resultado (el juego espera antes de aplicar el daño).
    pub fn busy(&self) -> bool {
        !self.queue.is_empty() || (self.current.is_some() && self.t < self.tier.roll)
    }

    pub fn update(&mut self, dt: f32) {
        let speed = if self.queue.len() >= FAST_QUEUE { 2.0 } else { 1.0 };
        for s in &mut self.sparks {
            s[0] += s[2] * dt;
            s[1] += s[3] * dt;
            s[3] += 200.0 * dt; // gravedad
            s[4] -= dt;
        }
        self.sparks.retain(|s| s[4] > 0.0);
        if self.current.is_none() {
            let Some(e) = self.queue.pop_front() else { return };
            self.tier = tier(&e);
            self.current = Some(e);
            self.t = 0.0;
            self.tick = 0.0;
        }
        let Tier { roll, show, min_show, .. } = self.tier;
        let step = dt * speed;
        self.t += step;
        let e = self.current.as_ref().unwrap();
        let (value, lo, hi, ok) = (e.value, e.lo, e.hi, e.ok);
        if self.t < roll {
            self.tick -= step;
            if self.tick <= 0.0 {
                self.shown = self.rng.int_range_incl(lo, hi);
                self.tick = TICK_TIME;
            }
        } else if self.t - step < roll && ((value == hi && hi > lo) || ok == Some(true)) {
            self.burst(if ok == Some(true) { 28 } else { 14 }); // justo al terminar de girar: máximo o superada
        }
        if self.t >= roll + show || (!self.queue.is_empty() && self.t >= roll + min_show) {
            self.current = None;
        }
    }

    fn burst(&mut self, n: usize) {
        for _ in 0..n {
            let r = &mut self.rng;
            let x = r.float_range(-(BOX_W as f64) / 2.0, BOX_W as f64 / 2.0) as f32;
            let y = r.float_range(-(BOX_H as f64) / 2.0, BOX_H as f64 / 2.0) as f32;
            let vx = r.float_range(-70.0, 70.0) as f32;
            let vy = r.float_range(-110.0, -30.0) as f32;
            let life = r.float_range(0.3, 0.6) as f32;
            self.sparks.push([x, y, vx, vy, life]);
        }
    }

    /// En la capa de UI (w x h pixeles virtuales). `bottom` = lo que ocupa la UI abajo (los dados chicos van arriba:
    /// en vertical, del log).
    pub fn draw(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, bottom: f32) {
        if let Some(e) = self.current.clone() {
            let bw = BOX_W.max(font.width(&e.label) + 6.0); // "CON CD11" no entra en el ancho base
            // un texto largo ("Persuasión CD12") a escala 3 se sale en vertical: la escala baja (entera, nítida) hasta
            // que el recuadro entre en el ancho
            let s = self.tier.scale.min(((w - 4.0) / bw).floor()).max(1.0);
            let rolling = self.t < self.tier.roll;
            let extreme = if rolling || e.hi == e.lo {
                None
            } else if e.value == e.lo {
                Some(RED)
            } else if e.value == e.hi {
                Some(GOLD)
            } else {
                None
            };
            // salvación: verde superada, rojo fallada
            let border = match e.ok {
                Some(ok) if !rolling => Some(if ok { GREEN } else { RED }),
                _ => extreme,
            };
            let (mut dx, mut dy) = (0.0, 0.0);
            if (extreme.is_some() || (e.ok.is_some() && !rolling)) && self.t - self.tier.roll < SHAKE_TIME {
                dx = self.rng.int_range_incl(-SHAKE_PX, SHAKE_PX) as f32 * s;
                dy = self.rng.int_range_incl(-SHAKE_PX, SHAKE_PX) as f32 * s;
            }
            let (x, y) = if self.tier == DECIDE {
                (((w - bw * s) / 2.0).floor(), (h / 2.0 - BOX_H * s - DECIDE_RISE).floor())
            } else {
                (w - BOX_MARGIN.0 - bw * s, h - BOX_MARGIN.1.max(bottom) - BOX_H * s)
            };
            let (x, y) = (x + dx, y + dy);
            out.push(atlas.rect([x, y], [bw * s, BOX_H * s], BG));
            let c = border.unwrap_or(FRAME);
            out.push(atlas.rect([x, y], [bw * s, s], c));
            out.push(atlas.rect([x, y + (BOX_H - 1.0) * s], [bw * s, s], c));
            out.push(atlas.rect([x, y], [s, BOX_H * s], c));
            out.push(atlas.rect([x + (bw - 1.0) * s, y], [s, BOX_H * s], c));
            let num = (if rolling { self.shown } else { e.value }).to_string();
            let nw = font.width(&num);
            let ncolor = if rolling { WHITE } else { border.unwrap_or(WHITE) };
            font.text_scaled(out, &num, x + ((bw - nw) / 2.0).floor() * s, y + 2.0 * s, ncolor, s);
            let lw = font.width(&e.label);
            font.text_scaled(out, &e.label, x + ((bw - lw) / 2.0).floor() * s, y + (BOX_H - 9.0) * s, LABEL, s);
            self.anchor = (x - dx + bw * s / 2.0, y - dy + BOX_H * s / 2.0, s);
        }
        let (cx, cy, s) = self.anchor;
        for sp in &self.sparks {
            let color = if sp[4] > 0.2 { GOLD } else { ORANGE };
            out.push(atlas.rect([(cx + sp[0] * s).floor(), (cy + sp[1] * s).floor()], [s, s], color));
        }
    }
}
