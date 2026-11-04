//! Ventana (winit), bucle del juego y render (wgpu). Lee el estado de `game`, nunca al revés.
//! R2: recorrer un piso con un observador (adelanto del héroe, como en la fase 2 del prototipo).
mod atlas;
mod font;
mod gpu;
mod map;

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use game::fov::MAX_DISTANCE;
use game::geom::{HEIGHT, WIDTH};
use game::level::Level;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use atlas::Atlas;
use font::Font;
use gpu::{Gpu, Sprite, Tex};
use map::TILE;

pub const TITLE: &str = "PixelDungeon-RollInitiative";
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.27, 1.0];
const GREY: [f32; 4] = [0.53, 0.53, 0.53, 1.0];
const MOVE_TIME: f32 = 0.1; // segundos por casilla (CharSprite.MOVE_INTERVAL)
const DRAG_MIN_PX: f64 = 8.0; // arrastrar más que esto (pixeles de pantalla) mueve la cámara; menos es un click
const PINCH_STEP: f64 = 1.25; // cuánto tiene que cambiar la distancia entre dedos para un paso de zoom
const HERO_W: f32 = 12.0;
const HERO_H: f32 = 15.0;
const MAX_DEPTH: i32 = 4;

/// Hojas del atlas, en orden: los índices de `Atlas::rects`. Van dentro del binario (Android sin AssetManager).
const SHEETS: [&[u8]; 4] = [
    include_bytes!("../../Assets/Fonts/font1x.png"),
    include_bytes!("../../Assets/Characters/warrior.png"),
    include_bytes!("../../Assets/Tiles/tiles0.png"),
    include_bytes!("../../Assets/Tiles/water0.png"),
];
const FONT: usize = 0;
const WARRIOR: usize = 1;
const TILES: usize = 2;
const WATER: usize = 3;

/// El piso y quien lo recorre. El observador se reemplaza por el héroe en R3.
struct World {
    seed: i64,
    depth: i32,
    level: Level,
    pos: i32,
    sprite: [f32; 2], // dónde se dibuja (pixeles del mundo); viaja hacia su celda
    target: Option<i32>,
    fog_dirty: bool,
}

impl World {
    fn new(seed: i64, depth: i32) -> Self {
        let level = Level::new(seed, depth);
        let pos = level.entrance;
        let mut w = Self { seed, depth, level, pos, sprite: Self::sprite_pos(pos), target: None, fog_dirty: true };
        w.level.observe(&[pos], MAX_DISTANCE);
        w
    }

    /// Esquina del sprite parado en su celda: centrado en x, apoyado abajo (CharSprite.worldToCamera).
    fn sprite_pos(pos: i32) -> [f32; 2] {
        [(pos % WIDTH) as f32 * TILE + (TILE - HERO_W) / 2.0, (pos / WIDTH + 1) as f32 * TILE - HERO_H]
    }

    fn animating(&self) -> bool {
        self.sprite != Self::sprite_pos(self.pos)
    }

    /// Un paso: abre y cierra puertas al pasar (Door.enter/leave) y recalcula la vista.
    fn step(&mut self, dx: i32, dy: i32) -> bool {
        let to = self.pos + dx + dy * WIDTH;
        if !(0..WIDTH * HEIGHT).contains(&to) || !self.level.passable[to as usize] {
            return false;
        }
        self.level.leave(self.pos);
        self.pos = to;
        self.level.enter(to);
        self.level.observe(&[to], MAX_DISTANCE);
        self.fog_dirty = true;
        true
    }

    /// Hacia el destino tocado: diagonal si se puede, si no por un eje. Sin pathfinder todavía (R3).
    fn walk_to_target(&mut self) {
        let Some(t) = self.target else { return };
        let (dx, dy) = (((t % WIDTH) - (self.pos % WIDTH)).signum(), ((t / WIDTH) - (self.pos / WIDTH)).signum());
        if (dx, dy) == (0, 0) || !(self.step(dx, dy) || (dx != 0 && self.step(dx, 0)) || (dy != 0 && self.step(0, dy))) {
            self.target = None;
        }
    }

    fn update(&mut self, dt: f32) {
        let goal = Self::sprite_pos(self.pos);
        let max = TILE / MOVE_TIME * dt;
        for k in 0..2 {
            let d = goal[k] - self.sprite[k];
            self.sprite[k] += d.clamp(-max, max);
        }
    }
}

#[derive(Default)]
struct Input {
    held: [bool; 4], // arriba, abajo, izquierda, derecha
    cursor: PhysicalPosition<f64>,
    press: Option<(PhysicalPosition<f64>, [f32; 2])>, // (dónde se apretó, pan al apretar)
    dragging: bool,
    touches: Vec<(u64, PhysicalPosition<f64>)>,
    pinch: Option<f64>, // distancia entre dedos del último paso de zoom
}

struct App {
    atlas: Atlas,
    font: Font,
    gpu: Option<Gpu>,
    sprites: Vec<Sprite>,
    world: World,
    input: Input,
    pan: [f32; 2],      // desplazamiento de la cámara al arrastrar (pixeles del mundo)
    zoom: Option<u32>,  // pixeles de pantalla por pixel del mundo (None = el de la UI)
    water_off: f32,
    last: Instant,
    fps_since: Instant,
    frames: u32,
    fps: u32,
}

