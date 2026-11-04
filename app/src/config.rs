//! Configuración local (`core/Config.py`): `Local.config`, el mismo INI del prototipo. Se leen y guardan sólo las
//! claves que usa Rust (FPS, pantalla completa, controles); el resto del archivo se conserva tal cual.
//! En escritorio está en la carpeta desde donde se corre el juego; en Android, en la carpeta de datos de la app.
//! El idioma (`language`) es el del contenido (los JSON); los textos del código están en español, como el prototipo.
//! ponytail: sin traducción de los textos del código (falta la estructura); English sólo cambia el contenido
use std::path::PathBuf;
use std::sync::OnceLock;

use winit::keyboard::KeyCode;

/// Carpeta de la configuración en Android (`AndroidApp::internal_data_path`).
pub static DIR: OnceLock<PathBuf> = OnceLock::new();
/// FPS que se pueden elegir; 0 = sin límite.
pub const FPS_OPTIONS: [u32; 4] = [30, 60, 120, 0];
/// Sin archivo: 60 (el prototipo usaba 30; en un panel de 120 Hz se nota).
const DEFAULT_FPS: u32 = 60;
/// Tamaño de la ventana (`resolution` en Local.config, `core/constants/Resolutions.py`): (clave, ancho, alto).
pub const RESOLUTIONS: [(&str, u32, u32); 3] = [("Basic", 640, 480), ("Hd", 1280, 720), ("Fhd", 1920, 1080)];

/// Acciones con tecla configurable. Las flechas siempre mueven; Esc, R, C, F y F11 son fijas.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bind {
    Up,
    Down,
    Left,
    Right,
    Wait,
    Search,
    Bag,
    Quick1,
    Quick2,
}

impl Bind {
    pub const ALL: [Bind; 9] =
        [Bind::Up, Bind::Down, Bind::Left, Bind::Right, Bind::Wait, Bind::Search, Bind::Bag, Bind::Quick1, Bind::Quick2];

    pub fn label(self) -> &'static str {
        match self {
            Bind::Up => "Arriba",
            Bind::Down => "Abajo",
            Bind::Left => "Izquierda",
            Bind::Right => "Derecha",
            Bind::Wait => "Esperar",
            Bind::Search => "Buscar",
            Bind::Bag => "Mochila",
            Bind::Quick1 => "Rapido 1",
            Bind::Quick2 => "Rapido 2",
        }
    }

    /// Clave en la sección [keys] de Local.config.
    fn ini(self) -> &'static str {
        match self {
            Bind::Up => "up",
            Bind::Down => "down",
            Bind::Left => "left",
            Bind::Right => "right",
            Bind::Wait => "wait",
            Bind::Search => "search",
            Bind::Bag => "bag",
            Bind::Quick1 => "quick1",
            Bind::Quick2 => "quick2",
        }
    }

    fn default_key(self) -> KeyCode {
        match self {
            Bind::Up => KeyCode::KeyW,
            Bind::Down => KeyCode::KeyS,
            Bind::Left => KeyCode::KeyA,
            Bind::Right => KeyCode::KeyD,
            Bind::Wait => KeyCode::Space,
            Bind::Search => KeyCode::KeyQ,
            Bind::Bag => KeyCode::KeyI,
            Bind::Quick1 => KeyCode::Digit1,
            Bind::Quick2 => KeyCode::Digit2,
        }
    }
}

/// Teclas que se pueden asignar (se guardan por su nombre en winit: "KeyW", "Space"...).
const KEYS: [KeyCode; 73] = {
    use KeyCode::*;
    [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS,
        KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ, Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8,
        Digit9, Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9, ArrowUp,
        ArrowDown, ArrowLeft, ArrowRight, Space, Enter, Tab, Backspace, ShiftLeft, ShiftRight, ControlLeft, ControlRight,
        AltLeft, Comma, Period, Slash, Semicolon, Quote, BracketLeft, BracketRight, Minus, Equal, Backquote, Backslash,
        Home, End, Delete,
    ]
};

