//! Ventana (winit), bucle del juego y render (wgpu). Lee el estado de `game`, nunca al revés.
//! R3: el héroe juega un piso con turnos, mobs y combate 5e. R4: personajes animados, dados y efectos.
mod atlas;
mod config;
mod dice_fx;
mod font;
mod gpu;
mod map;
mod sprites;
mod ui;

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use config::{Bind, Config};

use game::dungeon::Dungeon;
use game::geom::{HEIGHT, LENGTH, WIDTH};
use game::hero::Action;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use atlas::Atlas;
use dice_fx::DiceFx;
use font::Font;
use gpu::{Gpu, Sprite, Tex};
use map::TILE;
use sprites::{Chars, CHAR_SHEETS, HERO_H, HERO_W};
use game::hero::Verb;
use ui::{BagMode, Ui, UiAction, Window as UiWindow};

pub const TITLE: &str = "PixelDungeon-RollInitiative";
const DRAG_MIN_PX: f64 = 8.0; // arrastrar más que esto (pixeles de pantalla) mueve la cámara; menos es un click
const PINCH_STEP: f64 = 1.25; // cuánto tiene que cambiar la distancia entre dedos para un paso de zoom
/// Clases jugables (C cambia de clase y empieza otra partida). ponytail: hasta la creación de personaje (R5)
const CLASSES: [&str; 4] = ["pdri:warrior", "pdri:mage", "pdri:rogue", "pdri:huntress"];

/// Hojas del atlas, en orden: los índices de `Atlas::rects`. Van dentro del binario (Android sin AssetManager).
const SHEETS: [&[u8]; 15] = [
    include_bytes!("../../Assets/Fonts/font1x.png"),
    include_bytes!("../../Assets/Tiles/tiles0.png"),
    include_bytes!("../../Assets/Tiles/water0.png"),
    include_bytes!("../../Assets/chrome.png"),
    include_bytes!("../../Assets/status_pane.png"),
    include_bytes!("../../Assets/toolbar.png"),
    include_bytes!("../../Assets/hp_bar.png"),
    include_bytes!("../../Assets/exp_bar.png"),
    include_bytes!("../../Assets/icons.png"),
    include_bytes!("../../Assets/Items/items.png"),
    include_bytes!("../../Assets/buffs.png"),
    include_bytes!("../../Assets/arcs1.png"),
    include_bytes!("../../Assets/arcs2.png"),
    include_bytes!("../../Assets/banners.png"),
    include_bytes!("../../Assets/dashboard.png"),
];
const FONT: usize = 0;
pub(crate) const TILES: usize = 1;
pub(crate) const WATER: usize = 2;
pub(crate) const CHROME: usize = 3;
pub(crate) const STATUS: usize = 4;
pub(crate) const TOOLBAR: usize = 5;
pub(crate) const HP_BAR: usize = 6;
pub(crate) const EXP_BAR: usize = 7;
pub(crate) const ICONS: usize = 8;
pub(crate) const ITEMS: usize = 9;
pub(crate) const BUFFS: usize = 10;
pub(crate) const ARCS_BG: usize = 11;
pub(crate) const ARCS_FG: usize = 12;
pub(crate) const BANNERS: usize = 13;
pub(crate) const DASHBOARD: usize = 14;

/// La partida, sus personajes en pantalla y la animación de dados.
struct World {
    d: Dungeon,
    class: usize,
    chars: Chars,
    dice: DiceFx,
    depth: i32,
}

impl World {
    fn new(seed: i64, class: usize) -> Self {
        let d = Dungeon::new(seed, CLASSES[class]);
        let depth = d.depth;
        // el azar visual va aparte del rng de la partida
        let chars = Chars::new(SHEETS.len(), ICONS, seed ^ 0x5EED);
        Self { d, class, chars, dice: DiceFx::new(seed ^ 0xD1CE), depth }
    }

