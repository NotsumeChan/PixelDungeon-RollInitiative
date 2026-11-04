//! Personajes en pantalla (`screens/charSprite.py` + los efectos de `screens/gameScreen.py`): dónde se dibuja cada uno
//! (viaja hacia su celda), su animación (quieto, correr, atacar, operar, morir; mira hacia donde va), los cadáveres que
//! se desvanecen, los textos flotantes, las casillas revisadas y las hojas del pasto. Todo por dt; el azar es visual.
use std::collections::VecDeque;
use game::actor::ActorId;
use game::dungeon::{Dungeon, Fx, Tone};
use game::geom::WIDTH;
use game::mobs::State;
use game::rng::Rng;
use serde_json::Value;

use crate::atlas::Atlas;
use crate::font::Font;
use crate::gpu::Sprite;
use crate::map::TILE;

/// Segundos por casilla (CharSprite.MOVE_INTERVAL).
pub const MOVE_TIME: f32 = 0.1;
/// HeroSprite: frames de 12x15; una fila por tier de armadura.
pub const HERO_W: u32 = 12;
pub const HERO_H: u32 = 15;
// FloatingText.java: segundos de vida, pixeles que sube, separación entre textos del mismo personaje
const FLOAT_TIME: f32 = 1.0;
const FLOAT_RISE: f32 = 16.0;
const FLOAT_STACK: f32 = 8.0;
/// MobSprite: tras morir, el cadáver se desvanece en este tiempo.
const FADE_TIME: f32 = 3.0;
/// CheckedCell: se achica y se desvanece.
const CHECKED_TIME: f32 = 0.8;
const CHECKED_COLOR: [f32; 3] = [0x55 as f32 / 255.0, 0xAA as f32 / 255.0, 1.0];
/// LeafParticle de las Alcantarillas.
const LEAF_COLORS: [[f32; 4]; 2] = [[0.0, 0x44 as f32 / 255.0, 0.0, 1.0], [0x88 as f32 / 255.0, 0xCC as f32 / 255.0, 0x44 as f32 / 255.0, 1.0]];
/// HeroSprite.updateArmor: (nombre, fps, frames, se repite). Iguales en todas las clases.
const HERO_ANIMS: [(&str, f32, &[u32], bool); 6] = [
    ("idle", 1.0, &[0, 0, 0, 1, 0, 0, 1, 1], true),
    ("run", 20.0, &[2, 3, 4, 5, 6, 7], true),
    ("die", 20.0, &[8, 9, 10, 11, 12, 11], false),
    ("attack", 15.0, &[13, 14, 15, 0], false),
    ("operate", 8.0, &[16, 17, 16, 17], false),
    ("read", 20.0, &[19, 20, 20, 20, 20, 20, 20, 20, 20, 19], false),
];
/// Icons.java: Zzz sobre los mobs dormidos y ! cuando te notan (CharSprite.showSleep / showAlert), con su pulso
/// (EmoIcon: tamaño máximo, velocidad).
const ICON_SLEEP: [u32; 4] = [13, 45, 9, 8];
const ICON_ALERT: [u32; 4] = [22, 45, 8, 8];
const EMO_SLEEP: (f32, f32) = (1.2, 0.5);
const EMO_ALERT: (f32, f32) = (1.3, 2.0);
/// Segundos que se ve el ! cuando un mob te nota (en Java, hasta su próximo turno).
const ALERT_TIME: f32 = 1.5;
/// Se reproducen enteras antes de volver a quieto / correr.
const ONESHOT: [&str; 3] = ["attack", "operate", "read"];