fn new_seed() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_micros() as i64 % 2_147_483_647)
}

impl App {
    fn new() -> Self {
        let atlas = Atlas::build(&SHEETS);
        let font = Font::new(&atlas, FONT);
        let now = Instant::now();
        Self {
            atlas,
            font,
            gpu: None,
            sprites: Vec::new(),
            world: World::new(new_seed(), 1),
            input: Input::default(),
            pan: [0.0; 2],
            zoom: None,
            water_off: 0.0,
            last: now,
            fps_since: now,
            frames: 0,
            fps: 0,
        }
    }

    /// Rango de zooms de la cámara: como el prototipo, del zoom de la UI -2 a +2 (siempre entero: nítido).
    fn zoom_range(gpu: &Gpu) -> (u32, u32) {
        let z = gpu.ui_zoom();
        (z.saturating_sub(2).max(1), z + 2)
    }

    fn zoom_by(&mut self, step: i32) {
        let Some(gpu) = self.gpu.as_ref() else { return };
        let (lo, hi) = Self::zoom_range(gpu);
        let cur = self.zoom.unwrap_or(gpu.ui_zoom()) as i32;
        self.zoom = Some((cur + step).clamp(lo as i32, hi as i32) as u32);
    }

    /// Origen del mapa en la capa del mundo: la cámara centrada en el sprite del observador, más el pan.
    fn map_origin(&self, gpu: &Gpu) -> (f32, f32) {
        let (vw, vh, _, _) = gpu.world_view();
        let s = self.world.sprite;
        (
            (vw as f32 / 2.0 - (s[0] + HERO_W / 2.0) + self.pan[0]).floor(),
            (vh as f32 / 2.0 - (s[1] + HERO_H / 2.0) + self.pan[1]).floor(),
        )
    }

    /// Celda bajo un punto de la ventana.
    fn cell_at(&self, p: PhysicalPosition<f64>) -> Option<i32> {
        let gpu = self.gpu.as_ref()?;
        let (_, _, (offx, offy), scale) = gpu.world_view();
        let (ox, oy) = self.map_origin(gpu);
        let wx = ((p.x as f32 - offx as f32) / scale as f32 - ox) / TILE;
        let wy = ((p.y as f32 - offy as f32) / scale as f32 - oy) / TILE;
        let (x, y) = (wx.floor() as i32, wy.floor() as i32);
        ((0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y)).then_some(x + y * WIDTH)
    }

    fn press(&mut self, p: PhysicalPosition<f64>) {
        self.input.press = Some((p, self.pan));
        self.input.dragging = false;
    }

    fn drag(&mut self, p: PhysicalPosition<f64>) {
        let Some((start, pan0)) = self.input.press else { return };
        let (dx, dy) = (p.x - start.x, p.y - start.y);
        if self.input.dragging || dx.abs() + dy.abs() >= DRAG_MIN_PX {
            self.input.dragging = true;
            let scale = self.gpu.as_ref().map_or(1, |g| g.world_view().3) as f32;
            self.pan = [pan0[0] + dx as f32 / scale, pan0[1] + dy as f32 / scale];
        }
    }

    /// Soltar sin arrastrar = click / toque: caminar hacia esa celda.
    fn release(&mut self, p: PhysicalPosition<f64>) {
        if self.input.press.take().is_some() && !self.input.dragging {
            self.world.target = self.cell_at(p);
        }
    }