fn key_name(k: KeyCode) -> String {
    format!("{k:?}")
}

/// Nombre corto para la UI: "W", "1", "Space".
pub fn key_label(k: KeyCode) -> String {
    let n = key_name(k);
    n.strip_prefix("Key").or_else(|| n.strip_prefix("Digit")).unwrap_or(&n).to_string()
}

/// ¿Se puede asignar esta tecla? (las fijas no).
pub fn assignable(k: KeyCode) -> bool {
    KEYS.contains(&k) && !matches!(k, KeyCode::KeyR | KeyCode::KeyC | KeyCode::KeyF)
}

pub struct Config {
    pub max_fps: u32,
    /// Contador de FPS en la partida (`showFPS`; sin archivo, apagado).
    pub show_fps: bool,
    pub fullscreen: bool,
    /// Idioma del contenido (código de `game::registry::LANGS`; sin archivo, "es").
    pub language: &'static str,
    /// `[dev] dice_log`: el log muestra el cálculo de cada tirada (opción de desarrollador, sólo en el archivo).
    pub dice_log: bool,
    /// `[dev] hotseat`: modo de prueba de la red, 2 héroes en la misma PC (Tab cambia). Sólo en el archivo.
    pub hotseat: bool,
    /// Clave de `RESOLUTIONS` (sin archivo o desconocida: "Hd", como el prototipo).
    pub resolution: &'static str,
    /// Escala de la interfaz respecto del zoom que entra (1 = 100%; `ui_scale` en Local.config, como el prototipo).
    pub ui_scale: f32,
    /// El zoom entero que entra en la ventana (lo pone la app cada frame; no se guarda).
    pub ui_base: u32,
    pub keys: Vec<(Bind, KeyCode)>,
    /// Las líneas del archivo (se conserva lo que Rust no usa).
    lines: Vec<String>,
}

fn path() -> PathBuf {
    DIR.get().map_or_else(|| PathBuf::from("Local.config"), |d| d.join("Local.config"))
}

impl Config {
    pub fn load() -> Self {
        let lines: Vec<String> = std::fs::read_to_string(path()).unwrap_or_default().lines().map(str::to_string).collect();
        let mut c = Self { max_fps: DEFAULT_FPS, show_fps: false, fullscreen: false, language: "es", dice_log: false, hotseat: false, resolution: "Hd", ui_scale: 1.0, ui_base: 1, keys: Vec::new(), lines };
        if let Some(s) = c.get("general", "ui_scale").and_then(|v| v.parse().ok()) {
            c.ui_scale = s;
        }
        if let Some(f) = c.get("general", "maxFPS").and_then(|v| v.parse().ok()) {
            c.max_fps = f;
        }
        if let Some(l) = c.get("general", "language").and_then(|v| game::registry::LANGS.iter().find(|l| l.0 == v)) {
            c.language = l.0;
        }
        c.dice_log = c.get("dev", "dice_log").is_some_and(|v| v.eq_ignore_ascii_case("true"));
        c.hotseat = c.get("dev", "hotseat").is_some_and(|v| v.eq_ignore_ascii_case("true"));
        if let Some(r) = c.get("general", "resolution").and_then(|v| RESOLUTIONS.iter().find(|r| r.0 == v)) {
            c.resolution = r.0;
        }
        c.show_fps = c.get("general", "showFPS").is_some_and(|v| v.eq_ignore_ascii_case("true"));
        c.fullscreen = c.get("general", "fullscreen").is_some_and(|v| v.eq_ignore_ascii_case("true"));
        for b in Bind::ALL {
            let k = c.get("keys", b.ini()).and_then(|n| KEYS.iter().copied().find(|&k| key_name(k) == n));
            c.keys.push((b, k.unwrap_or(b.default_key())));
        }
        c
    }