/// Hojas de personajes por nombre de archivo (el `sprite` de data/classes.json y data/mobs.json). Van al atlas
/// detrás de las hojas fijas. Sin hoja = se dibuja el nombre.
pub const CHAR_SHEETS: [(&str, &[u8]); 10] = [
    ("warrior.png", include_bytes!("../../Assets/Characters/warrior.png")),
    ("mage.png", include_bytes!("../../Assets/Characters/mage.png")),
    ("rogue.png", include_bytes!("../../Assets/Characters/rogue.png")),
    ("ranger.png", include_bytes!("../../Assets/Characters/ranger.png")),
    ("rat.png", include_bytes!("../../Assets/Characters/rat.png")),
    ("gnoll.png", include_bytes!("../../Assets/Characters/gnoll.png")),
    ("crab.png", include_bytes!("../../Assets/Characters/crab.png")),
    ("swarm.png", include_bytes!("../../Assets/Characters/swarm.png")),
    ("skeleton.png", include_bytes!("../../Assets/Characters/skeleton.png")),
    ("thief.png", include_bytes!("../../Assets/Characters/thief.png")),
];

/// Esquina del sprite parado en su celda: centrado en x, apoyado abajo (CharSprite.worldToCamera).
pub fn sprite_pos(pos: i32, w: u32, h: u32) -> [f32; 2] {
    [(pos % WIDTH) as f32 * TILE + (TILE - w as f32) / 2.0, (pos / WIDTH + 1) as f32 * TILE - h as f32]
}

fn tone_color(t: Tone) -> [f32; 4] {
    match t {
        Tone::Default => [1.0, 1.0, 1.0, 1.0],
        Tone::Pos => [0.0, 1.0, 0.0, 1.0],
        Tone::Neg => [1.0, 0x44 as f32 / 255.0, 0x44 as f32 / 255.0, 1.0],
        Tone::Warn => [1.0, 0x88 as f32 / 255.0, 0.0, 1.0],
        Tone::Neutral => [1.0, 1.0, 0.0, 1.0],
    }
}

struct AnimDef {
    fps: f32,
    frames: Vec<u32>,
    looped: bool,
}

/// La animación `name`: la del héroe (stats None) o la de `anims` del mob (sin "anims": el primer frame quieto).
/// None = ese personaje no la tiene.
fn anim_def(stats: Option<&Value>, name: &str) -> Option<AnimDef> {
    let Some(s) = stats else {
        return HERO_ANIMS.iter().find(|a| a.0 == name).map(|a| AnimDef { fps: a.1, frames: a.2.to_vec(), looped: a.3 });
    };
    let Some(anims) = s.get("anims") else {
        return (name == "idle").then(|| AnimDef { fps: 1.0, frames: vec![0], looped: true });
    };
    let e = anims.get(name)?;
    let frames: Vec<u32> = e[1].as_array()?.iter().filter_map(|f| f.as_u64().map(|f| f as u32)).collect();
    Some(AnimDef { fps: e[0].as_f64()? as f32, frames, looped: e[2].as_bool().unwrap_or(false) })
}

/// Animación actual de un personaje (CharSprite.play). flip = mira a la izquierda (CharSprite.turnTo).
#[derive(Clone)]
struct Anim {
    name: &'static str,
    t: f32,
    flip: bool,
}

impl Anim {
    fn new(name: &'static str, flip: bool) -> Self {
        Self { name, t: 0.0, flip }
    }

    fn play(&mut self, name: &'static str) {
        self.name = name;
        self.t = 0.0;
    }

    fn current(&self, stats: Option<&Value>) -> Option<AnimDef> {
        anim_def(stats, self.name).or_else(|| anim_def(stats, "idle")).filter(|d| !d.frames.is_empty() && d.fps > 0.0)
    }

    fn duration(&self, stats: Option<&Value>) -> f32 {
        self.current(stats).map_or(0.0, |d| d.frames.len() as f32 / d.fps)
    }

    fn done(&self, stats: Option<&Value>) -> bool {
        match anim_def(stats, self.name) {
            None => true,
            Some(d) => !d.looped && d.fps > 0.0 && self.t >= d.frames.len() as f32 / d.fps,
        }
    }