    fn touch(&mut self, id: u64, phase: TouchPhase, p: PhysicalPosition<f64>) {
        let t = &mut self.input.touches;
        match phase {
            TouchPhase::Started => {
                t.push((id, p));
                if t.len() == 1 {
                    self.press(p);
                } else {
                    self.input.press = None; // dos dedos: pellizco, no arrastre
                    self.input.pinch = Some(Self::spread(&self.input.touches));
                }
            }
            TouchPhase::Moved => {
                if let Some(e) = t.iter_mut().find(|(i, _)| *i == id) {
                    e.1 = p;
                }
                if let Some(base) = self.input.pinch {
                    let d = Self::spread(&self.input.touches);
                    if d > base * PINCH_STEP {
                        self.zoom_by(1);
                        self.input.pinch = Some(d);
                    } else if d < base / PINCH_STEP {
                        self.zoom_by(-1);
                        self.input.pinch = Some(d);
                    }
                } else {
                    self.drag(p);
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                t.retain(|(i, _)| *i != id);
                if self.input.pinch.is_none() && phase == TouchPhase::Ended {
                    self.release(p);
                }
                if self.input.touches.is_empty() {
                    self.input.pinch = None;
                    self.input.press = None;
                }
            }
        }
    }

    fn spread(t: &[(u64, PhysicalPosition<f64>)]) -> f64 {
        match t {
            [(_, a), (_, b), ..] => ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt().max(1.0),
            _ => 1.0,
        }
    }

    fn key(&mut self, code: KeyCode, down: bool) {
        let dir = match code {
            KeyCode::KeyW | KeyCode::ArrowUp => Some(0),
            KeyCode::KeyS | KeyCode::ArrowDown => Some(1),
            KeyCode::KeyA | KeyCode::ArrowLeft => Some(2),
            KeyCode::KeyD | KeyCode::ArrowRight => Some(3),
            _ => None,
        };
        if let Some(d) = dir {
            self.input.held[d] = down;
            return;
        }
        if !down {
            return;
        }
        match code {
            KeyCode::KeyR => self.new_world(new_seed(), self.world.depth),
            KeyCode::KeyN => self.new_world(self.world.seed, self.world.depth % MAX_DEPTH + 1),
            _ => {}
        }
    }

    fn new_world(&mut self, seed: i64, depth: i32) {
        self.world = World::new(seed, depth);
        self.pan = [0.0; 2];
    }

    fn frame(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        self.frames += 1;
        if (now - self.fps_since).as_secs_f32() >= 1.0 {
            self.fps = self.frames;
            self.frames = 0;
            self.fps_since = now;
            log::info!("FPS {}", self.fps);
        }

        // --- lógica: un paso cada vez que el observador llega a su celda ---
        let w = &mut self.world;
        if !w.animating() {
            let h = self.input.held;
            let (dx, dy) = ((h[3] as i32) - (h[2] as i32), (h[1] as i32) - (h[0] as i32));
            if (dx, dy) != (0, 0) {
                w.target = None;
                if !w.step(dx, dy) {
                    let _ = (dx != 0 && w.step(dx, 0)) || (dy != 0 && w.step(0, dy));
                }
            } else {
                w.walk_to_target();
            }
            if w.animating() && self.input.press.is_none() {
                self.pan = [0.0; 2]; // el observador camina: la cámara vuelve a seguirlo
            }
        }
        w.update(dt);
        self.water_off += map::WATER_SPEED * dt;

        let Some(gpu) = self.gpu.as_mut() else { return };
        let (lo, hi) = Self::zoom_range(gpu);
        gpu.set_world_scale(self.zoom.unwrap_or(gpu.ui_zoom()).clamp(lo, hi));
        if self.world.fog_dirty {
            gpu.set_fog(&map::fog_corners(&self.world.level));
            self.world.fog_dirty = false;
        }
        let gpu = self.gpu.as_ref().unwrap();
        let (ox, oy) = self.map_origin(gpu);

        // --- mundo: agua, tiles, observador | niebla ---
        let s = &mut self.sprites;
        s.clear();
        map::water(s, &self.atlas, self.atlas.rects[WATER], ox, oy, self.water_off);
        map::tiles(s, &self.atlas, self.atlas.rects[TILES], &self.world.level, ox, oy);
        let r = self.atlas.rects[WARRIOR];
        let hero = self.world.sprite;
        s.push(self.atlas.sprite(r[0], r[1], 12, 15, [ox + hero[0], oy + hero[1]], WHITE));
        let atlas_end = s.len() as u32;
        s.push(map::fog_quad(ox, oy));
        let world_end = s.len() as u32;

        // --- UI ---
        let (_, uh) = gpu.ui_size();
        let w = &self.world;
        self.font.text(s, &format!("FPS {}", self.fps), 4.0, 4.0, YELLOW);
        let info = format!("seed {}  piso {}  {}", w.seed, w.depth, w.level.feeling.name());
        self.font.text(s, &info, 4.0, 14.0, WHITE);
        let help = "WASD/toque: caminar  rueda/pellizco: zoom  arrastrar: camara  R/N: seed/piso";
        self.font.text(s, help, 4.0, uh as f32 - 12.0, GREY);
        let ui_end = s.len() as u32;

        let batches = [(Tex::Atlas, 0..atlas_end), (Tex::Fog, atlas_end..world_end)];
        self.gpu.as_mut().unwrap().draw(&self.sprites, &batches, world_end..ui_end);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        // Android destruye la superficie al irse al fondo: se recrea ventana y GPU en cada resumed
        let window = Arc::new(el.create_window(Window::default_attributes().with_title(TITLE)).expect("ventana"));
        self.gpu = Some(pollster::block_on(Gpu::new(window, &self.atlas)));
        self.world.fog_dirty = true; // la textura de niebla es nueva
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.gpu = None;
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => self.frame(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.key(code, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.cursor = position;
                self.drag(position);
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let p = self.input.cursor;
                if state == ElementState::Pressed { self.press(p) } else { self.release(p) }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(p) => p.y,
                };
                if y != 0.0 {
                    self.zoom_by(if y > 0.0 { 1 } else { -1 });
                }
            }
            WindowEvent::Touch(t) => self.touch(t.id, t.phase, t.location),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(gpu) = &self.gpu {
            gpu.window.request_redraw(); // sin tope: se dibuja tan rápido como presente la superficie
        }
    }
}

pub fn run(el: EventLoop<()>) {
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut App::new()).expect("bucle de eventos");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    run(EventLoop::builder().with_android_app(app).build().expect("bucle de eventos"));
}