    /// Un frame de lógica, en el orden del prototipo: los sprites caminan (y sueltan al héroe al llegar), lo que el
    /// dado ya mostró se aplica, corren los turnos, y después los efectos y los dados avanzan.
    fn update(&mut self, dt: f32) {
        self.chars.animate(&mut self.d, dt);
        self.dice.feed(&mut self.d.dice_events);
        if !self.d.pending.is_empty() && !self.dice.busy() {
            self.d.resolve_pending(); // recién cuando el dado mostró el resultado se aplica el daño
        }
        self.d.process();
        if self.d.depth != self.depth {
            self.depth = self.d.depth;
            self.chars.clear(); // otro piso: todos aparecen en su celda
        }
        self.chars.consume(&mut self.d);
        self.chars.update(&self.d, dt);
        self.dice.feed(&mut self.d.dice_events);
        self.dice.update(dt);
    }

    fn hero_sprite(&self) -> [f32; 2] {
        self.chars.hero_sprite(&self.d)
    }

    /// Paso con las teclas: diagonal si se puede, si no por un eje (atacar si hay alguien).
    fn step(&mut self, dx: i32, dy: i32) {
        let me = self.d.hero;
        let pos = self.d.heroes[me].ch.pos;
        let open = |c: i32| (0..LENGTH).contains(&c) && (self.d.level().passable[c as usize] || self.d.find_char(c).is_some());
        let tries = [(dx, dy), (dx, 0), (0, dy)];
        if let Some(c) = tries.iter().filter(|t| **t != (0, 0)).map(|(x, y)| pos + x + y * WIDTH).find(|&c| open(c)) {
            self.d.handle(me, c, false);
        }
    }