    fn index(&self, stats: Option<&Value>) -> u32 {
        let Some(d) = self.current(stats) else { return 0 };
        let i = (self.t * d.fps) as usize;
        if d.looped { d.frames[i % d.frames.len()] } else { d.frames[i.min(d.frames.len() - 1)] }
    }
}

/// Cómo se dibuja un personaje: hoja del atlas (None = el nombre), tamaño del frame, fila (héroe: la de su armadura)
/// y datos del mob (None = héroe, usa HERO_ANIMS).
#[derive(Clone)]
pub struct Look {
    pub sheet: Option<usize>,
    pub w: u32,
    pub h: u32,
    row: Option<u32>,
    stats: Option<Value>,
    pub name: String,
}

fn sheet_of(file: Option<&str>, first: usize) -> Option<usize> {
    file.and_then(|s| CHAR_SHEETS.iter().position(|c| c.0 == s)).map(|i| i + first)
}

fn mob_look(stats: &Value, name: String, first: usize) -> Look {
    let f = |k: usize| stats["frame"][k].as_u64().unwrap_or(16) as u32;
    Look { sheet: sheet_of(stats["sprite"].as_str(), first), w: f(0), h: f(1), row: None, stats: Some(stats.clone()), name }
}

pub fn look(d: &Dungeon, a: ActorId, first: usize) -> Option<Look> {
    match a {
        ActorId::Hero(i) => {
            let h = &d.heroes[i];
            let tier = h.armor.as_ref().map_or(0, |a| a.int("tier")) as u32;
            let sheet = sheet_of(h.cls["sprite"].as_str(), first);
            Some(Look { sheet, w: HERO_W, h: HERO_H, row: Some(tier), stats: None, name: h.ch.name.clone() })
        }
        ActorId::Mob(id) => d.mob(id).map(|m| mob_look(&m.stats, m.ch.name.clone(), first)),
        _ => None,
    }
}

struct Corpse {
    look: Look,
    pos: [f32; 2],
    cell: i32,
    anim: Anim,
}

struct Float {
    who: ActorId,
    text: String,
    color: [f32; 4],
    t: f32,
    row: f32,
    pos: [f32; 2],
    w: u32,
}

struct Leaf {
    p: [f32; 2],
    v: [f32; 2],
    life: f32,
    color: [f32; 4],
}

/// Los personajes del piso local en pantalla y sus efectos.
pub struct Chars {
    /// Índice en el atlas de la primera hoja de `CHAR_SHEETS`.
    first: usize,
    /// Dónde se dibuja cada personaje (pixeles del mundo).
    pub draw: Vec<(ActorId, [f32; 2])>,
    /// Pasos que a cada sprite le faltan recorrer (esquinas de sprite de cada celda).
    paths: Vec<(ActorId, VecDeque<[f32; 2]>)>,
    anims: Vec<(ActorId, Anim)>,
    corpses: Vec<Corpse>,
    floats: Vec<Float>,
    checked: Vec<(i32, f32)>,
    leaves: Vec<Leaf>,
    /// Hoja de icons.png en el atlas.
    icons: usize,
    /// Estado de cada mob en el frame anterior y los ! que se están viendo.
    emo_states: Vec<(u32, State)>,
    alerts: Vec<(u32, f32)>,
    emo_t: f32,
    rng: Rng,
}

impl Chars {
    pub fn new(first: usize, icons: usize, seed: i64) -> Self {
        Self {
            first,
            draw: Vec::new(),
            paths: Vec::new(),
            anims: Vec::new(),
            corpses: Vec::new(),
            floats: Vec::new(),
            checked: Vec::new(),
            leaves: Vec::new(),
            icons,
            emo_states: Vec::new(),
            alerts: Vec::new(),
            emo_t: 0.0,
            rng: Rng::new(seed),
        }
    }

    /// Piso o partida nueva: todos aparecen en su celda, sin deslizarse ni efectos viejos.
    pub fn clear(&mut self) {
        self.draw.clear();
        self.paths.clear();
        self.anims.clear();
        self.corpses.clear();
        self.floats.clear();
        self.checked.clear();
        self.leaves.clear();
        self.emo_states.clear();
        self.alerts.clear();
    }

