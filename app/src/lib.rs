//! Ventana (winit), bucle del juego y render (wgpu). Lee el estado de `game`, nunca al revés.
mod atlas;
mod font;
mod gpu;

use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use atlas::Atlas;
use font::Font;
use gpu::{Gpu, Sprite};

pub const TITLE: &str = "PixelDungeon-RollInitiative";
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.27, 1.0];
const HERO_SPEED: f32 = 40.0; // px del juego por segundo (todo lo que se mueve va por dt)

/// Hojas del atlas, en orden: los índices de `Atlas::rects`. Van dentro del binario (Android sin AssetManager).
const SHEETS: [&[u8]; 2] = [
    include_bytes!("../../Assets/Fonts/font1x.png"),
    include_bytes!("../../Assets/Characters/warrior.png"),
];
const FONT: usize = 0;
const WARRIOR: usize = 1;

struct App {
    atlas: Atlas,
    font: Font,
    gpu: Option<Gpu>,
    sprites: Vec<Sprite>,
    last: Instant,
    fps_since: Instant,
    frames: u32,
    fps: u32,
    hero_x: f32,
}

impl App {
    fn new() -> Self {
        let atlas = Atlas::build(&SHEETS);
        let font = Font::new(&atlas, FONT);
        let now = Instant::now();
        Self { atlas, font, gpu: None, sprites: Vec::new(), last: now, fps_since: now, frames: 0, fps: 0, hero_x: 0.0 }
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
        let Some(gpu) = self.gpu.as_mut() else { return };
        let (w, _) = gpu.virtual_size();

        // R0: texto + un sprite que cruza la pantalla (prueba de dt y del lote de instancias)
        self.sprites.clear();
        self.font.text(&mut self.sprites, &format!("FPS {}", self.fps), 4.0, 4.0, YELLOW);
        self.font.text(&mut self.sprites, &format!("{TITLE} v{}", game::VERSION), 4.0, 14.0, WHITE);
        self.hero_x = (self.hero_x + HERO_SPEED * dt) % (w as f32 + 12.0);
        let r = self.atlas.rects[WARRIOR];
        self.sprites.push(self.atlas.sprite(r[0], r[1], 12, 15, [self.hero_x - 12.0, 40.0], WHITE));
        gpu.draw(&self.sprites);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        // Android destruye la superficie al irse al fondo: se recrea ventana y GPU en cada resumed
        let window = Arc::new(el.create_window(Window::default_attributes().with_title(TITLE)).expect("ventana"));
        self.gpu = Some(pollster::block_on(Gpu::new(window, &self.atlas)));
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