    pub fn key(&self, b: Bind) -> KeyCode {
        self.keys.iter().find(|e| e.0 == b).map_or(b.default_key(), |e| e.1)
    }

    /// La acción de una tecla.
    pub fn bind_of(&self, k: KeyCode) -> Option<Bind> {
        self.keys.iter().find(|e| e.1 == k).map(|e| e.0)
    }

    /// Asigna la tecla; si otra acción la tenía, se intercambian (nunca quedan dos con la misma).
    pub fn rebind(&mut self, b: Bind, k: KeyCode) {
        let old = self.key(b);
        for e in &mut self.keys {
            if e.1 == k {
                e.1 = old;
            }
        }
        for e in &mut self.keys {
            if e.0 == b {
                e.1 = k;
            }
        }
        for (b, k) in self.keys.clone() {
            self.set("keys", b.ini(), &key_name(k));
        }
        self.save();
    }

    pub fn set_fps(&mut self, fps: u32) {
        self.max_fps = fps;
        self.set("general", "maxFPS", &fps.to_string());
        self.save();
    }

    pub fn set_show_fps(&mut self, on: bool) {
        self.show_fps = on;
        self.set("general", "showFPS", if on { "true" } else { "false" });
        self.save();
    }

    pub fn set_ui_scale(&mut self, s: f32) {
        self.ui_scale = s;
        self.set("general", "ui_scale", &s.to_string());
        self.save();
    }

    /// Tamaño de la ventana en pixeles.
    pub fn window_size(&self) -> (u32, u32) {
        RESOLUTIONS.iter().find(|r| r.0 == self.resolution).map_or((1280, 720), |r| (r.1, r.2))
    }

    pub fn set_language(&mut self, code: &'static str) {
        self.language = code;
        self.set("general", "language", code);
        self.save();
    }

    pub fn set_resolution(&mut self, r: &'static str) {
        self.resolution = r;
        self.set("general", "resolution", r);
        self.save();
    }

    pub fn set_fullscreen(&mut self, on: bool) {
        self.fullscreen = on;
        self.set("general", "fullscreen", if on { "true" } else { "false" });
        self.save();
    }

    fn get(&self, section: &str, key: &str) -> Option<String> {
        let mut cur = String::new();
        for l in &self.lines {
            let t = l.trim();
            if t.starts_with('[') && t.ends_with(']') {
                cur = t[1..t.len() - 1].to_string();
            } else if cur == section
                && let Some((k, v)) = t.split_once('=')
                && k.trim() == key
            {
                return Some(v.trim().to_string());
            }
        }
        None
    }

    /// Cambia la clave en su lugar, o la agrega al final de su sección (o la sección al final del archivo).
    fn set(&mut self, section: &str, key: &str, value: &str) {
        let line = format!("{key} = {value}");
        let mut cur = String::new();
        let mut end = None; // dónde termina la sección
        for (i, l) in self.lines.iter().enumerate() {
            let t = l.trim();
            if t.starts_with('[') && t.ends_with(']') {
                cur = t[1..t.len() - 1].to_string();
                if cur == section {
                    end = Some(i + 1);
                }
                continue;
            }
            if cur == section {
                if t.split_once('=').is_some_and(|(k, _)| k.trim() == key) {
                    self.lines[i] = line;
                    return;
                }
                if !t.is_empty() {
                    end = Some(i + 1);
                }
            }
        }
        match end {
            Some(i) => self.lines.insert(i, line),
            None => {
                if self.lines.last().is_some_and(|l| !l.trim().is_empty()) {
                    self.lines.push(String::new());
                }
                self.lines.push(format!("[{section}]"));
                self.lines.push(line);
            }
        }
    }

    /// Guarda en el acto; si no se puede (sin permiso), sigue con lo elegido en memoria.
    fn save(&self) {
        if let Err(e) = std::fs::write(path(), self.lines.join("\n") + "\n") {
            log::warn!("no se pudo guardar {}: {e}", path().display());
        }
    }
}