    pub fn look(&self, d: &Dungeon, a: ActorId) -> Option<Look> {
        look(d, a, self.first)
    }

    fn drawn(&self, a: ActorId) -> Option<[f32; 2]> {
        self.draw.iter().find(|e| e.0 == a).map(|e| e.1)
    }

    /// Dónde está el sprite del héroe local (la cámara lo sigue).
    pub fn hero_sprite(&self, d: &Dungeon) -> [f32; 2] {
        self.drawn(ActorId::Hero(d.hero)).unwrap_or_else(|| sprite_pos(d.local().ch.pos.max(0), HERO_W, HERO_H))
    }

    /// Los sprites viajan a su celda a 1 casilla cada MOVE_TIME; cada héroe del piso se suelta (`animating`) al
    /// llegar (los de otro piso no se dibujan: no esperan).
    pub fn animate(&mut self, d: &mut Dungeon, dt: f32) {
        let budget = TILE / MOVE_TIME * dt;
        let mut next = Vec::new();
        let mut paths = Vec::new();
        for (a, pos) in d.chars() {
            let Some(lk) = self.look(d, a) else { continue };
            let trail = d.ch_mut(a).map(|c| std::mem::take(&mut c.trail)).unwrap_or_default();
            if pos < 0 {
                continue;
            }
            let goal = sprite_pos(pos, lk.w, lk.h);
            let mut path = self.paths.iter().position(|e| e.0 == a).map(|i| self.paths.swap_remove(i).1).unwrap_or_default();
            let p = match self.drawn(a) {
                Some(mut p) if d.level().visible[pos as usize] => {
                    if trail.is_empty() && path.is_empty() && (goal[0] - p[0]).abs().max((goal[1] - p[1]).abs()) > TILE * 1.5 {
                        p = goal; // llegó sin caminar (teletransporte): aparece ahí
                    }
                    // recorre los pasos uno por uno; lo que sobra del frame sigue con el próximo
                    path.extend(trail.iter().map(|&c| sprite_pos(c, lk.w, lk.h)));
                    let mut left = budget;
                    loop {
                        let t = path.front().copied().unwrap_or(goal);
                        let dist = (t[0] - p[0]).abs().max((t[1] - p[1]).abs());
                        if dist > left {
                            p = [p[0] + (t[0] - p[0]).clamp(-left, left), p[1] + (t[1] - p[1]).clamp(-left, left)];
                            break;
                        }
                        p = t;
                        left -= dist;
                        if path.pop_front().is_none() {
                            break;
                        }
                    }
                    p
                }
                _ => {
                    path.clear(); // fuera de la vista: aparece en su celda
                    goal
                }
            };
            if !path.is_empty() {
                paths.push((a, path));
            }
            next.push((a, p));
        }
        self.draw = next;
        self.paths = paths;
        for i in 0..d.heroes.len() {
            let h = &d.heroes[i];
            let rest = sprite_pos(h.ch.pos, HERO_W, HERO_H);
            let walking = self.paths.iter().any(|e| e.0 == ActorId::Hero(i));
            if h.depth != d.depth || (!walking && self.drawn(ActorId::Hero(i)).is_none_or(|p| p == rest)) {
                d.heroes[i].animating = false;
            }
        }
    }

    fn anim_mut(&mut self, a: ActorId) -> &mut Anim {
        let i = match self.anims.iter().position(|e| e.0 == a) {
            Some(i) => i,
            None => {
                self.anims.push((a, Anim::new("idle", false)));
                self.anims.len() - 1
            }
        };
        &mut self.anims[i].1
    }

