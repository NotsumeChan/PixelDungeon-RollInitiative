//! Ventana (winit), bucle del juego y render (wgpu). Lee el estado de `game`, nunca al revés.
//! R3: el héroe juega un piso con turnos, mobs y combate 5e. R4: personajes animados, dados y efectos.
mod atlas;
mod config;
mod dice_fx;
mod font;
mod gpu;
mod map;
mod mods;
mod net;
mod sprites;
mod ui;
mod updater;

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use config::{Bind, Config};
use game::{characters, save, tr, trf};

use game::dungeon::Dungeon;
use game::hero::Action;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
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
/// Clases jugables, en el orden de la creación de personaje.
/// La clase si falta (el juego base siempre la trae). Las que se eligen salen de `data/classes.json`, en su orden.
const DEFAULT_CLASS: &str = "pdri:warrior";

/// Hojas del atlas, en orden: los índices de `Atlas::rects`. Van dentro del binario (Android sin AssetManager).
const SHEETS: [&[u8]; 15] = [
    include_bytes!("../../Assets/Fonts/silkscreen/silkscreen.png"), // Silkscreen (OFL), `tools/font_strip.py`
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
    chars: Chars,
    dice: DiceFx,
    zone: game::level::Zone,
}

impl World {
    /// Partida nueva de un personaje (clase, características y nombre). `hotseat`: suma un segundo héroe igual
    /// ("<nombre> 2"), el modo de prueba de la red.
    fn new(seed: i64, class: &str, abilities: [i32; 6], name: &str, skin: &str, hotseat: bool) -> Self {
        let mut d = Dungeon::new_with(seed, class, abilities);
        if !name.is_empty() {
            d.heroes[0].ch.name = name.to_string();
        }
        d.heroes[0].skin = skin.to_string();
        if hotseat {
            let j = d.add_hero(class, abilities);
            d.heroes[j].ch.name = format!("{} 2", d.heroes[0].ch.name);
            d.heroes[j].skin = skin.to_string();
        }
        Self::from_dungeon(d)
    }

    /// Hotseat (`AutoSwitch` del prototipo): si el local murió o cayó, la UI pasa al primero en pie; si el piso espera
    /// la acción de otro héroe, pasa a ese. Con los héroes en pisos distintos se cambia con Tab.
    fn auto_switch(&mut self) {
        let d = &mut self.d;
        let up = |h: &game::hero::Hero| h.is_alive() && !h.is_downed();
        if !up(d.local()) {
            if let Some(j) = (0..d.heroes.len()).find(|&j| up(&d.heroes[j])) {
                d.set_local(j);
            }
            return;
        }
        if let Some(b) = d.blocking()
            && b != d.hero
            && up(&d.heroes[b])
            && d.heroes[b].action.is_none()
            && !d.heroes[b].animating
            && !d.local().animating
        {
            d.set_local(b);
        }
    }

    /// Una partida cargada (o recién creada).
    fn from_dungeon(d: Dungeon) -> Self {
        let depth = d.zone;
        let seed = d.seed;
        // el azar visual va aparte del rng de la partida
        let chars = Chars::new(SHEETS.len(), ICONS, seed ^ 0x5EED);
        Self { d, chars, dice: DiceFx::new(seed ^ 0xD1CE), zone: depth }
    }