    /// Lo que puede hacer el jugador ahora (vivo, sin acción en curso y sin animación pendiente).
    fn ready(&self) -> bool {
        let h = self.d.local();
        h.is_alive() && h.action.is_none() && !h.animating
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
    ui: Ui,
    config: Config,
    /// En la pantalla de título (la partida de atrás no avanza).
    title: bool,
    /// Nombres de las clases jugables (data/classes.json), para elegir en el título.
    class_names: Vec<String>,
    quit: bool,
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
        let sheets: Vec<&[u8]> = SHEETS.iter().copied().chain(CHAR_SHEETS.iter().map(|c| c.1)).collect();
        let atlas = Atlas::build(&sheets);
        let font = Font::new(&atlas, FONT);
        let now = Instant::now();
        Self {
            atlas,
            font,
            gpu: None,
            sprites: Vec::new(),
            world: World::new(new_seed(), 0),
            ui: Ui::new(),
            config: Config::load(),
            title: true,
            class_names: {
                let c = game::registry::load("classes").unwrap_or_default();
                CLASSES.iter().map(|k| c.get(*k).map_or(k.to_string(), |v| game::registry::text(&v["name"]))).collect()
            },
            quit: false,
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

    /// Origen del mapa en la capa del mundo: la cámara centrada en el sprite del héroe local, más el pan.
    fn map_origin(&self, gpu: &Gpu) -> (f32, f32) {
        let (vw, vh, _, _) = gpu.world_view();
        let s = self.world.hero_sprite();
        (
            (vw as f32 / 2.0 - (s[0] + HERO_W as f32 / 2.0) + self.pan[0]).floor(),
            (vh as f32 / 2.0 - (s[1] + HERO_H as f32 / 2.0) + self.pan[1]).floor(),
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

    /// Un punto de la ventana en pixeles de la capa de UI.
    fn ui_point(&self, p: PhysicalPosition<f64>) -> Option<[f32; 2]> {
        let ((offx, offy), scale) = self.gpu.as_ref()?.ui_view();
        Some([(p.x as f32 - offx as f32) / scale as f32, (p.y as f32 - offy as f32) / scale as f32])
    }

    /// Soltar sin arrastrar = click / toque. Primero la UI (de arriba hacia abajo); si no la toca, el mapa: examinar
    /// la casilla o Hero.handle (caminar, atacar, ayudar o cambiar de piso). Sobre el propio héroe = buscar.
    fn release(&mut self, p: PhysicalPosition<f64>) {
        if self.input.press.take().is_none() || self.input.dragging {
            return;
        }
        if let Some(act) = self.ui_point(p).and_then(|q| self.ui.click(q, false)) {
            self.ui_action(act);
            return;
        }
        if self.title {
            return; // el título no tiene mapa
        }
        let Some(cell) = self.cell_at(p) else { return };
        if self.ui.examining {
            self.ui.examining = false;
            self.ui.examine(&self.world.d, cell);
            return;
        }
        if let Some(k) = self.ui.casting.take() {
            self.ui_action(UiAction::Act(Action::Cast(k, cell)));
            return;
        }
        let d = &mut self.world.d;
        if !d.local().is_alive() {
            return;
        }
        let me = d.hero;
        if cell == d.local().ch.pos {
            d.heroes[me].action = Some(Action::Search);
        } else {
            d.handle(me, cell, true);
        }
    }

    fn ui_action(&mut self, act: UiAction) {
        match act {
            UiAction::Nothing => {}
            UiAction::Close => self.ui.window = None,
            UiAction::Open(w) => self.ui.window = Some(w),
            UiAction::Act(a) => {
                let d = &mut self.world.d;
                if d.local().is_alive() {
                    let me = d.hero;
                    d.heroes[me].action = Some(a);
                }
            }
            UiAction::Examine => self.ui.examining = !self.ui.examining,
            UiAction::NewGame => self.new_world(new_seed(), self.world.class),
            UiAction::Quit => self.quit = true,
            UiAction::SetFps(f) => self.config.set_fps(f),
            UiAction::SetFullscreen(on) => {
                self.config.set_fullscreen(on);
                self.apply_fullscreen();
            }
            UiAction::Rebind(b) => self.ui.rebinding = Some(b),
            UiAction::Play(k) => {
                self.new_world(new_seed(), k);
                self.ui.in_title = false;
                self.title = false;
            }
            UiAction::MainMenu => {
                // ponytail: la partida se pierde hasta que haya guardado (R5)
                self.ui = Ui::new();
                self.ui.in_title = true;
                self.title = true;
            }
            UiAction::Cast(k) => {
                self.ui.window = None;
                self.ui.casting = Some(k);
            }
            UiAction::SetQuick(q, id) => {
                let me = self.world.d.hero;
                self.world.d.heroes[me].quickslots[q] = Some(id);
                self.ui.window = None;
            }
            UiAction::UseQuick(q) => {
                let h = self.world.d.local();
                let Some(slot) = h.quick_item(q) else {
                    self.ui.window = Some(UiWindow::Bag(BagMode::Quick(q)));
                    return;
                };
                // la acción principal; si sólo se puede soltar (arrojadizas hasta R5d), se abre la ventana del ítem
                match h.verbs(slot).first() {
                    Some(&verb) if verb != Verb::Drop => self.ui_action(UiAction::Verb(slot, verb)),
                    _ => self.ui.window = Some(UiWindow::Item(slot)),
                }
            }
            UiAction::Verb(slot, verb) => {
                let h = self.world.d.local();
                let need = h.item(slot).map_or(String::new(), |i| i.str("target").to_string());
                if verb == Verb::Read && !need.is_empty() {
                    self.ui.window = Some(UiWindow::Bag(BagMode::Target { scroll: slot, need })); // primero el objetivo
                } else {
                    self.ui.window = None;
                    self.ui_action(UiAction::Act(Action::Use(slot, verb, None)));
                }
            }
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
        // configuración: la próxima tecla va a la acción elegida (Esc cancela)
        if let Some(b) = self.ui.rebinding {
            if down {
                if code != KeyCode::Escape && config::assignable(code) {
                    self.config.rebind(b, code);
                }
                self.ui.rebinding = None;
                self.input.held = [false; 4];
            }
            return;
        }
        let bind = self.config.bind_of(code);
        // las flechas siempre mueven; las otras teclas de movimiento son configurables
        let dir = match (code, bind) {
            (KeyCode::ArrowUp, _) | (_, Some(Bind::Up)) => Some(0),
            (KeyCode::ArrowDown, _) | (_, Some(Bind::Down)) => Some(1),
            (KeyCode::ArrowLeft, _) | (_, Some(Bind::Left)) => Some(2),
            (KeyCode::ArrowRight, _) | (_, Some(Bind::Right)) => Some(3),
            _ => None,
        };
        if let Some(d) = dir {
            self.input.held[d] = down;
            return;
        }
        if !down {
            return;
        }
        if self.title {
            match code {
                KeyCode::Escape => self.ui.window = None,
                KeyCode::F11 => self.ui_action(UiAction::SetFullscreen(!self.config.fullscreen)),
                _ => {}
            }
            return;
        }
        let free = self.ui.window.is_none();
        match code {
            KeyCode::KeyR => self.new_world(new_seed(), self.world.class),
            KeyCode::KeyC => self.new_world(new_seed(), (self.world.class + 1) % CLASSES.len()),
            // Esc: cierra la ventana abierta, cancela examinar o abre el menú
            KeyCode::Escape if !free => self.ui.window = None,
            KeyCode::Escape if self.ui.examining => self.ui.examining = false,
            KeyCode::Escape if self.ui.casting.is_some() => self.ui.casting = None,
            KeyCode::Escape => self.ui.window = Some(UiWindow::Menu),
            KeyCode::F11 => self.ui_action(UiAction::SetFullscreen(!self.config.fullscreen)),
            // rasgo de clase: Segundo Aliento, o el libro de conjuros
            KeyCode::KeyF if free => {
                let h = self.world.d.local();
                if h.has_trait("pdri:second_wind") {
                    self.ui_action(UiAction::Act(Action::SecondWind));
                } else if h.spellcasting().is_some() {
                    self.ui.window = Some(UiWindow::Spells);
                }
            }
            _ => match bind {
                Some(Bind::Wait) if free => self.ui_action(UiAction::Act(Action::Rest)),
                Some(Bind::Search) if free => self.ui_action(UiAction::Act(Action::Search)),
                Some(Bind::Bag) if free => self.ui.window = Some(UiWindow::Bag(BagMode::Browse)),
                Some(Bind::Bag) => self.ui.window = None,
                Some(Bind::Quick1) if free => self.ui_action(UiAction::UseQuick(0)),
                Some(Bind::Quick2) if free => self.ui_action(UiAction::UseQuick(1)),
                _ => {}
            },
        }
    }

    /// Pantalla completa sin bordes (en Android la ventana ya ocupa todo).
    fn apply_fullscreen(&self) {
        if cfg!(target_os = "android") {
            return;
        }
        if let Some(gpu) = &self.gpu {
            gpu.window.set_fullscreen(self.config.fullscreen.then_some(winit::window::Fullscreen::Borderless(None)));
        }
    }

    fn new_world(&mut self, seed: i64, class: usize) {
        self.world = World::new(seed, class);
        self.ui = Ui::new();
        self.pan = [0.0; 2];
    }

    /// La pantalla de título: sólo la capa de UI.
    fn frame_title(&mut self, dt: f32) {
        let Some(gpu) = self.gpu.as_ref() else { return };
        let (uw, uh) = gpu.ui_size();
        self.ui.in_title = true;
        self.sprites.clear();
        let w = &self.world;
        let classes = &self.class_names;
        self.ui.draw_title(&self.atlas, &self.font, &w.chars, &w.d, &self.config, &mut self.sprites, uw as f32, uh as f32, dt, classes);
        let end = self.sprites.len() as u32;
        self.gpu.as_mut().unwrap().draw(&self.sprites, &[], 0..end);
    }

    fn frame(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        if self.title {
            self.frame_title(dt);
            return;
        }
        self.frames += 1;
        if (now - self.fps_since).as_secs_f32() >= 1.0 {
            self.fps = self.frames;
            self.frames = 0;
            self.fps_since = now;
            log::info!("FPS {}", self.fps);
        }

        // --- lógica: teclas mantenidas, animaciones, dados y turnos ---
        let w = &mut self.world;
        if w.ready() {
            let h = self.input.held;
            let (dx, dy) = ((h[3] as i32) - (h[2] as i32), (h[1] as i32) - (h[0] as i32));
            if (dx, dy) != (0, 0) {
                w.step(dx, dy);
            }
        }
        w.update(dt);
        if w.d.local().animating && self.input.press.is_none() {
            self.pan = [0.0; 2]; // el héroe camina: la cámara vuelve a seguirlo
        }
        self.water_off += map::WATER_SPEED * dt;

        let Some(gpu) = self.gpu.as_mut() else { return };
        let (lo, hi) = Self::zoom_range(gpu);
        gpu.set_world_scale(self.zoom.unwrap_or(gpu.ui_zoom()).clamp(lo, hi));
        if self.world.d.view_changed {
            gpu.set_fog(&map::fog_corners(self.world.d.level()));
            self.world.d.view_changed = false;
        }
        let gpu = self.gpu.as_ref().unwrap();
        let (ox, oy) = self.map_origin(gpu);

        // --- mundo: agua, tiles, casillas revisadas, cadáveres, personajes | niebla | hojas, textos, nombres ---
        let s = &mut self.sprites;
        s.clear();
        let w = &self.world;
        map::water(s, &self.atlas, self.atlas.rects[WATER], ox, oy, self.water_off);
        map::tiles(s, &self.atlas, self.atlas.rects[TILES], w.d.level(), ox, oy);
        ui::draw_heaps(s, &self.atlas, &self.font, &w.d, ox, oy);
        w.chars.draw_world(&w.d, &self.atlas, s, ox, oy);
        let atlas_end = s.len() as u32;
        s.push(map::fog_quad(ox, oy));
        let fog_end = s.len() as u32;
        w.chars.draw_over(&w.d, &self.atlas, &self.font, s, ox, oy);
        let world_end = s.len() as u32;

        // --- UI: log, panel de estado, carrusel, barra de herramientas, ventanas, tooltips | dados ---
        let (uw, uh) = gpu.ui_size();
        let (uw, uh) = (uw as f32, uh as f32);
        self.ui.draw(&self.atlas, &self.font, &w.chars, &w.d, &self.config, s, uw, uh, dt, self.fps);
        self.world.dice.draw(&self.atlas, &self.font, &mut self.sprites, uw, uh);
        let ui_end = self.sprites.len() as u32;

        let batches = [(Tex::Atlas, 0..atlas_end), (Tex::Fog, atlas_end..fog_end), (Tex::Atlas, fog_end..world_end)];
        self.gpu.as_mut().unwrap().draw(&self.sprites, &batches, world_end..ui_end);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        // Android destruye la superficie al irse al fondo: se recrea ventana y GPU en cada resumed
        let window = Arc::new(el.create_window(Window::default_attributes().with_title(TITLE)).expect("ventana"));
        self.gpu = Some(pollster::block_on(Gpu::new(window, &self.atlas)));
        self.world.d.view_changed = true; // la textura de niebla es nueva
        self.apply_fullscreen();
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
                self.ui.mouse = self.ui_point(position); // tooltips
                self.drag(position);
            }
            WindowEvent::CursorLeft { .. } => self.ui.mouse = None,
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let p = self.input.cursor;
                if state == ElementState::Pressed { self.press(p) } else { self.release(p) }
            }
            // click derecho: la acción alternativa de la UI (elegir el acceso rápido)
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Right, .. } => {
                if let Some(act) = self.ui_point(self.input.cursor).and_then(|q| self.ui.click(q, true)) {
                    self.ui_action(act);
                }
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

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if self.quit {
            el.exit();
            return;
        }
        let Some(gpu) = &self.gpu else { return };
        // tope de FPS (configuración): se espera hasta el próximo frame; 0 = tan rápido como presente la superficie
        let cap = self.config.max_fps;
        let next = self.last + Duration::from_secs_f64(1.0 / cap.max(1) as f64);
        if cap == 0 || Instant::now() >= next {
            el.set_control_flow(ControlFlow::Poll);
            gpu.window.request_redraw();
        } else {
            el.set_control_flow(ControlFlow::WaitUntil(next));
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
    if let Some(dir) = app.internal_data_path() {
        let _ = config::DIR.set(dir); // Local.config en la carpeta de datos de la app
    }
    run(EventLoop::builder().with_android_app(app).build().expect("bucle de eventos"));
}