    /// Consume `d.fx` (animaciones, muertes, textos flotantes, casillas revisadas, hojas).
    pub fn consume(&mut self, d: &mut Dungeon) {
        for e in std::mem::take(&mut d.fx) {
            match e {
                Fx::Anim { who, kind, cell } => {
                    // ataque / operar: se gira hacia la celda (CharSprite.attack / operate)
                    let pos = d.ch(who).map_or(cell, |c| c.pos);
                    let a = self.anim_mut(who);
                    if cell % WIDTH != pos % WIDTH {
                        a.flip = cell % WIDTH < pos % WIDTH;
                    }
                    a.play(kind);
                }
                Fx::Die { who, cell, stats } => {
                    let look = mob_look(&stats, String::new(), self.first);
                    let pos = self.drawn(who).unwrap_or_else(|| sprite_pos(cell, look.w, look.h));
                    let flip = self.anims.iter().find(|e| e.0 == who).is_some_and(|e| e.1.flip);
                    self.corpses.push(Corpse { look, pos, cell, anim: Anim::new("die", flip) });
                }
                Fx::Status { who, text, tone } => {
                    let row = self.floats.iter().filter(|f| f.who == who).count() as f32;
                    let Some(lk) = self.look(d, who) else { continue };
                    let pos = self.drawn(who).unwrap_or_else(|| sprite_pos(d.ch(who).map_or(0, |c| c.pos), lk.w, lk.h));
                    self.floats.push(Float { who, text, color: tone_color(tone), t: 0.0, row, pos, w: lk.w });
                }
                Fx::Checked(cell) => self.checked.push((cell, CHECKED_TIME)),
                Fx::Leaves(cell) => {
                    let (x, y) = ((cell % WIDTH) as f32 * TILE, (cell / WIDTH) as f32 * TILE);
                    for _ in 0..4 {
                        let r = &mut self.rng;
                        let p = [x + r.float_range(2.0, 14.0) as f32, y + r.float_range(4.0, 14.0) as f32];
                        let v = [r.float_range(-8.0, 8.0) as f32, r.float_range(-25.0, -10.0) as f32];
                        let color = LEAF_COLORS[r.int(2) as usize];
                        self.leaves.push(Leaf { p, v, life: 1.0, color });
                    }
                }
            }
        }
    }

    /// Avanza las animaciones (charSprite.Update) y los efectos.
    pub fn update(&mut self, d: &Dungeon, dt: f32) {
        let chars = d.chars();
        self.anims.retain(|e| chars.iter().any(|c| c.0 == e.0));
        for (a, pos) in chars {
            let Some(lk) = self.look(d, a) else { continue };
            let stats = lk.stats.as_ref();
            let drawn = self.drawn(a);
            let lying = !d.alive(a) || matches!(a, ActorId::Hero(i) if d.heroes[i].is_downed());
            let frozen = d.alive(a) && d.has_flag(a, "paralysed"); // CharSprite.paralyse: la animación se congela
            let an = self.anim_mut(a);
            if frozen {
                continue;
            }
            an.t += dt;
            if lying {
                if an.name != "die" {
                    an.play("die"); // caído a 0 PV: tendido (queda en el último frame de morir)
                }
                continue;
            }
            if ONESHOT.contains(&an.name) && !an.done(stats) {
                continue;
            }
            let target = sprite_pos(pos, lk.w, lk.h);
            match drawn {
                Some(p) if p != target => {
                    if p[0] != target[0] {
                        an.flip = target[0] < p[0];
                    }
                    if an.name != "run" {
                        an.play("run");
                    }
                }
                _ => {
                    if an.name != "idle" {
                        an.play("idle");
                    }
                }
            }
        }
        for f in &mut self.floats {
            f.t += dt;
            if let Some(p) = self.draw.iter().find(|e| e.0 == f.who) {
                f.pos = p.1;
            }
        }
        self.floats.retain(|f| f.t < FLOAT_TIME);
        for c in &mut self.checked {
            c.1 -= dt;
        }
        self.checked.retain(|c| c.1 > 0.0);
        for l in &mut self.leaves {
            l.p[0] += l.v[0] * dt;
            l.p[1] += l.v[1] * dt;
            l.v[1] += 40.0 * dt;
            l.life -= dt;
        }
        self.leaves.retain(|l| l.life > 0.0);
        for c in &mut self.corpses {
            c.anim.t += dt;
        }
        self.corpses.retain(|c| corpse_alpha(c) > 0.0);
        // un mob que se despierta o pasa a cazarte muestra el ! durante ALERT_TIME
        self.emo_t += dt;
        let mobs = &d.floor().mobs;
        for m in mobs {
            let prev = self.emo_states.iter().find(|e| e.0 == m.id).map(|e| e.1);
            if let Some(prev) = prev
                && prev != m.state
                && m.state != State::Sleeping
                && (prev == State::Sleeping || m.state == State::Hunting)
            {
                self.alerts.retain(|a| a.0 != m.id);
                self.alerts.push((m.id, ALERT_TIME));
            }
        }
        self.emo_states = mobs.iter().map(|m| (m.id, m.state)).collect();
        for a in &mut self.alerts {
            a.1 -= dt;
        }
        self.alerts.retain(|a| a.1 > 0.0);
    }