    /// Un frame de lógica, en el orden del prototipo: los sprites caminan (y sueltan al héroe al llegar), lo que el
    /// dado ya mostró se aplica, corren los turnos, y después los efectos y los dados avanzan.
    /// `remote` = cliente de una partida LAN: la partida la corre el host (acá sólo se anima lo que llega).
    fn update(&mut self, dt: f32, remote: bool) {
        self.chars.animate(&mut self.d, dt);
        self.dice.feed(&mut self.d.dice_events);
        if !remote && !self.d.pending.is_empty() && !self.dice.busy() {
            self.d.resolve_pending(); // recién cuando el dado mostró el resultado se aplica el daño
        }
        if !remote {
            self.d.process();
        }
        if self.d.zone != self.zone {
            self.zone = self.d.zone;
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
        let l = self.d.level();
        let open = |c: i32| l.inside(c) && (l.passable[c as usize] || self.d.find_char(c).is_some());
        let tries = [(dx, dy), (dx, 0), (0, dy)];
        let in_row = |t: &&(i32, i32)| (0..l.width).contains(&(pos % l.width + t.0)); // sin dar la vuelta por el costado
        let found = tries.iter().filter(|t| **t != (0, 0)).filter(in_row).map(|(x, y)| pos + x + y * l.width).find(|&c| open(c));
        if let Some(c) = found {
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
    /// Las clases para la creación de personaje: nombre, "Guerrero d10" y avatar.
    class_rows: Vec<ui::RosterRow>,
    classes: serde_json::Map<String, serde_json::Value>,
    /// Las apariencias (`data/skins.json`) y sus filas para la creación.
    skins: serde_json::Map<String, serde_json::Value>,
    skin_rows: Vec<ui::RosterRow>,
    /// El personaje en juego (characters.json) y la lista de personajes (con sus filas para la UI).
    character: Option<serde_json::Value>,
    roster: Vec<serde_json::Value>,
    roster_rows: Vec<ui::RosterRow>,
    /// Piso del último autoguardado (se guarda al cambiar de piso).
    saved_depth: game::level::Zone,
    quit: bool,
    input: Input,
    pan: [f32; 2],      // desplazamiento de la cámara al arrastrar (pixeles del mundo)
    zoom: Option<u32>,  // pixeles de pantalla por pixel del mundo (None = el de la UI)
    water_off: f32,
    last: Instant,
    fps_since: Instant,
    frames: u32,
    fps: u32,
    /// Partida en red: host, cliente o ninguna.
    net: Net,
    /// Buscando partidas (ventana Unirse), lo encontrado y el último aviso de la red.
    browser: Option<net::Browser>,
    lan_games: Vec<net::Game>,
    lan_status: String,
    /// Cliente: mandó una acción y espera el estado que la incluya (no repetir el paso con la tecla mantenida).
    awaiting: bool,
    /// Adónde se intentó unir por última vez (para reintentar con la clave que pide el host).
    join_addr: Option<String>,
    /// Android: el teclado del sistema está abierto; volver a abrirlo (se tocó la línea: Atrás lo pudo cerrar).
    kb_shown: bool,
    kb_again: bool,
    /// Actualizador (D4, `updater`): la respuesta que se espera, la versión que se ofrece, el aviso y si está bajando.
    upd_rx: Option<std::sync::mpsc::Receiver<updater::Msg>>,
    offer: Option<String>,
    upd_msg: String,
    upd_busy: bool,
    /// La respuesta que se espera la pidió el jugador (buscar a mano, actualizar): se muestra todo, no sólo `Found`.
    upd_manual: bool,
    /// Conectando en otro hilo (DNS + connect): el resultado y el personaje con el que se entra.
    connecting: Option<(std::sync::mpsc::Receiver<std::io::Result<net::ClientGame>>, serde_json::Value)>,
}

enum Net {
    Off,
    Host(net::HostGame),
    Client(net::ClientGame),
}

/// Carpeta de datos del usuario (personajes y partidas), la misma que el prototipo: Windows %APPDATA%/PixelDND, Linux
/// $XDG_DATA_HOME/PixelDND (o ~/.local/share/PixelDND); en Android, la carpeta de datos de la app.
fn data_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    if let Some(d) = config::DIR.get() {
        return d.clone();
    }
    let var = |k: &str| std::env::var_os(k).map(PathBuf::from);
    let base = if cfg!(windows) { var("APPDATA") } else { var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share"))) };
    base.unwrap_or_else(|| PathBuf::from(".")).join("PixelDND")
}

/// Las clases para la creación de personaje, en el idioma actual: nombre y "Guerrero d10" (sin avatar: la apariencia
/// es aparte). Todas las de `data/classes.json`, en su orden.
fn class_rows(c: &serde_json::Map<String, serde_json::Value>) -> Vec<ui::RosterRow> {
    c.values()
        .map(|v| ui::RosterRow {
            name: game::registry::text(&v["name"]),
            status: format!("{} d{}", game::registry::text(game::hero::class_5e(v)), v["hit_die"]),
            stats: String::new(),
            sheet: None,
        })
        .collect()
}

/// La hoja del atlas de una skin (None = no está o su png no está cargado).
fn skin_sheet(skins: &serde_json::Map<String, serde_json::Value>, id: &str) -> Option<usize> {
    let file = skins.get(id)?["sprite"].as_str()?;
    CHAR_SHEETS.iter().position(|e| e.0 == file).map(|i| i + SHEETS.len())
}

/// Las apariencias para la creación: nombre y avatar.
fn skin_rows(s: &serde_json::Map<String, serde_json::Value>) -> Vec<ui::RosterRow> {
    s.iter()
        .map(|(k, v)| ui::RosterRow { name: game::registry::text(&v["name"]), status: String::new(), stats: String::new(), sheet: skin_sheet(s, k) })
        .collect()
}

/// El orden de las características de una clase (`ability_priority` de classes.json, como índices de `ABILITIES`); las
/// que falten van al final en su orden.
fn ability_order(cls: Option<&serde_json::Value>) -> [usize; 6] {
    let names = cls.and_then(|c| c["ability_priority"].as_array()).cloned().unwrap_or_default();
    let named = names.iter().filter_map(|n| game::dice::ABILITIES.iter().position(|a| n.as_str() == Some(*a)));
    let mut order = Vec::new();
    for a in named.chain(0..6) {
        if !order.contains(&a) {
            order.push(a);
        }
    }
    std::array::from_fn(|k| order[k])
}

/// Otra versión del protocolo: quién tiene que actualizar.
fn version_msg(other: &str) -> String {
    if net::newer(other, game::VERSION) && updater::ENABLED {
        trf!("La partida es de la versión {}: toca ACTUALIZAR.", "The game is on version {}: tap UPDATE.", other)
    } else if net::newer(other, game::VERSION) {
        trf!("La partida es de la versión {}: actualiza el juego (Releases en GitHub).", "The game is on version {}: update yours (GitHub Releases).", other)
    } else {
        trf!("La partida es de la versión {}, más vieja que la tuya: el host tiene que actualizar.", "The game is on version {}, older than yours: the host has to update.", other)
    }
}

/// El aviso de la sala de espera: IP y puerto (para un túnel o el puerto abierto) y si pide clave.
fn host_status(locked: bool) -> String {
    let lock = if locked { tr!("Con clave.", "Password set.") } else { tr!("Sin clave.", "No password.") };
    trf!("Partida abierta en {}:{}. {}", "Game open at {}:{}. {}", net::local_ip(), net::TCP_PORT, lock)
}

fn new_seed() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_micros() as i64 % 2_147_483_647)
}

impl App {
    fn new() -> Self {
        let config = Config::load();
        updater::cleanup(&data_dir());
        let upd_rx = if config.check_updates() { updater::check() } else { None };
        for m in game::registry::load_mods(&data_dir(), &config.disabled_mods()) {
            log::info!("{m}"); // Android: logcat
            eprintln!("{m}");
        }
        // opciones de los mods: las de Local.config o el `default` de su mod.json
        for m in game::registry::mods() {
            for s in m.manifest["settings"].as_array().into_iter().flatten() {
                let Some(id) = s["id"].as_str() else { continue };
                let v = config.mod_setting(&m.folder, id).and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_else(|| s["default"].clone());
                game::registry::set_setting(&m.folder, id, v);
            }
        }
        // los íconos de los mods van al final del atlas (`ui::MOD_ICONS`: mod -> hoja)
        let icons: Vec<Option<&[u8]>> = game::registry::mods().iter().map(|m| m.icon.as_deref().filter(|b| atlas::icon_ok(b))).collect();
        let first = SHEETS.len() + CHAR_SHEETS.len();
        let mut next = first;
        let _ = ui::MOD_ICONS.set(icons.iter().map(|i| i.map(|_| (next, next += 1).0)).collect());
        let sheets: Vec<&[u8]> = SHEETS.iter().copied().chain(CHAR_SHEETS.iter().map(|c| c.1)).chain(icons.iter().flatten().copied()).collect();
        let atlas = Atlas::build(&sheets);
        let font = Font::new(&atlas, FONT);
        let now = Instant::now();
        game::registry::set_lang(config.language);
        Self {
            atlas,
            font,
            gpu: None,
            sprites: Vec::new(),
            world: World::new(new_seed(), DEFAULT_CLASS, game::dice::DEFAULT_SCORES, "", "", false),
            ui: Ui::new(),
            config,
            title: true,
            class_rows: class_rows(&game::registry::load("classes").unwrap_or_default()),
            classes: game::registry::load("classes").unwrap_or_default(),
            skins: game::registry::load("skins").unwrap_or_default(),
            skin_rows: skin_rows(&game::registry::load("skins").unwrap_or_default()),
            character: None,
            roster: Vec::new(),
            roster_rows: Vec::new(),
            saved_depth: game::level::Zone::NONE,
            quit: false,
            input: Input::default(),
            pan: [0.0; 2],
            zoom: None,
            water_off: 0.0,
            last: now,
            fps_since: now,
            frames: 0,
            fps: 0,
            net: Net::Off,
            browser: None,
            lan_games: Vec::new(),
            lan_status: String::new(),
            awaiting: false,
            join_addr: None,
            connecting: None,
            kb_shown: false,
            kb_again: false,
            upd_rx,
            offer: None,
            upd_msg: String::new(),
            upd_busy: false,
            upd_manual: false,
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
        let l = self.world.d.level();
        ((0..l.width).contains(&x) && (0..l.height).contains(&y)).then_some(x + y * l.width)
    }

    fn press(&mut self, p: PhysicalPosition<f64>) {
        self.input.press = Some((p, self.pan));
        self.input.dragging = false;
        // creación de personaje: apretar sobre una tirada o característica puede empezar a arrastrarla
        if self.creating()
            && let Some(q) = self.ui_point(p)
        {
            let under = self.ui.click(q, false);
            self.ui.create.press(under.as_ref(), q);
        }
    }

    fn creating(&self) -> bool {
        self.title && matches!(self.ui.window, Some(UiWindow::Create))
    }

    fn drag(&mut self, p: PhysicalPosition<f64>) {
        let Some((start, pan0)) = self.input.press else { return };
        if self.ui.create.drag.is_some()
            && let Some(q) = self.ui_point(p)
        {
            self.ui.create.moved_to(q);
            return;
        }
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
        if self.ui.create.drag.is_some()
            && let Some(q) = self.ui_point(p)
        {
            self.input.press = None;
            let under = self.ui.click(q, false);
            self.ui.create.release(under.as_ref(), q);
            return;
        }
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
        if let Some(s) = self.ui.throwing.take() {
            self.ui_action(UiAction::Act(Action::Throw(s, cell)));
            return;
        }
        let d = &mut self.world.d;
        if !d.local().is_alive() {
            return;
        }
        let me = d.hero;
        // tocarse = usar lo que hay abajo (montón, cofre, escalera, portal, borde: lo que decida `handle`); si no hay
        // nada (`handle` da caminar a la propia celda), buscar
        d.handle(me, cell, true);
        if cell == d.local().ch.pos && d.heroes[me].action == Some(Action::Move(cell)) {
            d.heroes[me].action = Some(Action::Search);
        }
        self.ui.note_walk(&self.world.d);
    }

    /// Resultado de importar un mod (`mods.rs`): el aviso en la pantalla de Mods y la lista al día.
    fn mod_imported(&mut self, r: Result<String, String>) {
        self.ui.mods_msg = match r {
            Ok(name) => game::trf!("Mod \"{name}\" instalado: reinicia el juego para cargarlo.", "Mod \"{name}\" installed: restart the game to load it."),
            Err(e) => game::trf!("No se pudo importar: {e}", "Couldn't import: {e}"),
        };
    }

    fn ui_action(&mut self, act: UiAction) {
        if matches!(act, UiAction::Prompt(ui::Cr::Name) | UiAction::Chat(ui::Cr::Name) | UiAction::Create(ui::Cr::Name | ui::Cr::Stat(_))) {
            self.kb_again = true;
        }
        match act {
            UiAction::Nothing => {}
            UiAction::Close => self.ui.window = None,
            UiAction::Open(w) => self.ui.window = Some(w),
            UiAction::ToggleMod(m) => self.config.toggle_mod(&m),
            UiAction::ModSetting(k, id, v) => {
                if let Some(m) = game::registry::mods().get(k) {
                    self.config.set_mod_setting(&m.folder, &id, &v.to_string());
                    game::registry::set_setting(&m.folder, &id, v);
                }
            }
            UiAction::AckMods => self.config.set_mods_ack(),
            UiAction::ImportMod => {
                if let Some(r) = mods::pick(&data_dir()) {
                    self.mod_imported(r);
                }
            }
            UiAction::Act(a) => {
                let d = &mut self.world.d;
                if d.local().is_alive() {
                    let me = d.hero;
                    d.heroes[me].action = Some(a);
                    self.ui.note_walk(&self.world.d);
                }
            }
            UiAction::Examine => self.ui.examining = !self.ui.examining,
            UiAction::NewGame => self.new_world(new_seed()),
            UiAction::Quit => {
                self.autosave();
                self.quit = true;
            }
            UiAction::OpenRoster => {
                // desde el título (sala de espera, Unirse): se deja la partida LAN y de buscar
                if self.title {
                    self.net = Net::Off;
                    self.browser = None;
                    self.connecting = None; // si conecta después, se descarta (y se corta)
                }
                self.open_roster();
            }
            UiAction::StartLan => {
                if let Net::Host(h) = &mut self.net {
                    h.start();
                }
                self.title = false;
                self.ui = Ui::new();
                self.pan = [0.0; 2];
                self.lan_status.clear();
            }
            UiAction::Trait => {
                // rasgo de clase: Segundo Aliento, o el libro de conjuros (F, o el botón de la barra)
                let h = self.world.d.local();
                if h.has_trait("pdri:second_wind") {
                    self.ui_action(UiAction::Act(Action::SecondWind));
                } else if h.has_book() {
                    self.ui.window = Some(UiWindow::Spells);
                }
            }
            UiAction::HostLan => {
                let Some(c) = self.ui.roster_sel.and_then(|i| self.roster.get(i)).cloned() else { return };
                let name = c["name"].as_str().unwrap_or("?").to_string();
                self.start(c);
                if self.title {
                    return; // no se pudo empezar (clase que no está cargada)
                }
                match net::HostGame::new(&name) {
                    Ok(h) => {
                        // sala de espera: la partida queda cargada y empieza con EMPEZAR
                        self.net = Net::Host(h);
                        self.title = true;
                        self.ui = Ui::new();
                        self.ui.in_title = true;
                        self.ui.window = Some(UiWindow::Lobby);
                        self.lan_status = host_status(false);
                    }
                    Err(e) => self.world.d.log(&trf!("No se pudo abrir la partida LAN: {e}", "Couldn't open the LAN game: {e}")),
                }
            }
            UiAction::OpenJoin => {
                self.lan_status.clear();
                match net::Browser::new() {
                    Ok(b) => self.browser = Some(b),
                    Err(e) => self.lan_status = trf!("Sin red: {e}.", "No network: {e}."),
                }
                self.ui.window = Some(UiWindow::Join);
            }
            UiAction::JoinGame(i) => {
                let Some(g) = self.lan_games.get(i) else { return };
                if g.info["protocol"] != net::PROTOCOL {
                    let v = g.info["version"].as_str().unwrap_or("?").to_string();
                    self.lan_status = version_msg(&v);
                    if updater::ENABLED && net::newer(&v, game::VERSION) {
                        self.offer = Some(v); // ACTUALIZAR baja la versión del host, no la última
                    }
                    return;
                }
                if g.info["hash"] != game::registry::content_hash().as_str() {
                    self.lan_status = tr!("Esa partida tiene otro contenido (data/ o mods).", "That game has different content (data/ or mods).").into();
                    return;
                }
                let addr = g.addr.to_string();
                self.join(addr, String::new());
            }
            UiAction::Ask(a) => {
                let text = if a == ui::Ask::Addr { self.config.last_address() } else { String::new() };
                self.ui.prompt = Some((a, text));
            }
            UiAction::Prompt(cr) => match cr {
                ui::Cr::Back => _ = self.ui.prompt.as_mut().and_then(|p| p.1.pop()),
                ui::Cr::Name => {} // tocar la línea: sólo vuelve a abrir el teclado del sistema
                ui::Cr::Done => {
                    let Some((a, text)) = self.ui.prompt.take() else { return };
                    match a {
                        ui::Ask::Addr => {
                            self.config.set_last_address(&text);
                            self.join(text, String::new());
                        }
                        ui::Ask::Pass => {
                            if let Some(addr) = self.join_addr.clone() {
                                self.join(addr, text);
                            }
                        }
                        ui::Ask::HostPass => {
                            if let Net::Host(h) = &mut self.net {
                                h.set_password(&text);
                                self.lan_status = host_status(h.locked());
                            }
                        }
                    }
                }
                _ => self.ui.prompt = None,
            },
            UiAction::SelectChar(i) => {
                self.ui.roster_sel = Some(i);
                self.ui.confirm = false;
            }
            UiAction::PlayChar => {
                if let Some(c) = self.ui.roster_sel.and_then(|i| self.roster.get(i)).cloned() {
                    self.start(c);
                }
            }
            UiAction::DeleteChar => {
                let Some(i) = self.ui.roster_sel.filter(|&i| i < self.roster.len()) else { return };
                if !self.ui.confirm {
                    self.ui.confirm = true; // primer click: pedir confirmación
                    return;
                }
                let c = self.roster.remove(i);
                let dir = data_dir();
                if let Err(e) = characters::save(&dir, &self.roster) {
                    log::warn!("no se pudo guardar la lista de personajes: {e}");
                }
                save::delete_run(&dir, characters::id(&c));
                self.open_roster();
            }
            UiAction::SetFps(f) => self.config.set_fps(f),
            UiAction::SetShowFps(on) => self.config.set_show_fps(on),
            UiAction::SetCheckUpdates(on) => self.config.set_check_updates(on),
            UiAction::SettingsTab(t) => {
                if t == 1 {
                    self.ui.window = Some(UiWindow::Controls);
                } else {
                    self.ui.settings_tab = t;
                    self.ui.window = Some(UiWindow::Settings);
                }
            }
            UiAction::LangScroll(d) => self.ui.lang_scroll = self.ui.lang_scroll.saturating_add_signed(d as isize),
            UiAction::CheckUpdates => {
                if !self.upd_busy {
                    self.upd_rx = updater::check();
                    self.upd_busy = self.upd_rx.is_some();
                    self.upd_manual = true;
                    self.upd_msg = tr!("Buscando actualizaciones", "Checking for updates").into();
                }
            }
            UiAction::UpdateLater => self.offer = None,
            UiAction::Update(v) => {
                if !self.upd_busy {
                    self.upd_rx = updater::install(&v, &data_dir());
                    self.upd_busy = self.upd_rx.is_some();
                    self.upd_manual = true;
                    self.upd_msg = trf!("Descargando la versión {}", "Downloading version {}", v);
                    self.lan_status = self.upd_msg.clone();
                }
            }
            UiAction::SetLanguage(code) => {
                // lo que ya está en la partida (nombres de mobs) cambia con lo próximo que se cargue
                self.config.set_language(code);
                game::registry::set_lang(code);
                self.class_rows = class_rows(&self.classes);
                self.skin_rows = skin_rows(&self.skins);
            }
            UiAction::SetResolution(r) => {
                self.config.set_resolution(r);
                if let Some(gpu) = &self.gpu
                    && !self.config.fullscreen
                {
                    let (w, h) = self.config.window_size();
                    let _ = gpu.window.request_inner_size(PhysicalSize::new(w, h));
                }
            }
            UiAction::SetFullscreen(on) => {
                self.config.set_fullscreen(on);
                self.apply_fullscreen();
            }
            UiAction::Rebind(b) => self.ui.rebinding = Some(b),
            UiAction::SetRule(key, v) => {
                let r = &mut self.world.d.rules;
                match key {
                    "dot_death_saves" => r.dot_death_saves = v,
                    "save_half" => r.save_half = v,
                    _ => {}
                }
            }
            UiAction::SetUiScale(s) => {
                self.config.set_ui_scale(s);
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.set_ui_scale(s);
                }
            }
            UiAction::Create(ui::Cr::Start) => {
                let st = &self.ui.create;
                let Some(ab) = game::dice::valid_scores(&st.scores) else { return };
                let name = match st.name.trim() {
                    "" => self.class_rows[st.class].name.clone(),
                    n => n.to_string(),
                };
                let Some(class) = self.classes.keys().nth(st.class).cloned() else { return };
                let mut c = characters::new_character(&name, &class, ab);
                if let Some(s) = self.skins.keys().nth(st.skin) {
                    c["skin"] = serde_json::Value::String(s.clone());
                }
                self.roster.push(c.clone());
                if let Err(e) = characters::save(&data_dir(), &self.roster) {
                    log::warn!("no se pudo guardar la lista de personajes: {e}");
                }
                self.start(c);
            }
            UiAction::Create(c) => {
                if matches!(c, ui::Cr::New) {
                    self.ui.window = Some(UiWindow::Create);
                }
                let fresh = matches!(c, ui::Cr::New | ui::Cr::Roll); // tiradas nuevas: se reparten siempre
                let reorder = fresh || matches!(c, ui::Cr::Class(_));
                self.ui.create.apply(c, new_seed());
                if reorder {
                    let order = ability_order(self.classes.values().nth(self.ui.create.class));
                    self.ui.create.auto_assign(&order, fresh);
                }
            }
            UiAction::ShopSel(npc, k) => self.ui.shop_sel = Some((npc, k)),
            UiAction::Chat(cr) => match cr {
                ui::Cr::New => self.ui.chat = Some(String::new()),
                ui::Cr::Back => _ = self.ui.chat.as_mut().and_then(String::pop),
                ui::Cr::Name => {} // tocar la línea: sólo vuelve a abrir el teclado del sistema
                ui::Cr::Done => {
                    let text = self.ui.chat.take().unwrap_or_default();
                    match &mut self.net {
                        Net::Host(h) => h.chat(&mut self.world.d, &text),
                        Net::Client(c) => c.send_chat(&text),
                        Net::Off => {}
                    }
                }
                _ => self.ui.chat = None,
            },
            UiAction::MainMenu => {
                self.autosave();
                self.net = Net::Off; // cierra la partida LAN (los clientes ven que se cortó)
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
                // apuntando con esta arrojadiza: tocar su acceso rápido otra vez la lanza al enemigo más cerca
                let d = &self.world.d;
                let aim = d.nearest_enemy(d.hero).and_then(|a| d.ch(a)).map(|c| c.pos);
                if let (Some(s), Some(cell)) = (self.ui.throwing, aim)
                    && d.local().quick_item(q) == Some(s)
                {
                    self.ui.throwing = None;
                    self.ui_action(UiAction::Act(Action::Throw(s, cell)));
                    return;
                }
                let h = self.world.d.local();
                let Some(slot) = h.quick_item(q) else {
                    self.ui.window = Some(UiWindow::Bag(BagMode::Quick(q)));
                    return;
                };
                // la acción principal (arrojadizas: LANZAR); si sólo se puede soltar, se abre la ventana del ítem
                match h.verbs(slot).first() {
                    Some(&verb) if verb != Verb::Drop => self.ui_action(UiAction::Verb(slot, verb)),
                    _ => self.ui.window = Some(UiWindow::Item(slot)),
                }
            }
            UiAction::Verb(slot, verb) => {
                let h = self.world.d.local();
                let need = h.item(slot).map_or(String::new(), |i| i.str("target").to_string());
                if verb == Verb::Throw {
                    self.ui.window = None;
                    self.ui.throwing = Some(slot); // primero la casilla
                } else if verb == Verb::Read && !need.is_empty() {
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
                // Esc en la creación vuelve a la lista de personajes
                KeyCode::Escape if matches!(self.ui.window, Some(UiWindow::Create)) => {
                    self.ui.create.focus = None;
                    self.open_roster();
                }
                KeyCode::Escape if matches!(self.ui.window, Some(UiWindow::Lobby | UiWindow::Join)) => {
                    self.ui_action(UiAction::OpenRoster)
                }
                KeyCode::Escape => self.ui.window = None,
                KeyCode::F11 => self.ui_action(UiAction::SetFullscreen(!self.config.fullscreen)),
                _ => {}
            }
            return;
        }
        let free = self.ui.window.is_none();
        match code {
            KeyCode::KeyR if matches!(self.net, Net::Off) => self.new_world(new_seed()),
            // hotseat: Tab pasa al siguiente héroe vivo
            KeyCode::Tab if free && self.config.hotseat => {
                let d = &mut self.world.d;
                let alive: Vec<usize> = (0..d.heroes.len()).filter(|&j| d.heroes[j].is_alive()).collect();
                if let Some(k) = alive.iter().position(|&j| j == d.hero)
                    && alive.len() > 1
                {
                    d.set_local(alive[(k + 1) % alive.len()]);
                    (self.ui.casting, self.ui.throwing, self.ui.examining) = (None, None, false);
                    self.pan = [0.0; 2];
                }
            }
            // Esc: cierra la ventana abierta, cancela examinar o abre el menú
            KeyCode::Escape if self.world.d.local().talking.is_some() || self.world.d.local().trading.is_some() => {
                self.ui_action(UiAction::Act(Action::Bye))
            }
            KeyCode::Escape if !free => self.ui.window = None,
            KeyCode::Escape if self.ui.examining => self.ui.examining = false,
            KeyCode::Escape if self.ui.casting.is_some() => self.ui.casting = None,
            KeyCode::Escape if self.ui.throwing.is_some() => self.ui.throwing = None,
            KeyCode::Escape => self.ui.window = Some(UiWindow::Menu),
            KeyCode::F11 => self.ui_action(UiAction::SetFullscreen(!self.config.fullscreen)),
            // rasgo de clase: Segundo Aliento, o el libro de conjuros
            KeyCode::KeyF if free => self.ui_action(UiAction::Trait),
            KeyCode::Enter | KeyCode::NumpadEnter if free && bind.is_none() && self.ui.lan => self.ui_action(UiAction::Chat(ui::Cr::New)),
            _ => match bind {
                Some(Bind::Wait) if free => self.ui_action(UiAction::Act(Action::Rest)),
                Some(Bind::Search) if free => self.ui_action(UiAction::Act(Action::Search)),
                Some(Bind::Bag) if free => self.ui.window = Some(UiWindow::Bag(BagMode::Browse)),
                Some(Bind::Bag) => self.ui.window = None,
                Some(Bind::Quick1) if free => self.ui_action(UiAction::UseQuick(0)),
                Some(Bind::Quick2) if free => self.ui_action(UiAction::UseQuick(1)),
                Some(Bind::Action) if free => {
                    let d = &self.world.d;
                    if let Some(a) = d.quick_action(d.hero) {
                        self.ui_action(UiAction::Act(a));
                    }
                }
                Some(Bind::Resume) if free && self.world.d.floor().scheduler.order.is_none() => {
                    if let Some(a) = self.ui.resumable(&self.world.d) {
                        self.ui_action(UiAction::Act(a));
                    }
                }
                _ => {}
            },
        }
    }

    /// Actualizador (D4): recoge lo que contestó su hilo y lo pasa a la UI. `Ready`: en escritorio reinicia con el
    /// ejecutable nuevo; en Android abre la instalación del APK (`MainActivity.installApk`).
    fn update_pump(&mut self) {
        let got = self.upd_rx.as_ref().map(|rx| rx.try_recv());
        match got {
            Some(Ok(m)) => {
                self.upd_rx = None;
                self.upd_busy = false;
                let manual = std::mem::take(&mut self.upd_manual);
                match m {
                    updater::Msg::Found(v) => {
                        self.offer = Some(v);
                        self.upd_msg.clear();
                    }
                    updater::Msg::UpToDate if manual => self.upd_msg = tr!("Ya tienes la última versión.", "You already have the latest version.").into(),
                    updater::Msg::Failed(e) if manual => self.upd_msg = trf!("No se pudo actualizar: {}.", "Couldn't update: {}.", e),
                    updater::Msg::UpToDate | updater::Msg::Failed(_) => {} // al abrir: sin red o al día, nada
                    updater::Msg::Ready(path) => self.apply_update(&path),
                }
                if !self.upd_msg.is_empty() && matches!(self.ui.window, Some(UiWindow::Join)) {
                    self.lan_status = self.upd_msg.clone();
                }
            }
            Some(Err(std::sync::mpsc::TryRecvError::Disconnected)) => (self.upd_rx, self.upd_busy) = (None, false),
            _ => {}
        }
        self.ui.offer = self.offer.clone();
        self.ui.update_msg = self.upd_msg.clone();
        self.ui.updating = self.upd_busy;
        self.ui.busy |= self.upd_busy;
    }

    fn apply_update(&mut self, path: &std::path::Path) {
        self.offer = None;
        #[cfg(not(target_os = "android"))]
        {
            self.autosave();
            match std::process::Command::new(path).args(std::env::args_os().skip(1)).spawn() {
                Ok(_) => self.quit = true,
                Err(e) => self.upd_msg = trf!("Actualizado: vuelve a abrir el juego ({}).", "Updated: open the game again ({}).", e),
            }
        }
        #[cfg(target_os = "android")]
        {
            let p = path.to_string_lossy().into_owned();
            let r = with_activity(|env, act| {
                let s = env.new_string(&p)?;
                env.call_method(act, "installApk", "(Ljava/lang/String;)V", &[(&s).into()]).map(|_| ())
            });
            self.upd_msg = match r {
                Ok(()) => tr!("Confirma la instalación.", "Confirm the install.").into(),
                Err(e) => trf!("No se pudo instalar: {}.", "Couldn't install: {}.", e),
            };
        }
    }

    /// Texto en Android: el teclado del sistema abierto mientras se escribe (dirección, clave, chat, nombre o
    /// características) y sus letras (`TYPED`) a lo que se está escribiendo. En escritorio llegan por winit.
    fn text_input(&mut self) {
        let want = self.ui.prompt.is_some() || self.ui.chat.is_some() || (self.creating() && self.ui.create.focus.is_some());
        let again = std::mem::take(&mut self.kb_again);
        if cfg!(target_os = "android") && (want != self.kb_shown || (want && again)) {
            #[cfg(target_os = "android")]
            if let Err(e) = with_activity(|env, act| env.call_method(act, "setKeyboard", "(Z)V", &[jni::objects::JValue::Bool(want.into())]).map(|_| ())) {
                log::warn!("teclado del sistema: {e}");
            }
            self.kb_shown = want;
        }
        #[cfg(target_os = "android")]
        for ch in std::mem::take(&mut *TYPED.lock().unwrap()).chars() {
            let s = ch.to_string();
            if self.ui.prompt.is_some() {
                self.ui.prompt_type(&s);
            } else if self.ui.chat.is_some() {
                self.ui.chat_type(&s);
            } else if self.creating() {
                self.ui.create.key(KeyCode::Space, Some(&s)); // Space: cualquier tecla que no sea Enter, Tab ni borrar
            }
        }
    }

    /// Unirse a `target` (`net::resolve`: "IP:puerto", "dominio:puerto"...) con el personaje elegido en la lista;
    /// `pass` = la clave de la sala ("" si no pide). El DNS y el connect van en otro hilo: la pantalla sigue
    /// dibujando "Conectando..." y `net_pump` recoge el resultado.
    fn join(&mut self, target: String, pass: String) {
        if self.connecting.is_some() {
            return;
        }
        let Some(c) = self.ui.roster_sel.and_then(|i| self.roster.get(i)).cloned() else {
            self.lan_status = tr!("Elige un personaje en la lista.", "Choose a character from the list.").into();
            return;
        };
        // personaje portátil: entra con lo de su partida (si tiene una con el héroe vivo)
        let run = save::load_run(&data_dir(), characters::id(&c)).filter(|d| d.heroes.iter().any(|h| h.is_alive()));
        let hero = run.map(|d| save::portable(d.local()));
        self.join_addr = Some(target.clone());
        let (tx, rx) = std::sync::mpsc::channel();
        let ch = c.clone();
        std::thread::spawn(move || {
            let _ = tx.send(net::resolve(&target).and_then(|a| net::ClientGame::connect(a, &ch, hero, &pass)));
        });
        self.connecting = Some((rx, c));
        self.lan_status = tr!("Conectando...", "Connecting...").into();
    }

    /// La red, una vez por frame (en el hilo del juego): el host recibe acciones y manda el estado; el cliente manda
    /// lo que eligió su héroe y aplica el último estado. Los héroes sin nadie que los maneje (un jugador que se fue, o
    /// una partida guardada con otros héroes) esperan descansando.
    fn net_pump(&mut self) {
        if let Some(b) = &self.browser {
            self.lan_games = b.games();
        }
        self.ui.lan = !matches!(self.net, Net::Off);
        let got = self.connecting.as_ref().map(|(rx, _)| rx.try_recv());
        match got {
            Some(Ok(r)) => {
                let (_, c) = self.connecting.take().unwrap();
                match r {
                    Ok(cg) => {
                        self.net = Net::Client(cg);
                        self.character = Some(c);
                    }
                    Err(e) => self.lan_status = trf!("No se pudo conectar: {e}.", "Couldn't connect: {e}."),
                }
            }
            Some(Err(std::sync::mpsc::TryRecvError::Disconnected)) => self.connecting = None,
            _ => {}
        }
        self.ui.busy = self.connecting.is_some() || (self.title && matches!(self.net, Net::Client(_)));
        match &mut self.net {
            Net::Off => {}
            Net::Host(h) => h.pump(&mut self.world.d, &self.classes),
            Net::Client(c) => {
                let d = &mut self.world.d;
                if !self.title
                    && let Some(a) = d.heroes.get_mut(d.hero).and_then(|h| h.action.take())
                {
                    c.send_action(a);
                    self.awaiting = true;
                }
                if let Some(names) = c.lobby.take() {
                    self.lan_status = trf!("En la sala: {}. Esperando que el host empiece...", "In the lobby: {}. Waiting for the host to start...", names.join(", "));
                }
                if let Some((nd, wait)) = c.pump() {
                    self.world.d = nd;
                    self.awaiting = false;
                    if self.title {
                        // primer estado: a jugar
                        self.title = false;
                        self.ui = Ui::new();
                        self.pan = [0.0; 2];
                        self.lan_status.clear();
                        self.browser = None;
                    }
                    self.ui.waiting = wait;
                }
                // animaciones, números y dados del host (después del estado: si llegó uno nuevo, van sobre ese)
                self.world.d.fx.append(&mut c.fx);
                self.world.d.dice_events.append(&mut c.dice);
                if c.closed || c.refused.is_some() {
                    let need_pass = c.need_pass;
                    let host_v = c.host_version.clone();
                    let why = c.refused.clone().map_or(tr!("Se corto la conexion con el host.", "Lost the connection to the host.").to_string(), |w| trf!("El host no te dejo entrar: {w}.", "The host didn't let you in: {w}."));
                    let why = c.host_version.as_deref().map_or(why, version_msg);
                    self.autosave(); // antes de Net::Off: guarda su héroe, no la partida del host
                    self.net = Net::Off;
                    self.awaiting = false;
                    self.lan_status = why;
                    self.ui = Ui::new();
                    self.ui.in_title = true;
                    self.ui.window = Some(UiWindow::Join);
                    if need_pass {
                        self.ui.prompt = Some((ui::Ask::Pass, String::new()));
                    }
                    if let Some(v) = host_v.filter(|v| updater::ENABLED && net::newer(v, game::VERSION)) {
                        self.offer = Some(v); // ACTUALIZAR baja la versión del host, no la última
                    }
                    self.title = true;
                    if self.browser.is_none() {
                        self.browser = net::Browser::new().ok();
                    }
                }
            }
        }
        if matches!(self.net, Net::Client(_)) {
            return;
        }
        self.ui.waiting = self.world.d.heroes.iter().map(net::needs_input).collect();
        if self.config.hotseat || self.title {
            return;
        }
        let d = &mut self.world.d;
        for j in 0..d.heroes.len() {
            let controlled = j == d.hero || matches!(&self.net, Net::Host(h) if h.controls(j));
            let h = &mut d.heroes[j];
            if !controlled && h.is_alive() && !h.is_downed() && h.action.is_none() {
                h.action = Some(Action::Rest);
            }
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

    /// Partida nueva del personaje en juego (R, o Nueva partida al morir).
    fn new_world(&mut self, seed: i64) {
        if !matches!(self.net, Net::Off) {
            return; // en red, la partida no se reemplaza (R, nueva partida)
        }
        let c = self.character.clone().unwrap_or_default();
        let class = c["class"].as_str().unwrap_or(DEFAULT_CLASS);
        let name = c["name"].as_str().unwrap_or("");
        self.world = World::new(seed, class, characters::abilities(&c), name, characters::skin(&c), self.config.hotseat);
        self.saved_depth = self.world.d.zone;
        self.ui = Ui::new();
        self.pan = [0.0; 2];
    }

    /// Lee la lista de personajes del disco y arma sus filas (con la partida guardada de cada uno, si sigue vivo).
    fn open_roster(&mut self) {
        let dir = data_dir();
        self.roster = characters::load(&dir);
        self.roster_rows = self
            .roster
            .iter()
            .map(|c| {
                let name = c["name"].as_str().unwrap_or("?").to_string();
                let class = c["class"].as_str().unwrap_or("");
                let Some(cls) = self.classes.get(class) else {
                    // clase de un mod que no está cargado (o del prototipo): no se puede jugar
                    return ui::RosterRow { name, status: trf!("clase desconocida: {class}", "unknown class: {class}"), stats: String::new(), sheet: None };
                };
                let run = save::load_run(&dir, characters::id(c)).filter(|d| d.heroes.iter().any(|h| h.is_alive()));
                let status = match run {
                    Some(d) => trf!("en partida - piso {}, nivel {}", "in a run - depth {}, level {}", d.zone.z, d.local().lvl),
                    None => format!("{} d{}", game::registry::text(game::hero::class_5e(cls)), cls["hit_die"]),
                };
                let ab = characters::abilities(c);
                let stats: Vec<String> = (0..6).map(|k| format!("{} {}", game::dice::ability_names()[k], ab[k])).collect();
                let sheet = skin_sheet(&self.skins, characters::skin(c)).or_else(|| skin_sheet(&self.skins, class));
                ui::RosterRow { name, status, stats: stats.join("  "), sheet }
            })
            .collect();
        self.ui.roster_sel = (!self.roster.is_empty()).then(|| self.ui.roster_sel.unwrap_or(0).min(self.roster.len() - 1));
        self.ui.confirm = false;
        self.ui.window = Some(UiWindow::Roster);
    }

    /// Juega con un personaje: su partida guardada si sigue vivo; si no, una nueva.
    fn start(&mut self, c: serde_json::Value) {
        if !self.classes.contains_key(c["class"].as_str().unwrap_or("")) {
            return; // clase que no está cargada
        }
        let run = save::load_run(&data_dir(), characters::id(&c)).filter(|d| d.heroes.iter().any(|h| h.is_alive()));
        self.character = Some(c);
        match run {
            Some(mut d) => {
                // los héroes de otra sesión LAN no siguen en la partida (salvo en el hotseat, que son de esta PC)
                if !self.config.hotseat {
                    for j in (0..d.heroes.len()).rev() {
                        if j != d.hero {
                            d.remove_hero(j);
                        }
                    }
                }
                self.world = World::from_dungeon(d);
                self.saved_depth = self.world.d.zone;
            }
            None => self.new_world(new_seed()),
        }
        self.ui = Ui::new();
        self.pan = [0.0; 2];
        self.title = false;
    }

    /// Guarda la partida del personaje en juego (si murió, se borra el guardado). En una partida LAN ajena, la partida
    /// es del host: lo que viaja con el personaje (nivel, XP, oro, equipo) vuelve a su partida de un jugador (o a una
    /// nueva si no tenía).
    fn autosave(&mut self) {
        if self.title {
            return;
        }
        let Some(c) = &self.character else { return };
        let (dir, id) = (data_dir(), characters::id(c));
        let r = if matches!(self.net, Net::Client(_)) {
            let h = self.world.d.local();
            if !h.is_alive() {
                save::delete_run(&dir, id);
                return;
            }
            let p = save::portable(h);
            let mut run = save::load_run(&dir, id).filter(|d| d.heroes.iter().any(|h| h.is_alive())).unwrap_or_else(|| {
                let class = c["class"].as_str().unwrap_or(DEFAULT_CLASS);
                World::new(new_seed(), class, characters::abilities(c), c["name"].as_str().unwrap_or(""), characters::skin(c), false).d
            });
            let me = run.hero;
            save::set_portable(&mut run, me, &p);
            save::save_run(&dir, &mut run, id)
        } else {
            save::save_run(&dir, &mut self.world.d, id)
        };
        if let Err(e) = r {
            log::warn!("no se pudo guardar la partida: {e}");
        }
    }

    /// La pantalla de título: sólo la capa de UI.
    fn frame_title(&mut self, dt: f32) {
        #[cfg(target_os = "android")]
        if let Some(r) = mods::poll(&data_dir()) {
            self.mod_imported(r);
        }
        let Some(gpu) = self.gpu.as_ref() else { return };
        let (uw, uh) = gpu.ui_size();
        self.ui.in_title = true;
        self.sprites.clear();
        let w = &self.world;
        let classes = &self.class_rows;
        let skins = &self.skin_rows;
        let rows = &self.roster_rows;
        let lobby = match &self.net {
            Net::Host(h) if matches!(self.ui.window, Some(UiWindow::Lobby)) => Some(h.names(&w.d)),
            _ => None,
        };
        let lan: Vec<(String, bool)> = if let Some(names) = lobby {
            names.into_iter().map(|n| (n, true)).collect()
        } else {
            self.lan_games
            .iter()
            .map(|g| {
                let i = &g.info;
                let ok = i["protocol"] == net::PROTOCOL && i["hash"] == game::registry::content_hash().as_str();
                let name = i["name"].as_str().unwrap_or("?");
                let mut extra = if ok { String::new() } else { format!(" (v{})", i["version"].as_str().unwrap_or("?")) };
                if i["locked"] == true {
                    extra += tr!(" (clave)", " (password)");
                }
                (trf!("{name} - {} jugadores - {}{extra}", "{name} - {} players - {}{extra}", i["players"], g.addr.ip()), ok)
            })
            .collect()
        };
        let status = &self.lan_status;
        self.ui.draw_title(&self.atlas, &self.font, &w.chars, &w.d, &self.config, &mut self.sprites, uw as f32, uh as f32, dt, classes, skins, rows, &lan, status);
        let end = self.sprites.len() as u32;
        self.gpu.as_mut().unwrap().draw(&self.sprites, &[], 0..end);
    }

    fn frame(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        if let Some(gpu) = &self.gpu {
            self.config.ui_base = gpu.ui_zoom();
        }
        self.net_pump();
        self.text_input();
        self.update_pump();
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
        self.world.d.verbose_rolls = self.config.dice_log; // [dev] dice_log
        let w = &mut self.world;
        if w.ready() && !self.awaiting {
            let h = self.input.held;
            let (dx, dy) = ((h[3] as i32) - (h[2] as i32), (h[1] as i32) - (h[0] as i32));
            if (dx, dy) != (0, 0) {
                w.step(dx, dy);
            }
        }
        w.update(dt, matches!(self.net, Net::Client(_)));
        if self.config.hotseat && matches!(self.net, Net::Off) {
            w.auto_switch();
        }
        if w.d.zone != self.saved_depth {
            self.saved_depth = w.d.zone;
            self.autosave(); // como el prototipo: al cambiar de piso
        }
        let w = &mut self.world;
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
        map::water(s, &self.atlas, self.atlas.rects[WATER], w.d.level(), ox, oy, self.water_off);
        map::tiles(s, &self.atlas, self.atlas.rects[TILES], w.d.level(), ox, oy);
        ui::draw_heaps(s, &self.atlas, &self.font, &w.d, ox, oy);
        w.chars.draw_world(&w.d, &self.atlas, s, ox, oy);
        let atlas_end = s.len() as u32;
        s.push(map::fog_quad(w.d.level(), ox, oy));
        let fog_end = s.len() as u32;
        w.chars.draw_over(&w.d, &self.atlas, &self.font, s, ox, oy);
        let world_end = s.len() as u32;

        // --- UI: log, panel de estado, carrusel, barra de herramientas, ventanas, tooltips | dados ---
        let (uw, uh) = gpu.ui_size();
        let (uw, uh) = (uw as f32, uh as f32);
        self.ui.draw(&self.atlas, &self.font, &w.chars, &w.d, &self.config, s, uw, uh, dt, self.fps);
        // en vertical el log ocupa todo el ancho: los dados chicos van arriba de él
        let bottom = if uh > uw { ui::LOG_AREA + 2.0 } else { 0.0 };
        self.world.dice.draw(&self.atlas, &self.font, &mut self.sprites, uw, uh, bottom);
        let ui_end = self.sprites.len() as u32;

        let batches = [(Tex::Atlas, 0..atlas_end), (Tex::Fog, atlas_end..fog_end), (Tex::Atlas, fog_end..world_end)];
        self.gpu.as_mut().unwrap().draw(&self.sprites, &batches, world_end..ui_end);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        // Android destruye la superficie al irse al fondo: se recrea ventana y GPU en cada resumed
        let mut attrs = Window::default_attributes().with_title(TITLE);
        if !cfg!(target_os = "android") {
            let (w, h) = self.config.window_size();
            attrs = attrs.with_inner_size(PhysicalSize::new(w, h));
        }
        let window = Arc::new(el.create_window(attrs).expect("ventana"));
        let mut gpu = pollster::block_on(Gpu::new(window, &self.atlas));
        gpu.set_ui_scale(self.config.ui_scale);
        self.gpu = Some(gpu);
        self.world.d.view_changed = true; // la textura de niebla es nueva
        self.apply_fullscreen();
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.autosave(); // Android puede cerrar la app estando en el fondo
        self.gpu = None;
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.autosave();
                el.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => self.frame(),
            WindowEvent::KeyboardInput { event, .. } => {
                // el botón Atrás de Android hace lo mismo que Esc (cerrar la ventana, volver, menú)
                let back = event.logical_key == Key::Named(NamedKey::BrowserBack);
                let code = match event.physical_key {
                    _ if back => Some(KeyCode::Escape),
                    PhysicalKey::Code(c) => Some(c),
                    _ => None,
                };
                if let Some(code) = code {
                    let down = event.state == ElementState::Pressed;
                    // chat de la partida LAN: mientras se escribe, las teclas son texto
                    if down && !self.title && self.ui.chat.is_some() {
                        let cr = match code {
                            KeyCode::Escape => Some(ui::Cr::Unfocus),
                            KeyCode::Enter | KeyCode::NumpadEnter => Some(ui::Cr::Done),
                            KeyCode::Backspace => Some(ui::Cr::Back),
                            _ => None,
                        };
                        match (cr, &event.logical_key) {
                            (Some(cr), _) => self.ui_action(UiAction::Chat(cr)),
                            (None, Key::Character(t)) if !cfg!(target_os = "android") => self.ui.chat_type(t),
                            _ => {}
                        }
                        return;
                    }
                    // título: la dirección o la clave que se está escribiendo (Unirse, sala de espera)
                    if down && self.title && self.ui.prompt.is_some() {
                        let cr = match code {
                            KeyCode::Escape => Some(ui::Cr::Unfocus),
                            KeyCode::Enter | KeyCode::NumpadEnter => Some(ui::Cr::Done),
                            KeyCode::Backspace => Some(ui::Cr::Back),
                            _ => None,
                        };
                        match (cr, &event.logical_key) {
                            (Some(cr), _) => self.ui_action(UiAction::Prompt(cr)),
                            (None, Key::Character(t)) if !cfg!(target_os = "android") => self.ui.prompt_type(t),
                            _ => {}
                        }
                        return;
                    }
                    // creación de personaje: el nombre o el número que se está escribiendo
                    if down && self.creating() && code != KeyCode::Escape {
                        let text = match &event.logical_key {
                            Key::Character(t) if !cfg!(target_os = "android") => Some(t.as_str()),
                            _ => None,
                        };
                        if self.ui.create.key(code, text) {
                            return;
                        }
                    }
                    self.key(code, down);
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
                if y != 0.0 && self.title {
                    // lista de personajes: la rueda la desplaza
                    let s = &mut self.ui.roster_scroll;
                    *s = if y > 0.0 { s.saturating_sub(1) } else { *s + 1 };
                } else if y != 0.0 {
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
    // Escritorio: Local.config en la carpeta de datos, no donde se corre el binario (Descargas, Archivos de programa).
    // La primera vez se copia el de la carpeta actual, si hay. En Android ya lo puso `android_main`.
    if config::DIR.get().is_none() {
        let dir = data_dir();
        let to = dir.join("Local.config");
        if !to.exists() && std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::copy("Local.config", &to);
        }
        let _ = config::DIR.set(dir);
    }
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut App::new()).expect("bucle de eventos");
}

/// Android: las letras del teclado del sistema (`MainActivity.dispatchKeyEvent` -> `native_char`), hasta el próximo
/// frame.
#[cfg(target_os = "android")]
static TYPED: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

#[cfg(target_os = "android")]
extern "system" fn native_char(_: jni::JNIEnv, _: jni::objects::JClass, c: jni::sys::jint) {
    if let (Some(ch), Ok(mut t)) = (u32::try_from(c).ok().and_then(char::from_u32), TYPED.lock()) {
        t.push(ch);
    }
}

/// Android: la actividad por JNI (para llamarla o registrar `nativeChar`).
#[cfg(target_os = "android")]
fn with_activity<T>(f: impl FnOnce(&mut jni::JNIEnv, &jni::objects::JObject) -> jni::errors::Result<T>) -> Result<T, String> {
    let app = ANDROID_APP.get().ok_or("sin actividad")?;
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let act = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr().cast()) };
    f(&mut env, &act).map_err(|e| e.to_string())
}

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<winit::platform::android::activity::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let _ = ANDROID_APP.set(app.clone()); // para llamar a MainActivity (importar mods, teclado)
    let reg = with_activity(|env, act| {
        let class = env.get_object_class(act)?;
        let m = jni::NativeMethod { name: "nativeChar".into(), sig: "(I)V".into(), fn_ptr: native_char as *mut std::ffi::c_void };
        env.register_native_methods(&class, &[m])
    });
    if let Err(e) = reg {
        log::warn!("teclado del sistema: {e}");
    }
    if let Some(dir) = app.internal_data_path() {
        let _ = config::DIR.set(dir); // Local.config en la carpeta de datos de la app
    }
    run(EventLoop::builder().with_android_app(app).build().expect("bucle de eventos"));
    // Salir: Android deja vivo el proceso y al volver a abrir la app la actividad espera un bucle que ya terminó
    // (pantalla trabada). Se cierra el proceso entero; la partida ya se guardó al salir
    std::process::exit(0);
}