    /// El primer frame del personaje achicado para entrar en `max` x `max`, centrado en `center` (carrusel de
    /// iniciativa, ventana de examinar).
    pub fn portrait(&self, d: &Dungeon, atlas: &Atlas, a: ActorId, center: [f32; 2], max: f32) -> Option<Sprite> {
        let lk = self.look(d, a)?;
        let [sx, sy, _, _] = atlas.rects[lk.sheet?];
        let y = lk.row.unwrap_or(0) * lk.h;
        let f = (max / lk.w.max(lk.h) as f32).min(1.0);
        let (w, h) = ((lk.w as f32 * f).floor().max(1.0), (lk.h as f32 * f).floor().max(1.0));
        let mut s = atlas.sprite(sx, sy + y, lk.w, lk.h, [(center[0] - w / 2.0).floor(), (center[1] - h / 2.0).floor()], [1.0; 4]);
        s.size = [w, h];
        Some(s)
    }

    /// Casillas revisadas, cadáveres y personajes (debajo de la niebla). Los que no tienen hoja se dibujan en
    /// `draw_over` (su nombre).
    pub fn draw_world(&self, d: &Dungeon, atlas: &Atlas, out: &mut Vec<Sprite>, ox: f32, oy: f32) {
        let level = d.level();
        for &(cell, a) in &self.checked {
            let s = (TILE * a).floor().max(1.0);
            let (cx, cy) = ((cell % WIDTH) as f32 * TILE + TILE / 2.0, (cell / WIDTH) as f32 * TILE + TILE / 2.0);
            let [r, g, b] = CHECKED_COLOR;
            out.push(atlas.rect([ox + cx - (s / 2.0).floor(), oy + cy - (s / 2.0).floor()], [s, s], [r, g, b, a]));
        }
        for c in &self.corpses {
            if level.visible[c.cell as usize] {
                let alpha = corpse_alpha(c);
                frame(atlas, out, &c.look, &c.anim, [ox + c.pos[0], oy + c.pos[1]], alpha);
            }
        }
        for &(a, p) in &self.draw {
            let pos = d.ch(a).map_or(-1, |c| c.pos);
            if pos < 0 || !(a == ActorId::Hero(d.hero) || level.visible[pos as usize]) {
                continue;
            }
            let (Some(lk), Some(an)) = (self.look(d, a), self.anims.iter().find(|e| e.0 == a)) else { continue };
            frame(atlas, out, &lk, &an.1, [ox + p[0], oy + p[1]], 1.0);
        }
    }

    /// Encima de la niebla: hojas, textos flotantes y el nombre de los personajes sin hoja.
    pub fn draw_over(&self, d: &Dungeon, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, ox: f32, oy: f32) {
        for l in &self.leaves {
            out.push(atlas.rect([ox + l.p[0].floor(), oy + l.p[1].floor()], [1.0, 1.0], l.color));
        }
        for &(a, p) in &self.draw {
            let pos = d.ch(a).map_or(-1, |c| c.pos);
            let Some(lk) = self.look(d, a) else { continue };
            if lk.sheet.is_none() && pos >= 0 && d.level().visible[pos as usize] {
                let w = font.width(&lk.name);
                let x = (ox + p[0] + lk.w as f32 / 2.0 - w / 2.0).floor();
                font.text(out, &lk.name, x, (oy + p[1]).floor(), [1.0, 1.0, 1.0, 1.0]);
            }
        }
        let [ix, iy, _, _] = atlas.rects[self.icons];
        for m in &d.floor().mobs {
            if !d.level().visible[m.ch.pos as usize] {
                continue;
            }
            let (region, (max, rate)) = if m.state == State::Sleeping {
                (ICON_SLEEP, EMO_SLEEP)
            } else if self.alerts.iter().any(|a| a.0 == m.id) {
                (ICON_ALERT, EMO_ALERT)
            } else {
                continue;
            };
            let phase = (m.id % 7) as f32 * 0.57; // cada mob en su fase: que no pulsen todos juntos
            let s = pulse(self.emo_t + phase, max, rate);
            let (w, h) = ((region[2] as f32 * s).round(), (region[3] as f32 * s).round());
            let Some(p) = self.drawn(ActorId::Mob(m.id)) else { continue };
            let fw = self.look(d, ActorId::Mob(m.id)).map_or(16, |l| l.w) as f32;
            let pos = [(ox + p[0] + fw - w / 2.0).floor(), (oy + p[1] - h).floor()];
            let mut sp = atlas.sprite(ix + region[0], iy + region[1], region[2], region[3], pos, [1.0; 4]);
            sp.size = [w, h];
            out.push(sp);
        }
        for f in &self.floats {
            let mut color = f.color;
            color[3] = 1.0 - f.t / FLOAT_TIME;
            let w = font.width(&f.text);
            let x = (ox + f.pos[0] + f.w as f32 / 2.0 - w / 2.0).floor();
            let y = (oy + f.pos[1] - font.height() - FLOAT_RISE * f.t / FLOAT_TIME - FLOAT_STACK * f.row).floor();
            font.text(out, &f.text, x, y, color);
        }
    }
}

/// Escala que va y viene entre 1 y `max` (EmoIcon.update).
fn pulse(t: f32, max: f32, rate: f32) -> f32 {
    let span = max - 1.0;
    let p = (t * rate) % (2.0 * span);
    1.0 + if p < span { p } else { 2.0 * span - p }
}

/// 1 mientras se ve la animación de muerte; después baja a 0 en FADE_TIME.
fn corpse_alpha(c: &Corpse) -> f32 {
    (1.0 - (c.anim.t - c.anim.duration(c.look.stats.as_ref())).max(0.0) / FADE_TIME).max(0.0)
}

/// El frame actual del personaje. Héroe: la fila de su armadura; mob: los frames van de izquierda a derecha y bajan
/// de fila (TextureFilm). flip = espejado en x.
fn frame(atlas: &Atlas, out: &mut Vec<Sprite>, lk: &Look, an: &Anim, pos: [f32; 2], alpha: f32) {
    let Some(sheet) = lk.sheet else { return };
    let [sx, sy, sw, _] = atlas.rects[sheet];
    let i = an.index(lk.stats.as_ref());
    let (x, y) = match lk.row {
        Some(row) => (i * lk.w, row * lk.h),
        None => {
            let cols = (sw / lk.w).max(1);
            ((i % cols) * lk.w, (i / cols) * lk.h)
        }
    };
    let mut s = atlas.sprite(sx + x, sy + y, lk.w, lk.h, [pos[0].floor(), pos[1].floor()], [1.0, 1.0, 1.0, alpha]);
    if an.flip {
        s.uv.swap(0, 2);
    }
    out.push(s);
}
