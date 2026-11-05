//! UI de la partida (`screens/gameUi.py` + el log y examinar de `screens/gameScreen.py`) con los assets originales:
//! panel de estado, carrusel de iniciativa, barra de herramientas, log, tooltips y ventanas (examinar, ficha, menú).
//! UI inmediata: cada frame se dibuja todo y se registran zonas clickeables; el click busca de arriba hacia abajo.
//! R5: mochila (WndBag), ventana del ítem (WndItem), accesos rápidos e ícono de hambre.
//! ponytail: sin íconos de estados ni configuración todavía (R5)
use game::actor::ActorId;
use game::dice::{self, ABILITIES, ABILITY_NAMES};
use game::dungeon::Dungeon;
use game::npcs::DialogView;
use game::geom::LENGTH;
use game::hero::{Action, Slot, Verb, BACKPACK_SIZE, HUNGRY};
use game::items::{weapon_dice, Image, Item, SLOT_ARMOR, SLOT_RING, SLOT_WEAPON};
use game::registry::text;
use game::terrain;

use crate::atlas::Atlas;
use crate::font::Font;
use crate::gpu::Sprite;
use crate::config::{key_label, Bind, Config, FPS_OPTIONS, RESOLUTIONS};
use crate::sprites::Chars;
use crate::{ARCS_BG, ARCS_FG, BANNERS, BUFFS, CHROME, DASHBOARD, EXP_BAR, HP_BAR, ICONS, ITEMS, STATUS, TILES, TOOLBAR, WATER};

const LINE_H: f32 = 10.0;
const LOG_LINES: usize = 4;
const LOG_LINE_H: f32 = 9.0;
const TOOLBAR_H: f32 = 25.0;
/// WndInfoCell / WndInfoMob.
const INFO_W: f32 = 120.0;
/// WndGame.
const MENU_W: f32 = 120.0;
const MENU_BTN_H: f32 = 20.0;
const MENU_GAP: f32 = 2.0;
/// Chrome.java: (recorte en chrome.png, margen del nine-patch).
const CHROME_WINDOW: ([u32; 4], f32) = ([0, 0, 22, 22], 7.0);
const CHROME_BUTTON: ([u32; 4], f32) = ([58, 0, 6, 6], 2.0);
/// Toolbar.java: recortes de toolbar.png.
const TOOL_WAIT: [u32; 4] = [0, 7, 20, 25];
const TOOL_SEARCH: [u32; 4] = [20, 7, 20, 25];
const TOOL_INFO: [u32; 4] = [40, 7, 21, 25];
/// Icons.java.
const ICON_BUSY: [u32; 4] = [8, 0, 8, 8];
const ICON_CHECKED: [u32; 4] = [54, 12, 12, 12];
const ICON_UNCHECKED: [u32; 4] = [66, 12, 12, 12];
const ICON_EXIT: [u32; 4] = [98, 0, 16, 16];
/// Icons.WARRIOR / MAGE / ROGUE / HUNTRESS: 16x16 desde (0, 29), uno al lado del otro.
const ICON_CLASS_Y: u32 = 29;
const TOOL_BAG: [u32; 4] = [60, 7, 23, 25];
const TOOL_QUICK: [u32; 4] = [83, 7, 22, 25];
/// BuffIndicator.java: íconos de 7x7 en buffs.png; hambre y famélico son el 5 y el 6.
const BUFF_SIZE: u32 = 7;
const BUFF_HUNGER: u32 = 5;
const BUFF_STARVATION: u32 = 6;
/// Mochila: 1a fila = equipo (arma, armadura, 2 anillos) + oro aparte a la derecha; debajo, las ranuras de mochila.
const SLOT_SIZE: f32 = 28.0;
const SLOT_MARGIN: f32 = 1.0;
const BAG_COLS: usize = 7;
const BAG_TITLE_H: f32 = 12.0;
const BAG_GAP: f32 = 4.0;
const ITEM_WND_W: f32 = 170.0;
/// Carrusel de iniciativa: lado de cada casilla, cuántos retratos, velocidad del deslizamiento (casillas/s), alto.
const CAROUSEL_SLOT: f32 = 16.0;
const CAROUSEL_MAX: usize = 7;
const CAROUSEL_SPEED: f32 = 8.0;
const CAROUSEL_Y: f32 = 30.0;

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
}

const WHITE: [f32; 4] = rgba(0xFF, 0xFF, 0xFF, 0xFF);
const GREY: [f32; 4] = rgba(0x88, 0x88, 0x88, 0xFF);
const YELLOW: [f32; 4] = rgba(0xFF, 0xFF, 0x44, 0xFF);
const GOLDEN: [f32; 4] = rgba(0xFF, 0xEB, 0xA4, 0xFF);
const DEPTH_COLOR: [f32; 4] = rgba(0xCA, 0xCF, 0xC2, 0xFF);
const HERO_BOX: [f32; 4] = rgba(0x22, 0x44, 0x66, 0xCC);
const MOB_BOX: [f32; 4] = rgba(0x55, 0x22, 0x22, 0xCC);
const DEAD_TEXT: [f32; 4] = rgba(255, 80, 80, 255);
const DOWN_TEXT: [f32; 4] = rgba(255, 136, 0, 255);
const HP_RED: [f32; 4] = rgba(0xCC, 0, 0, 0xFF);
const HP_GREEN: [f32; 4] = rgba(0, 0xEE, 0, 0xFF);
const TIP_BG: [f32; 4] = rgba(0, 0, 0, 200);
const PROMPT_BG: [f32; 4] = rgba(0, 0, 0, 170);
const BUTTON_OFF: [f32; 4] = rgba(110, 110, 110, 0xFF);
const STATES: [f32; 4] = rgba(0xFF, 0x88, 0x88, 0xFF);
const SLOT_BG: [f32; 4] = rgba(0x3A, 0x3C, 0x36, 0xD8);
/// La mochila y la ventana del ítem, menos brillantes que el resto (pedido del dueño): multiplica el marco.
const BAG_TINT: [f32; 4] = [0.72, 0.72, 0.72, 1.0];
/// WndSettings.
const SETTINGS_W: f32 = 200.0;
const CTRL_H: f32 = 14.0;
const SPELLS_W: f32 = 190.0;
const LEVEL_UP: [f32; 4] = rgba(80, 255, 80, 0xFF);
const LEVEL_DOWN: [f32; 4] = rgba(255, 80, 80, 0xFF);

/// Para qué se abre la mochila.
#[derive(Clone, PartialEq)]
pub enum BagMode {
    Browse,
    /// Elegir el objetivo de un pergamino (identificar, mejorar).
    Target { scroll: Slot, need: String },
    /// Elegir qué va en el acceso rápido.
    Quick(usize),
}

/// Ícono de la ventana de examinar.
#[derive(Clone)]
pub enum Icon {
    Tile(u8),
    Char(ActorId),
}

#[derive(Clone)]
pub enum Window {
    /// WndInfoCell / WndInfoMob: ícono, título, texto y vida (0-1) si es un mob.
    Info { icon: Option<Icon>, title: String, text: String, hp: Option<f32> },
    /// Ficha 5e del personaje (WndHero).
    Hero,
    /// Menú de la partida (WndGame).
    Menu,
    Bag(BagMode),
    /// WndItem: un ítem del héroe con sus acciones.
    Item(Slot),
    /// WndSettings: FPS, pantalla completa y controles.
    Settings,
    /// Libro de conjuros (F).
    Spells,
    /// Reglas de la partida (como la configuración de mundo de Don't Starve Together).
    Rules,
    /// Controles (desde la configuración).
    Controls,
    /// Pantalla de texto del título (Mods, Acerca de): centrada sobre negro; click o Esc vuelve.
    Page(Vec<(String, [f32; 4])>),
    /// Lista de personajes (Jugar): JUGAR / NUEVO / BORRAR, CREAR LAN / UNIRSE.
    Roster,
    /// Partidas encontradas en la LAN (UNIRSE).
    Join,
    /// Sala de espera del host (CREAR LAN): quién se unió, EMPEZAR / VOLVER.
    Lobby,
    /// Creación de personaje (NUEVO): nombre, clase y características con 4d6 (`Ui::create`).
    Create,
}

/// Lo que hace un click en la UI; lo aplica la app.
#[derive(Clone)]
pub enum UiAction {
    /// Zona que sólo frena el click (el fondo de una ventana).
    Nothing,
    Close,
    Open(Window),
    Act(Action),
    /// Activa / cancela examinar: el próximo click en el mapa examina esa casilla.
    Examine,
    NewGame,
    Quit,
    /// Usar el acceso rápido (vacío: elegir qué va).
    UseQuick(usize),
    SetQuick(usize, String),
    /// Un verbo de la ventana del ítem (leer con objetivo primero pide el objetivo).
    Verb(Slot, Verb),
    SetFps(u32),
    SetShowFps(bool),
    SetFullscreen(bool),
    /// Idioma del contenido (código de `game::registry::LANGS`).
    SetLanguage(&'static str),
    /// Tamaño de la ventana (clave de `RESOLUTIONS`).
    SetResolution(&'static str),
    /// Esperar la próxima tecla para esa acción.
    Rebind(Bind),
    /// Elegir el objetivo del conjuro k en el mapa.
    Cast(usize),
    /// Creación de personaje (la app aplica `Cr::Start`; el resto, `Create::apply`).
    Create(Cr),
    /// Partida: volver a la pantalla de título.
    MainMenu,
    /// Título: abrir la lista de personajes (la app la lee del disco).
    OpenRoster,
    SelectChar(usize),
    PlayChar,
    /// Borrar el personaje elegido: el primer click pide confirmación.
    DeleteChar,
    SetRule(&'static str, bool),
    /// Jugar con el personaje elegido y abrir la partida en la LAN.
    HostLan,
    /// Buscar partidas en la LAN.
    OpenJoin,
    /// Unirse a la partida i de la lista, con el personaje elegido.
    JoinGame(usize),
    /// Sala de espera: empezar la partida LAN.
    StartLan,
    /// Rasgo de clase: Segundo Aliento o el libro de conjuros (F o el botón de la barra).
    Trait,
    /// Chat de la partida LAN: abrir (`Cr::New`), escribir, borrar, mandar (`Cr::Done`), cancelar (`Cr::Unfocus`).
    Chat(Cr),
    SetUiScale(f32),
}

struct Hot {
    rect: [f32; 4],
    act: UiAction,
    /// Click derecho (None = el mismo que el izquierdo).
    alt: Option<UiAction>,
    tip: Option<String>,
}

fn inside(r: [f32; 4], p: [f32; 2]) -> bool {
    p[0] >= r[0] && p[0] < r[0] + r[2] && p[1] >= r[1] && p[1] < r[1] + r[3]
}

/// Un recorte de una hoja del atlas estirado a `size`.
fn piece(atlas: &Atlas, sheet: usize, region: [u32; 4], pos: [f32; 2], size: [f32; 2], color: [f32; 4]) -> Sprite {
    let [sx, sy, _, _] = atlas.rects[sheet];
    let mut s = atlas.sprite(sx + region[0], sy + region[1], region[2], region[3], pos, color);
    s.size = size;
    s
}

/// NinePatch de noosa: estira `region` a `rect` conservando las esquinas de `m` pixeles.
fn nine(out: &mut Vec<Sprite>, atlas: &Atlas, region: [u32; 4], m: f32, rect: [f32; 4], color: [f32; 4]) {
    let [rx, ry, rw, rh] = region;
    let mu = m as u32;
    let cols = [(0, mu, 0.0, m), (mu, rw - 2 * mu, m, rect[2] - 2.0 * m), (rw - mu, mu, rect[2] - m, m)];
    let rows = [(0, mu, 0.0, m), (mu, rh - 2 * mu, m, rect[3] - 2.0 * m), (rh - mu, mu, rect[3] - m, m)];
    for (cx, cw, dx, dw) in cols {
        for (cy, ch, dy, dh) in rows {
            if cw > 0 && ch > 0 && dw > 0.0 && dh > 0.0 {
                out.push(piece(atlas, CHROME, [rx + cx, ry + cy, cw, ch], [rect[0] + dx, rect[1] + dy], [dw, dh], color));
            }
        }
    }
}

/// Parte el texto en líneas que entren en `width` pixeles (por palabras).
fn wrap(font: &Font, text: &str, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let test = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if !line.is_empty() && font.width(&test) > width {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = test;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

pub struct Ui {
    pub window: Option<Window>,
    pub examining: bool,
    /// Eligiendo el objetivo del conjuro k (el próximo click en el mapa).
    pub casting: Option<usize>,
    /// Eligiendo adónde lanzar esa arrojadiza (el próximo click en el mapa).
    pub throwing: Option<Slot>,
    /// En la pantalla de título (la configuración vuelve al título, no al menú de la partida).
    pub in_title: bool,
    /// Lista de personajes: el elegido, el primero que se ve y si BORRAR espera confirmación.
    pub roster_sel: Option<usize>,
    pub roster_scroll: usize,
    pub confirm: bool,
    /// Mouse en la capa de UI (tooltips); None en táctil.
    pub mouse: Option<[f32; 2]>,
    /// Esperando la tecla nueva para esta acción (configuración).
    pub rebinding: Option<Bind>,
    pub create: Create,
    /// Por héroe: ¿la partida espera que elija qué hacer? (la lista de quién falta, con 2+ héroes).
    pub waiting: Vec<bool>,
    /// En una partida LAN (muestra el botón del chat) y el mensaje que se está escribiendo.
    pub lan: bool,
    pub chat: Option<String>,
    hot: Vec<Hot>,
    car_cur: Option<ActorId>,
    car_off: f32,
    t: f32,
}

impl Ui {
    pub fn new() -> Self {
        Self { window: None, examining: false, casting: None, throwing: None, in_title: false, roster_sel: None, roster_scroll: 0, confirm: false, mouse: None, rebinding: None, create: Create::default(), waiting: Vec::new(), lan: false, chat: None, hot: Vec::new(), car_cur: None, car_off: 0.0, t: 0.0 }
    }

    /// Escribe en el mensaje del chat (si se está escribiendo uno).
    pub fn chat_type(&mut self, text: &str) {
        if let Some(c) = &mut self.chat {
            for ch in text.chars().filter(|c| !c.is_control()) {
                if c.chars().count() < crate::net::CHAT_MAX {
                    c.push(ch);
                }
            }
        }
    }

    /// La acción de la zona de más arriba bajo `p` (pixeles de la UI), o None si el click es para el mapa.
    pub fn click(&self, p: [f32; 2], alt: bool) -> Option<UiAction> {
        let h = self.hot.iter().rev().find(|h| inside(h.rect, p))?;
        Some(if alt { h.alt.clone().unwrap_or_else(|| h.act.clone()) } else { h.act.clone() })
    }

    fn hot(&mut self, rect: [f32; 4], act: UiAction, tip: Option<&str>) {
        self.hot.push(Hot { rect, act, alt: None, tip: tip.map(str::to_string) });
    }

    /// Toolbar.informer: qué hay en la casilla (el héroe, un mob o el terreno).
    pub fn examine(&mut self, d: &Dungeon, cell: i32) {
        let level = d.level();
        let c = cell as usize;
        if !(0..LENGTH).contains(&cell) || !(level.visited[c] || level.mapped[c]) {
            let text = "No sabes qué hay ahí.".to_string();
            self.window = Some(Window::Info { icon: None, title: "?".into(), text, hp: None });
            return;
        }
        if level.visible[c] {
            if cell == d.local().ch.pos {
                self.window = Some(Window::Hero);
                return;
            }
            if let Some(ActorId::Mob(id)) = d.find_char(cell) {
                let m = d.mob(id).unwrap();
                let desc = m.stats.get("desc").map_or(String::new(), text);
                let mut text = format!("{desc}\n\n{}. CA {}, PV {}/{}.", m.state.name(), m.armor_class(), m.ch.hp, m.ch.ht);
                if !m.ch.buffs.is_empty() {
                    let names: Vec<String> = m.ch.buffs.iter().map(|b| b.name()).collect();
                    text += &format!("\nEstados: {}", names.join(", "));
                }
                let hp = Some(m.ch.hp as f32 / m.ch.ht.max(1) as f32);
                let title = capitalize(&m.ch.name);
                self.window = Some(Window::Info { icon: Some(Icon::Char(ActorId::Mob(id))), title, text, hp });
                return;
            }
        }
        let (tile, name, desc) = terrain::info(level.map[c], level.water[c], level.pit[c]);
        let text = if desc[0].is_empty() { "No hay nada aquí." } else { desc[0] };
        let window = Window::Info { icon: Some(Icon::Tile(tile)), title: name[0].into(), text: text.into(), hp: None };
        self.window = Some(window);
    }

    /// Toda la UI, en la capa de UI (w x h pixeles virtuales).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, cfg: &Config, out: &mut Vec<Sprite>, w: f32, h: f32, dt: f32, fps: u32) {
        self.t += dt;
        self.hot.clear();
        self.draw_log(font, d, out, w, h);
        self.draw_status(atlas, font, chars, d, out, w, dt);
        if cfg.show_fps {
            font.text(out, &format!("FPS {fps}"), 2.0, 34.0, GREY);
        }
        self.draw_toolbar(atlas, font, d, out, w, h);
        self.draw_waiting(atlas, font, d, out, w);
        // diálogo con un NPC: tapa todo lo demás hasta que se cierre (Esc o Adiós)
        if let Some(dv) = d.dialog(d.hero) {
            self.draw_dialog(atlas, font, chars, d, out, w, h, &dv);
            return;
        }
        match self.window.clone() {
            Some(Window::Info { icon, title, text, hp }) => self.draw_info(atlas, font, chars, d, out, w, h, icon, &title, &text, hp),
            Some(Window::Hero) => self.draw_hero(atlas, font, d, out, w, h),
            Some(Window::Menu) => self.draw_menu(atlas, font, d, out, w, h),
            Some(Window::Bag(mode)) => self.draw_bag(atlas, font, d, out, w, h, &mode),
            Some(Window::Item(slot)) => self.draw_item(atlas, font, d, out, w, h, slot),
            Some(Window::Settings) => self.draw_settings(atlas, font, cfg, out, w, h),
            Some(Window::Spells) => self.draw_spells(atlas, font, d, out, w, h),
            Some(Window::Create | Window::Page(_) | Window::Roster | Window::Join | Window::Lobby) => self.window = None, // sólo en el título
            Some(Window::Rules) => self.draw_rules(atlas, font, d, out, w, h),
            Some(Window::Controls) => self.draw_controls(atlas, font, cfg, out, w, h),
            None if self.chat.is_some() => self.draw_chat(atlas, font, out, w, h),
            None => {
                self.draw_tooltip(atlas, font, out, w, h);
                let casting = self.casting.and_then(|k| d.local().spell_ids().get(k).cloned());
                let msg = casting.and_then(|sid| d.spell(&sid).map(|s| format!("Elige el objetivo: {} (Esc cancela)", text(&s["name"]))));
                let throwing = self.throwing.and_then(|s| d.local().item(s)).map(|it| d.item_title(it));
                let msg = msg.or_else(|| throwing.map(|t| format!("Elige adonde lanzar: {t} (Esc cancela)")));
                let msg = msg.or_else(|| self.examining.then(|| "Toca una casilla para examinarla".to_string()));
                if let Some(msg) = msg {
                    let msg = msg.as_str();
                    let tw = font.width(msg);
                    let x = ((w - tw - 8.0) / 2.0).floor();
                    out.push(atlas.rect([x, 34.0], [tw + 8.0, font.height() + 4.0], PROMPT_BG));
                    font.text(out, msg, x + 4.0, 36.0, WHITE);
                }
            }
        }
    }

    /// El mensaje que se está escribiendo, arriba (en el teléfono, el teclado en pantalla abajo).
    fn draw_chat(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let text = format!("Chat: {}_", self.chat.as_deref().unwrap_or(""));
        out.push(atlas.rect([0.0, 34.0], [w, font.height() + 4.0], PROMPT_BG));
        font.text(out, &text, 4.0, 36.0, WHITE);
        if !cfg!(target_os = "android") {
            let hint = "Enter manda, Esc cancela";
            font.text(out, hint, w - 4.0 - font.width(hint), 36.0 + font.height() + 6.0, GREY);
            return;
        }
        self.draw_keys(atlas, font, out, w, h, &["1234567890", "QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"], true, UiAction::Chat);
    }

    fn draw_log(&self, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let bottom = h - TOOLBAR_H - 1.0;
        let mine: Vec<&String> = d.messages.iter().filter(|m| m.0.is_none_or(|to| to == d.hero)).map(|m| &m.1).collect();
        let first = mine.len().saturating_sub(LOG_LINES);
        for (i, msg) in mine[first..].iter().enumerate() {
            font.text(out, msg, 2.0, bottom - (LOG_LINES - i) as f32 * LOG_LINE_H, WHITE);
        }
        let me = d.local();
        let center = if !d.heroes.iter().any(|h| h.is_alive()) {
            Some(("Has muerto - R nueva partida, Esc menu".to_string(), DEAD_TEXT))
        } else if me.is_downed() {
            let [s, f] = me.saves;
            let msg = if me.stable { "Inconsciente - estable".into() } else { format!("Inconsciente - exitos {s}/3, fallos {f}/3") };
            Some((msg, DOWN_TEXT))
        } else {
            None
        };
        if let Some((msg, color)) = center {
            font.text(out, &msg, ((w - font.width(&msg)) / 2.0).floor(), (h / 2.0 - 20.0).floor(), color);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_status(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, dt: f32) {
        let h = d.local();
        // NinePatch(STATUS, 80, 0, 48, 0) sobre 128 px: el centro mide 0, se estira la columna 80
        out.push(piece(atlas, STATUS, [0, 0, 80, 64], [0.0, 0.0], [80.0, 64.0], WHITE));
        if w > 128.0 {
            out.push(piece(atlas, STATUS, [80, 0, 1, 64], [80.0, 0.0], [w - 128.0, 64.0], WHITE));
        }
        out.push(piece(atlas, STATUS, [80, 0, 48, 64], [w - 48.0, 0.0], [48.0, 64.0], WHITE));
        // HeroSprite.avatar: recorte (1, 0) - (12, 15) de la hoja de la clase
        if let Some(sheet) = chars.look(d, ActorId::Hero(d.hero)).and_then(|l| l.sheet) {
            out.push(piece(atlas, sheet, [1, 0, 11, 15], [10.0, 9.0], [11.0, 15.0], WHITE));
        }
        self.hot([0.0, 1.0, 30.0, 30.0], UiAction::Open(Window::Hero), Some("Ficha del personaje"));
        let hpw = (64.0 * h.ch.hp.max(0) as f32 / h.ch.ht.max(1) as f32).floor();
        if hpw > 0.0 {
            out.push(piece(atlas, HP_BAR, [0, 0, hpw as u32, 4], [30.0, 3.0], [hpw, 4.0], WHITE));
        }
        let expw = (w * h.exp as f32 / h.max_exp() as f32).floor();
        if expw > 0.0 {
            out.push(piece(atlas, EXP_BAR, [0, 0, 16, 1], [0.0, 0.0], [expw, 1.0], WHITE));
        }
        let lvl = h.lvl.to_string();
        font.text(out, &lvl, 28.0 - (font.width(&lvl) / 2.0).floor(), 25.0, GOLDEN);
        let depth = h.depth.to_string();
        font.text(out, &depth, w - 24.0 - font.width(&depth) - 18.0, 6.0, DEPTH_COLOR);
        // BuffIndicator en (32, 11): hambre y los estados del héroe (los que tienen ícono)
        let mut icons: Vec<(u32, String)> = Vec::new();
        if h.is_starving() {
            icons.push((BUFF_STARVATION, "Famélico".into()));
        } else if h.hunger >= HUNGRY {
            icons.push((BUFF_HUNGER, "Hambriento".into()));
        }
        icons.extend(h.ch.buffs.iter().filter_map(|b| b.icon().map(|i| (i, b.name()))));
        let cols = atlas.rects[BUFFS][2] / BUFF_SIZE;
        for (k, (icon, tip)) in icons.iter().enumerate() {
            let x = 32.0 + k as f32 * (BUFF_SIZE as f32 + 2.0);
            let region = [(icon % cols) * BUFF_SIZE, (icon / cols) * BUFF_SIZE, BUFF_SIZE, BUFF_SIZE];
            out.push(piece(atlas, BUFFS, region, [x, 11.0], [7.0, 7.0], WHITE));
            self.hot([x, 11.0, 7.0, 7.0], UiAction::Nothing, Some(tip));
        }
        self.draw_initiative(atlas, font, chars, d, out, w, dt);
        // botón de menú: recorte (114, 3, 12, 11) de status_pane.png -> WndGame
        out.push(piece(atlas, STATUS, [114, 3, 12, 11], [w - 14.0, 3.0], [12.0, 11.0], WHITE));
        self.hot([w - 16.0, 1.0, 16.0, 16.0], UiAction::Open(Window::Menu), Some("Menu (Esc)"));
        // BusyIndicator: mientras el héroe no espera input.
        // ponytail: titila en vez de girar (el lote no rota sprites); rotar si se agrega un ángulo a Sprite
        let s = &d.floor().scheduler;
        let others_turn = s.order.is_some() && s.current() != Some(ActorId::Hero(d.hero));
        if h.is_alive() && (h.action.is_some() || h.animating || !d.pending.is_empty() || others_turn) {
            let a = 0.6 + 0.4 * (self.t * 8.0).sin();
            out.push(piece(atlas, ICONS, ICON_BUSY, [w - 12.0, 18.0], [8.0, 8.0], [1.0, 1.0, 1.0, a]));
        }
    }

    /// Carrusel de iniciativa (combate 5e), arriba a la derecha: el primero (borde amarillo) es al que le toca,
    /// después los siguientes. Al pasar el turno se desliza. Lo que no se ve sale como "?". Debajo: la ronda, y en tu
    /// turno los pies de movimiento que quedan y si te queda la acción.
    #[allow(clippy::too_many_arguments)]
    fn draw_initiative(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, dt: f32) {
        let s = &d.floor().scheduler;
        let Some(order) = &s.order else {
            self.car_cur = None;
            return;
        };
        let order: Vec<ActorId> = order.iter().copied().filter(|&a| s.contains(a)).collect();
        if order.is_empty() {
            return;
        }
        let cur = s.current();
        let i = cur.and_then(|c| order.iter().position(|&a| a == c)).unwrap_or(0);
        let ring: Vec<ActorId> = order[i..].iter().chain(&order[..i]).copied().take(CAROUSEL_MAX).collect();
        if cur != self.car_cur {
            if self.car_cur.is_some() {
                self.car_off = 1.0; // arranca corrido una casilla y vuelve a su lugar
            }
            self.car_cur = cur;
        }
        self.car_off = (self.car_off - CAROUSEL_SPEED * dt).max(0.0);
        let x0 = w - 2.0 - ring.len() as f32 * CAROUSEL_SLOT;
        for (k, &a) in ring.iter().enumerate() {
            let r = [x0 + ((k as f32 + self.car_off) * CAROUSEL_SLOT).floor(), CAROUSEL_Y, CAROUSEL_SLOT - 1.0, CAROUSEL_SLOT - 1.0];
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], if matches!(a, ActorId::Hero(_)) { HERO_BOX } else { MOB_BOX }));
            let Some(c) = d.ch(a) else { continue };
            let center = [r[0] + r[2] / 2.0, r[1] + r[3] / 2.0];
            if c.pos >= 0 && d.level().visible[c.pos as usize] {
                if let Some(p) = chars.portrait(d, atlas, a, center, CAROUSEL_SLOT - 3.0) {
                    out.push(p);
                }
                let ini = c.init.to_string();
                font.text(out, &ini, r[0] + r[2] - font.width(&ini), r[1] + r[3] - font.height(), WHITE);
                let tip = format!("{}: iniciativa {}", c.name, c.init);
                self.hot(r, UiAction::Nothing, Some(&tip));
            } else {
                font.text(out, "?", (center[0] - font.width("?") / 2.0).floor(), (center[1] - font.height() / 2.0).floor(), GREY);
            }
            if k == 0 {
                out.push(atlas.rect([r[0], r[1]], [r[2], 1.0], YELLOW));
                out.push(atlas.rect([r[0], r[1] + r[3] - 1.0], [r[2], 1.0], YELLOW));
                out.push(atlas.rect([r[0], r[1]], [1.0, r[3]], YELLOW));
                out.push(atlas.rect([r[0] + r[2] - 1.0, r[1]], [1.0, r[3]], YELLOW));
            }
        }
        let me = d.local();
        let label = if cur == Some(ActorId::Hero(d.hero)) {
            format!("Ronda {} - {} ft{}", s.round, me.ch.moves.max(0) * 5, if me.ch.has_action { " - accion" } else { "" })
        } else {
            format!("Ronda {}", s.round)
        };
        font.text(out, &label, w - 2.0 - font.width(&label), CAROUSEL_Y + CAROUSEL_SLOT + 1.0, GOLDEN);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_toolbar(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let y = h - TOOLBAR_H;
        let me = d.local();
        let bag = if self.window.is_some() { UiAction::Close } else { UiAction::Open(Window::Bag(BagMode::Browse)) };
        out.push(piece(atlas, TOOLBAR, TOOL_BAG, [w - 67.0, y], [23.0, 25.0], WHITE));
        self.hot([w - 67.0, y, 23.0, 25.0], bag, Some("Mochila (I)"));
        // 2 accesos rápidos (Toolbar.btnQuick1/2) = teclas 1 y 2. Click = usar (o elegir si está vacío), derecho = elegir
        for (q, x) in [(0usize, w - 44.0), (1, w - 22.0)] {
            out.push(piece(atlas, TOOLBAR, TOOL_QUICK, [x, y], [22.0, 25.0], WHITE));
            let item = me.quick_item(q).and_then(|s| me.item(s));
            let what = item.map_or("vacío, click derecho para elegir".to_string(), |i| d.item_name(i));
            let tip = format!("Acceso rápido {}: {what} ({})", q + 1, q + 1);
            self.hot.push(Hot {
                rect: [x, y, 22.0, 25.0],
                act: UiAction::UseQuick(q),
                alt: Some(UiAction::Open(Window::Bag(BagMode::Quick(q)))),
                tip: Some(tip),
            });
            if let Some(item) = item {
                item_icon(out, atlas, font, &d.item_image(item), [x + 3.0, y + 5.0], 1.0);
                if item.quantity > 1 {
                    font.text(out, &item.quantity.to_string(), x + 2.0, y + 3.0, WHITE);
                }
            }
        }
        let tools = [
            (TOOL_WAIT, 0.0, UiAction::Act(Action::Rest), "Esperar un turno (Espacio)"),
            (TOOL_SEARCH, 20.0, UiAction::Act(Action::Search), "Buscar (Q)"),
            (TOOL_INFO, 40.0, UiAction::Examine, "Examinar una casilla"),
        ];
        for (region, x, act, tip) in tools {
            let size = [region[2] as f32, region[3] as f32];
            out.push(piece(atlas, TOOLBAR, region, [x, y], size, WHITE));
            self.hot([x, y, size[0], size[1]], act, Some(tip));
        }
        // rasgo de clase (F): en el teléfono no hay teclado. El ícono de la clase de Icons.java
        if me.has_trait("pdri:second_wind") || me.spellcasting().is_some() {
            let x = 61.0;
            out.push(piece(atlas, TOOLBAR, TOOL_QUICK, [x, y], [22.0, 25.0], WHITE));
            let k = ["pdri:warrior", "pdri:mage", "pdri:rogue", "pdri:huntress"].iter().position(|c| *c == me.class_key).unwrap_or(0);
            let used = me.has_trait("pdri:second_wind") && !me.second_wind;
            let alpha = if used { 0.4 } else { 1.0 };
            out.push(piece(atlas, ICONS, [k as u32 * 16, ICON_CLASS_Y, 16, 16], [x + 3.0, y + 5.0], [16.0, 16.0], [1.0, 1.0, 1.0, alpha]));
            let tip = if me.has_trait("pdri:second_wind") { "Segundo Aliento (F)" } else { "Conjuros (F)" };
            self.hot([x, y, 22.0, 25.0], UiAction::Trait, Some(tip));
        }
        // chat de la partida LAN (Enter): en el teléfono no hay teclado
        if self.lan && self.window.is_none() && self.chat.is_none() {
            self.draw_button(atlas, font, out, [85.0, y + 3.0, 30.0, 19.0], "Chat", Some(UiAction::Chat(Cr::New)));
        }
    }

    /// Con 2+ héroes, fuera de combate: la lista de los del piso con ✓ si ya eligieron qué hacer (en combate manda el
    /// carrusel de iniciativa). Y si la partida espera a otro, a quién.
    fn draw_waiting(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32) {
        if d.heroes.len() < 2 || self.waiting.len() != d.heroes.len() {
            return;
        }
        let here: Vec<usize> = d.here().collect();
        let fight = d.floor().scheduler.order.is_some();
        if !fight {
            for (row, &i) in here.iter().enumerate() {
                let y = 45.0 + row as f32 * 13.0;
                let region = if self.waiting[i] { ICON_UNCHECKED } else { ICON_CHECKED };
                out.push(piece(atlas, ICONS, region, [2.0, y], [12.0, 12.0], WHITE));
                let h = &d.heroes[i];
                let color = if !h.is_alive() || h.is_downed() { GREY } else if i == d.hero { YELLOW } else { WHITE };
                font.text(out, &h.ch.name, 16.0, y + 2.0, color);
            }
        }
        let others: Vec<&str> = match d.floor().scheduler.current() {
            Some(ActorId::Hero(j)) if fight && j != d.hero => vec![d.heroes[j].ch.name.as_str()],
            _ if fight => Vec::new(),
            // fuera de combate: los que faltan, una vez que elegiste
            _ if !self.waiting[d.hero] => here.iter().filter(|&&i| i != d.hero && self.waiting[i]).map(|&i| d.heroes[i].ch.name.as_str()).collect(),
            _ => Vec::new(),
        };
        if !others.is_empty() && d.local().is_alive() {
            let msg = format!("Esperando a {}", others.join(", "));
            let tw = font.width(&msg);
            let x = ((w - tw - 8.0) / 2.0).floor();
            out.push(atlas.rect([x, 48.0], [tw + 8.0, font.height() + 4.0], PROMPT_BG));
            font.text(out, &msg, x + 4.0, 50.0, WHITE);
        }
    }

    /// Una ranura de la mochila: fondo, ícono (o el de ranura vacía, tenue), cantidad y nivel conocido.
    #[allow(clippy::too_many_arguments)]
    fn draw_slot(atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, r: [f32; 4], item: Option<&Item>, placeholder: Option<i32>, enabled: bool) {
        out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], SLOT_BG));
        let at = [(r[0] + r[2] / 2.0 - 8.0).floor(), (r[1] + r[3] / 2.0 - 8.0).floor()];
        let Some(item) = item else {
            if let Some(p) = placeholder {
                item_icon(out, atlas, font, &Image::Sheet(p), at, 80.0 / 255.0);
            }
            return;
        };
        item_icon(out, atlas, font, &d.item_image(item), at, if enabled { 1.0 } else { 70.0 / 255.0 });
        if item.quantity > 1 {
            font.text(out, &item.quantity.to_string(), r[0] + 1.0, r[1] + 1.0, WHITE);
        }
        if item.is_equipment() && item.level_known && item.level != 0 {
            let lv = format!("{:+}", item.level);
            let color = if item.level > 0 { LEVEL_UP } else { LEVEL_DOWN };
            font.text(out, &lv, r[0] + r[2] - font.width(&lv), r[1] + r[3] - font.height(), color);
        }
    }

    /// WndBag: equipo, oro y las ranuras de la mochila. Según el modo, click = ver el ítem, elegir el objetivo de un
    /// pergamino o elegir el acceso rápido.
    #[allow(clippy::too_many_arguments)]
    fn draw_bag(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32, mode: &BagMode) {
        let me = d.local();
        // las que entren (5 en vertical): la fila de equipo necesita 4 + el oro
        let cols = (((w - 16.0 + SLOT_MARGIN) / (SLOT_SIZE + SLOT_MARGIN)).floor() as usize).clamp(5, BAG_COLS);
        let rows = 1 + BACKPACK_SIZE.div_ceil(cols);
        let cw = SLOT_SIZE * cols as f32 + SLOT_MARGIN * (cols - 1) as f32;
        let ch = BAG_TITLE_H + SLOT_SIZE * rows as f32 + SLOT_MARGIN * (rows - 1) as f32 + BAG_GAP;
        let c = [((w - cw) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), cw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None); // click afuera = cerrar
        Self::draw_window_tinted(atlas, out, c, BAG_TINT);
        self.hot(c, UiAction::Nothing, None);
        let title = match mode {
            BagMode::Browse => "Mochila".to_string(),
            BagMode::Target { .. } => "Elige un objeto".to_string(),
            BagMode::Quick(q) => format!("Acceso rapido {}: elige un objeto", q + 1),
        };
        font.text(out, &title, c[0], c[1] + 2.0, YELLOW);
        let cell = |col: usize, row: usize| {
            let gap = if row > 0 { BAG_GAP } else { 0.0 };
            [
                c[0] + col as f32 * (SLOT_SIZE + SLOT_MARGIN),
                c[1] + BAG_TITLE_H + row as f32 * (SLOT_SIZE + SLOT_MARGIN) + gap,
                SLOT_SIZE,
                SLOT_SIZE,
            ]
        };
        let eligible = |slot: Slot, item: &Item| match mode {
            BagMode::Target { scroll, need } => slot != *scroll && me.item(*scroll).is_some_and(|s| s.id != item.id) && d.target_ok(need, item),
            _ => true,
        };
        let mut slots: Vec<([f32; 4], Option<Slot>, Option<i32>)> = vec![
            (cell(0, 0), me.weapon.as_ref().map(|_| Slot::Weapon), Some(SLOT_WEAPON)),
            (cell(1, 0), me.armor.as_ref().map(|_| Slot::Armor), Some(SLOT_ARMOR)),
            (cell(2, 0), None, Some(SLOT_RING)),
            (cell(3, 0), None, Some(SLOT_RING)),
        ];
        for k in 0..BACKPACK_SIZE {
            slots.push((cell(k % cols, 1 + k / cols), (k < me.backpack.len()).then_some(Slot::Pack(k)), None));
        }
        for (r, slot, placeholder) in slots {
            let item = slot.and_then(|s| me.item(s));
            let ok = slot.zip(item).is_some_and(|(s, i)| eligible(s, i));
            Self::draw_slot(atlas, font, d, out, r, item, placeholder, ok);
            let (Some(slot), Some(item)) = (slot, item) else { continue };
            let act = match mode {
                BagMode::Browse => UiAction::Open(Window::Item(slot)),
                BagMode::Quick(q) => UiAction::SetQuick(*q, item.id.clone()),
                BagMode::Target { scroll, .. } if ok => UiAction::Act(Action::Use(*scroll, Verb::Read, Some(slot))),
                BagMode::Target { .. } => UiAction::Nothing,
            };
            let tip = d.item_name(item);
            self.hot(r, act, Some(&tip));
        }
        // oro: ranura aparte, al final de la fila de equipo
        let gold = (me.gold > 0).then(|| {
            let mut g = d.gold_item();
            g.quantity = me.gold;
            g
        });
        Self::draw_slot(atlas, font, d, out, cell(cols - 1, 0), gold.as_ref(), Some(14), true);
    }

    /// WndItem: ícono, nombre, descripción con los números de 5e y un botón por acción.
    #[allow(clippy::too_many_arguments)]
    fn draw_item(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32, slot: Slot) {
        let me = d.local();
        let iw = ITEM_WND_W.min(w - 16.0);
        let Some(item) = me.item(slot) else {
            self.window = None; // se gastó o se soltó
            return;
        };
        let name = wrap(font, &d.item_title(item), iw - 20.0);
        let info = wrap(font, &d.item_info(item, me), iw);
        let btn_h = 16.0;
        let head = (name.len() as f32 * 9.0).max(16.0);
        let ch = head + 4.0 + info.len() as f32 * 9.0 + 4.0 + btn_h;
        let c = [((w - iw) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), iw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None);
        Self::draw_window_tinted(atlas, out, c, BAG_TINT);
        self.hot(c, UiAction::Nothing, None);
        item_icon(out, atlas, font, &d.item_image(item), [c[0], c[1]], 1.0);
        for (k, line) in name.iter().enumerate() {
            font.text(out, line, c[0] + 20.0, c[1] + 4.0 + k as f32 * 9.0, YELLOW);
        }
        for (k, line) in info.iter().enumerate() {
            font.text(out, line, c[0], c[1] + head + 4.0 + k as f32 * 9.0, WHITE);
        }
        let verbs = me.verbs(slot);
        let bw = ((iw - (verbs.len() as f32 - 1.0) * 2.0) / verbs.len() as f32).floor();
        for (k, v) in verbs.into_iter().enumerate() {
            let rect = [c[0] + k as f32 * (bw + 2.0), c[1] + ch - btn_h, bw, btn_h];
            self.draw_button(atlas, font, out, rect, v.label(), Some(UiAction::Verb(slot, v)));
        }
    }

    fn draw_window(atlas: &Atlas, out: &mut Vec<Sprite>, content: [f32; 4]) {
        Self::draw_window_tinted(atlas, out, content, WHITE);
    }

    fn draw_window_tinted(atlas: &Atlas, out: &mut Vec<Sprite>, content: [f32; 4], tint: [f32; 4]) {
        let (region, m) = CHROME_WINDOW;
        nine(out, atlas, region, m, [content[0] - m, content[1] - m, content[2] + 2.0 * m, content[3] + 2.0 * m], tint);
    }

    fn draw_button(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, rect: [f32; 4], label: &str, act: Option<UiAction>) {
        let (region, m) = CHROME_BUTTON;
        let alpha = if act.is_some() { 1.0 } else { 110.0 / 255.0 };
        nine(out, atlas, region, m, rect, [1.0, 1.0, 1.0, alpha]);
        let x = (rect[0] + rect[2] / 2.0 - font.width(label) / 2.0).floor();
        let y = (rect[1] + rect[3] / 2.0 - font.height() / 2.0).floor();
        font.text(out, label, x, y, if act.is_some() { WHITE } else { BUTTON_OFF });
        if let Some(act) = act {
            self.hot(rect, act, None);
        }
    }

    /// Diálogo con un NPC: retrato, nombre, lo que dice y las opciones como botones.
    #[allow(clippy::too_many_arguments)]
    fn draw_dialog(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32, dv: &DialogView) {
        let ww = (w - 16.0).min(220.0);
        let lines = wrap(font, &dv.text, ww);
        let opts: Vec<(usize, Vec<String>)> = dv.options.iter().map(|(k, t)| (*k, wrap(font, t, ww - 8.0))).collect();
        let opt_h: f32 = opts.iter().map(|o| o.1.len().max(1) as f32 * LINE_H + 6.0).sum();
        let head = 19.0;
        let ch = head + lines.len() as f32 * LINE_H + 4.0 + opt_h;
        let c = [((w - ww) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor().max(2.0), ww, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Nothing, None);
        Self::draw_window(atlas, out, c);
        if let Some(p) = chars.portrait(d, atlas, dv.npc, [c[0] + 8.0, c[1] + 8.0], 16.0) {
            out.push(p);
        }
        font.text(out, &capitalize(&dv.name), c[0] + 19.0, c[1] + 4.0, YELLOW);
        let mut y = c[1] + head;
        for l in &lines {
            font.text(out, l, c[0], y, WHITE);
            y += LINE_H;
        }
        y += 4.0;
        let (region, m) = CHROME_BUTTON;
        for (k, ls) in opts {
            let r = [c[0], y, ww, ls.len().max(1) as f32 * LINE_H + 4.0];
            nine(out, atlas, region, m, r, WHITE);
            for (j, l) in ls.iter().enumerate() {
                font.text(out, l, c[0] + 4.0, y + 2.0 + j as f32 * LINE_H, WHITE);
            }
            self.hot(r, UiAction::Act(Action::Choose(k)), None);
            y += r[3] + 2.0;
        }
    }

    /// WndInfoCell / WndInfoMob: ícono + título (+ barra de vida si es un mob) y descripción.
    #[allow(clippy::too_many_arguments)]
    fn draw_info(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32, icon: Option<Icon>, title: &str, text: &str, hp: Option<f32>) {
        let mut lines = Vec::new();
        for para in text.split('\n') {
            let l = wrap(font, para, INFO_W);
            if l.is_empty() { lines.push(String::new()) } else { lines.extend(l) }
        }
        let icon_h: f32 = if icon.is_some() { 16.0 } else { 0.0 };
        let head = icon_h.max(LINE_H + if hp.is_some() { 4.0 } else { 0.0 });
        let ch = head + 4.0 + lines.len() as f32 * LINE_H;
        let c = [((w - INFO_W) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), INFO_W, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None); // click en cualquier lado cierra
        Self::draw_window(atlas, out, c);
        let mut x = c[0];
        match icon {
            Some(Icon::Tile(t)) => {
                let (sheet, region) = if t == terrain::WATER {
                    (WATER, [0, 0, 16, 16])
                } else {
                    (TILES, [(t as u32 % 16) * 16, (t as u32 / 16) * 16, 16, 16])
                };
                out.push(piece(atlas, sheet, region, [x, c[1]], [16.0, 16.0], WHITE));
                x += 19.0;
            }
            Some(Icon::Char(a)) => {
                if let Some(p) = chars.portrait(d, atlas, a, [x + 8.0, c[1] + 8.0], 16.0) {
                    out.push(p);
                }
                x += 19.0;
            }
            None => {}
        }
        font.text(out, title, x, c[1] + 1.0, YELLOW);
        if let Some(hp) = hp {
            let bar = [x, c[1] + LINE_H + 1.0, c[0] + c[2] - x, 2.0];
            out.push(atlas.rect([bar[0], bar[1]], [bar[2], bar[3]], HP_RED));
            out.push(atlas.rect([bar[0], bar[1]], [(bar[2] * hp.clamp(0.0, 1.0)).floor(), bar[3]], HP_GREEN));
        }
        for (i, line) in lines.iter().enumerate() {
            font.text(out, line, c[0], c[1] + head + 4.0 + i as f32 * LINE_H, WHITE);
        }
    }

    /// Ficha 5e del personaje (WndHero del original, con los números de D&D).
    fn draw_hero(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let me = d.local();
        let abil = |range: std::ops::Range<usize>| {
            range
                .map(|i| format!("{} {}({:+})", ABILITY_NAMES[i], me.abilities[i], me.ability_mod(ABILITIES[i])))
                .collect::<Vec<_>>()
                .join("   ")
        };
        let dmg = me.weapon.as_ref().map_or("1".to_string(), |w| weapon_dice(w).to_string());
        let saves: Vec<String> = (0..6).map(|i| format!("{}{:+}", ABILITY_NAMES[i], me.save_bonus(ABILITIES[i]))).collect();
        let mut lines = vec![
            (format!("{} ({}) nivel {}", me.ch.name, text(&me.cls["dnd_class"]), me.lvl), YELLOW),
            (abil(0..3), WHITE),
            (abil(3..6), WHITE),
            (format!("PV {}/{}   CA {}   Competencia +{}", me.ch.hp, me.ch.ht, me.armor_class(), me.proficiency()), WHITE),
            (
                format!("Ataque {:+}   Daño {dmg}{:+}   Dado de golpe d{}", me.attack_bonus(), me.attack_mod(), me.cls["hit_die"]),
                WHITE,
            ),
            (format!("Salvaciones {}", saves.join(" ")), WHITE),
            (format!("XP {}/{}   Oro {}", me.exp, me.max_exp(), me.gold), WHITE),
        ];
        let trait_btn = if me.has_trait("pdri:second_wind") {
            let state = if me.second_wind { "disponible" } else { "usado" };
            lines.push((format!("Segundo Aliento (F): {state}"), WHITE));
            Some(("Segundo Aliento", me.second_wind.then_some(UiAction::Act(Action::SecondWind))))
        } else if me.spellcasting().is_some() {
            lines.push((format!("Conjuros (F): ataque {:+}, CD {}", me.spell_bonus(), me.spell_dc()), WHITE));
            Some(("Conjuros", Some(UiAction::Open(Window::Spells))))
        } else {
            None
        };
        if me.has_trait("pdri:sneak_attack") {
            lines.push((format!("Ataque Furtivo: +{}d6 con ventaja y arma sutil", me.sneak_dice()), WHITE));
        }
        if !me.ch.buffs.is_empty() {
            let names: Vec<String> = me.ch.buffs.iter().map(|b| b.name()).collect();
            lines.push((format!("Estados: {}", names.join(", ")), STATES));
        }
        // en vertical las líneas largas se parten
        let lines: Vec<(String, [f32; 4])> =
            lines.into_iter().flat_map(|(l, c)| wrap(font, &l, w - 30.0).into_iter().map(move |x| (x, c))).collect();
        let cw = lines.iter().map(|l| font.width(&l.0)).fold(0.0, f32::max);
        let btn = if trait_btn.is_some() { MENU_BTN_H + MENU_GAP } else { 0.0 };
        let ch = lines.len() as f32 * LINE_H + btn;
        let c = [((w - cw) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), cw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None);
        Self::draw_window(atlas, out, c);
        self.hot(c, UiAction::Nothing, None);
        for (i, (line, color)) in lines.iter().enumerate() {
            font.text(out, line, c[0], c[1] + i as f32 * LINE_H, *color);
        }
        // en el teléfono no hay F: el rasgo de clase se usa desde aquí
        if let Some((label, act)) = trait_btn {
            let rect = [c[0], c[1] + ch - MENU_BTN_H, cw, MENU_BTN_H];
            self.draw_button(atlas, font, out, rect, label, act);
        }
    }

    /// Libro de conjuros (F): bono, CD, espacios y usos gratis. Click en un conjuro = elegir el objetivo (o lanzarlo,
    /// si es sobre uno mismo). En gris los que no se pueden lanzar ahora.
    #[allow(clippy::too_many_arguments)]
    fn draw_spells(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let me = d.local();
        let Some(sc) = me.spellcasting() else {
            self.window = None;
            return;
        };
        let sw = SPELLS_W.min(w - 16.0);
        let ability = sc["ability"].as_str().unwrap_or("INT");
        let short = ABILITIES.iter().position(|a| *a == ability).map_or(ability, |k| ABILITY_NAMES[k]);
        let max = me.max_slots(me.lvl);
        let slots: Vec<String> = me.slots.iter().zip(&max).enumerate().map(|(k, (n, m))| format!("nv{} {n}/{m}", k + 1)).collect();
        let mut head = vec![
            ("Conjuros".to_string(), YELLOW),
            (format!("Ataque {:+}   CD {}   ({short})", me.spell_bonus(), me.spell_dc()), WHITE),
            (format!("Espacios: {}", if slots.is_empty() { "-".to_string() } else { slots.join("  ") }), WHITE),
        ];
        for (sid, n) in &me.free_casts {
            let name = d.spell(sid).map_or(sid.clone(), |s| text(&s["name"]));
            head.push((format!("{name}: {n} gratis"), WHITE));
        }
        let rows: Vec<(usize, String, Vec<String>, bool, bool)> = me
            .spell_ids()
            .iter()
            .enumerate()
            .filter_map(|(k, sid)| {
                let s = d.spell(sid)?;
                let lvl = s["level"].as_i64().unwrap_or(0);
                let tag = if lvl == 0 { "truco".to_string() } else { format!("nv {lvl}") };
                let desc = s.get("desc").map_or(String::new(), text);
                let lines = wrap(font, &desc, sw).into_iter().take(2).collect();
                Some((k, format!("{} ({tag})", text(&s["name"])), lines, d.can_cast(me, sid), s["target"] == "self"))
            })
            .collect();
        let ch = head.len() as f32 * LINE_H + 4.0 + rows.iter().map(|r| MENU_BTN_H + 2.0 + r.2.len() as f32 * LINE_H + MENU_GAP).sum::<f32>();
        let c = [((w - sw) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor().max(2.0), sw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None);
        Self::draw_window(atlas, out, c);
        self.hot(c, UiAction::Nothing, None);
        let mut y = c[1];
        for (line, color) in &head {
            font.text(out, line, c[0], y, *color);
            y += LINE_H;
        }
        y += 4.0;
        for (k, label, desc, ok, own) in rows {
            let act = if own { UiAction::Act(Action::Cast(k, me.ch.pos)) } else { UiAction::Cast(k) };
            self.draw_button(atlas, font, out, [c[0], y, sw, MENU_BTN_H], &label, ok.then_some(act));
            y += MENU_BTN_H + 2.0;
            for line in desc {
                font.text(out, &line, c[0], y, GREY);
                y += LINE_H;
            }
            y += MENU_GAP;
        }
    }

    /// WndGame: menú de la partida (botón de menú o Esc). Nueva partida sólo si el héroe murió, como el original.
    fn draw_menu(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        // WndGame: nueva partida sólo si el héroe murió, como el original
        let mut rows: Vec<Vec<(&str, UiAction)>> = Vec::new();
        if !d.local().is_alive() {
            rows.push(vec![("Nueva partida", UiAction::NewGame)]);
        }
        rows.push(vec![("Configuracion", UiAction::Open(Window::Settings))]);
        rows.push(vec![("Reglas de la partida", UiAction::Open(Window::Rules))]);
        rows.push(vec![("Menu principal", UiAction::MainMenu), ("Salir", UiAction::Quit)]);
        rows.push(vec![("Volver al juego", UiAction::Close)]);
        let mh = rows.len() as f32 * (MENU_BTN_H + MENU_GAP) - MENU_GAP;
        let c = [((w - MENU_W) / 2.0).floor(), ((h - TOOLBAR_H - mh) / 2.0).floor(), MENU_W, mh];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None); // click afuera cierra
        self.hot(c, UiAction::Nothing, None);
        Self::draw_window(atlas, out, c);
        for (r, row) in rows.into_iter().enumerate() {
            let n = row.len() as f32;
            let bw = ((MENU_W - MENU_GAP * (n - 1.0)) / n).floor();
            for (k, (label, act)) in row.into_iter().enumerate() {
                let rect = [c[0] + k as f32 * (bw + MENU_GAP), c[1] + r as f32 * (MENU_BTN_H + MENU_GAP), bw, MENU_BTN_H];
                self.draw_button(atlas, font, out, rect, label, Some(act));
            }
        }
    }

    /// Reglas de la partida: se aplican al momento. La primera opción de cada una es la de 5e.
    /// ponytail: se guardarán con la partida cuando haya guardado; en red las decidirá el host
    fn draw_rules(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let sw = SETTINGS_W.min(w - 16.0);
        let r = d.rules;
        let rows = [
            ("dot_death_saves", "Caido: el daño por turno (veneno, sangrado, fuego) suma fallos de salvacion", r.dot_death_saves, ["Si (5e)", "No"]),
            ("save_half", "Salvacion superada contra gas toxico o descarga electrica", r.save_half, ["Mitad (5e)", "Sin efecto"]),
        ];
        let wrapped: Vec<Vec<String>> = rows.iter().map(|row| wrap(font, row.1, sw)).collect();
        let ch = LINE_H + MENU_GAP * 2.0 + wrapped.iter().map(|l| l.len() as f32 * LINE_H + MENU_BTN_H + MENU_GAP * 2.0).sum::<f32>() + MENU_BTN_H;
        let c = [((w - sw) / 2.0).floor(), ((h - ch) / 2.0).floor(), sw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Open(Window::Menu), None);
        self.hot(c, UiAction::Nothing, None);
        Self::draw_window(atlas, out, c);
        let title = "Reglas de la partida";
        font.text(out, title, c[0] + ((sw - font.width(title)) / 2.0).floor(), c[1], YELLOW);
        let mut y = c[1] + LINE_H + MENU_GAP * 2.0;
        for ((key, _, value, labels), lines) in rows.into_iter().zip(wrapped) {
            for line in lines {
                font.text(out, &line, c[0], y, WHITE);
                y += LINE_H;
            }
            let bw = ((sw - MENU_GAP) / 2.0).floor();
            for (k, (label, v)) in labels.into_iter().zip([true, false]).enumerate() {
                let rect = [c[0] + k as f32 * (bw + MENU_GAP), y, bw, MENU_BTN_H];
                self.draw_button(atlas, font, out, rect, label, Some(UiAction::SetRule(key, v)));
                if value == v {
                    outline(out, atlas, rect, YELLOW);
                }
            }
            y += MENU_BTN_H + MENU_GAP * 2.0;
        }
        self.draw_button(atlas, font, out, [c[0], y, sw, MENU_BTN_H], "Volver", Some(UiAction::Open(Window::Menu)));
    }

    /// WndSettings: FPS, pantalla completa y controles (toca uno y presiona la tecla nueva). Cada cambio se aplica y
    /// se guarda al momento en Local.config. Volver = al menú.
    #[allow(clippy::too_many_arguments)]
    fn draw_settings(&mut self, atlas: &Atlas, font: &Font, cfg: &Config, out: &mut Vec<Sprite>, w: f32, h: f32) {
        // WndSettings: idioma, pantalla completa, FPS y escala de la interfaz; los controles en su ventana
        let sw = SETTINGS_W.min(w - 16.0);
        let row_h = LINE_H + MENU_BTN_H + MENU_GAP * 2.0;
        let desktop = !cfg!(target_os = "android");
        let ch = if desktop { 5.0 } else { 4.0 } * row_h + MENU_BTN_H;
        let c = [((w - sw) / 2.0).floor(), ((h - ch) / 2.0).floor(), sw, ch];
        let back = if self.in_title { UiAction::Close } else { UiAction::Open(Window::Menu) };
        self.hot([0.0, 0.0, w, h], back.clone(), None); // click afuera = volver
        self.hot(c, UiAction::Nothing, None);
        Self::draw_window(atlas, out, c);
        // idioma del contenido (nombres y descripciones de los JSON); los textos del código siguen en español
        let lang: Vec<(String, bool, Option<UiAction>)> = game::registry::LANGS
            .iter()
            .map(|&(code, name)| (name.to_string(), cfg.language == code, Some(UiAction::SetLanguage(code))))
            .collect();
        let full = vec![
            ("Si".to_string(), cfg.fullscreen, Some(UiAction::SetFullscreen(true))),
            ("No".to_string(), !cfg.fullscreen, Some(UiAction::SetFullscreen(false))),
        ];
        let fps: Vec<(String, bool, Option<UiAction>)> = FPS_OPTIONS
            .iter()
            .map(|&f| (if f == 0 { "Sin limite".into() } else { f.to_string() }, cfg.max_fps == f, Some(UiAction::SetFps(f))))
            .collect();
        // del zoom que entra hacia abajo (75% y 50% en HD): siempre entero, nítido
        let z = cfg.ui_base.max(1);
        let scales: Vec<(String, bool, Option<UiAction>)> = (z.saturating_sub(2).max(1)..=z)
            .map(|s| {
                let pref = s as f32 / z as f32;
                (format!("{}%", (pref * 100.0).round()), (cfg.ui_scale - pref).abs() < 0.01, Some(UiAction::SetUiScale(pref)))
            })
            .collect();
        let res: Vec<(String, bool, Option<UiAction>)> = RESOLUTIONS
            .iter()
            .map(|&(k, rw, rh)| (format!("{rw}x{rh}"), cfg.resolution == k, Some(UiAction::SetResolution(k))))
            .collect();
        let mut rows = vec![("Idioma", lang), ("Pantalla completa (F11)", full)];
        if desktop {
            rows.push(("Resolucion (ventana)", res));
        }
        rows.extend([("FPS", fps), ("Escala de la interfaz (en partida)", scales)]);
        let mut y = c[1];
        for (label, options) in rows {
            font.text(out, label, c[0], y, YELLOW);
            if label == "FPS" {
                // ver el contador en la partida: un botón chico en la línea del título (no suma una fila)
                let t = if cfg.show_fps { "Mostrar: Si" } else { "Mostrar: No" };
                let tw = font.width(t) + 8.0;
                self.draw_button(atlas, font, out, [c[0] + sw - tw, y - 1.0, tw, LINE_H], t, Some(UiAction::SetShowFps(!cfg.show_fps)));
            }
            y += LINE_H;
            let n = options.len() as f32;
            let bw = ((sw - MENU_GAP * (n - 1.0)) / n).floor();
            for (k, (text, selected, act)) in options.into_iter().enumerate() {
                let rect = [c[0] + k as f32 * (bw + MENU_GAP), y, bw, MENU_BTN_H];
                self.draw_button(atlas, font, out, rect, &text, act);
                if selected {
                    outline(out, atlas, rect, YELLOW);
                }
            }
            y += MENU_BTN_H + MENU_GAP * 2.0;
        }
        let bw = ((sw - MENU_GAP) / 2.0).floor();
        self.draw_button(atlas, font, out, [c[0], y, bw, MENU_BTN_H], "Controles", Some(UiAction::Open(Window::Controls)));
        self.draw_button(atlas, font, out, [c[0] + bw + MENU_GAP, y, bw, MENU_BTN_H], "Volver", Some(back));
    }

    /// Controles: toca una acción y presiona la tecla nueva (Esc cancela). Se guarda al momento.
    fn draw_controls(&mut self, atlas: &Atlas, font: &Font, cfg: &Config, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let sw = SETTINGS_W.min(w - 16.0);
        let rows = Bind::ALL.len().div_ceil(3);
        let ch = LINE_H + rows as f32 * (CTRL_H + MENU_GAP) + MENU_GAP + MENU_BTN_H;
        let c = [((w - sw) / 2.0).floor(), ((h - ch) / 2.0).floor(), sw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Open(Window::Settings), None);
        self.hot(c, UiAction::Nothing, None);
        Self::draw_window(atlas, out, c);
        font.text(out, "Controles (toca uno y presiona la tecla)", c[0], c[1], YELLOW);
        let y = c[1] + LINE_H;
        let bw = ((sw - MENU_GAP * 2.0) / 3.0).floor();
        for (k, b) in Bind::ALL.into_iter().enumerate() {
            let rect = [c[0] + (k % 3) as f32 * (bw + MENU_GAP), y + (k / 3) as f32 * (CTRL_H + MENU_GAP), bw, CTRL_H];
            let waiting = self.rebinding == Some(b);
            let text = if waiting { format!("{} ...", b.label()) } else { format!("{} {}", b.label(), key_label(cfg.key(b))) };
            self.draw_button(atlas, font, out, rect, &text, Some(UiAction::Rebind(b)));
            if waiting {
                outline(out, atlas, rect, YELLOW);
            }
        }
        let y = y + rows as f32 * (CTRL_H + MENU_GAP) + MENU_GAP;
        self.draw_button(atlas, font, out, [c[0], y, sw, MENU_BTN_H], "Volver", Some(UiAction::Open(Window::Settings)));
    }

    /// Lista de personajes (rosterScreen.py): elegir uno para jugar (su partida guardada si sigue vivo), crear uno
    /// nuevo o borrarlo (pide confirmación). Sin personajes, JUGAR y BORRAR quedan grises. La rueda desplaza la lista.
    #[allow(clippy::too_many_arguments)]
    fn draw_roster(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, rows: &[RosterRow]) {
        let narrow = w < 260.0;
        let row_h: f32 = if narrow { 40.0 } else { 30.0 };
        const LIST_Y: f32 = 15.0;
        let visible = (((h - LIST_Y - 42.0) / (row_h + 2.0)).floor() as usize).max(1);
        self.roster_scroll = self.roster_scroll.min(rows.len().saturating_sub(visible));
        out.push(atlas.rect([0.0, 0.0], [w, h], [0.0, 0.0, 0.0, 1.0]));
        self.hot([0.0, 0.0, w, h], UiAction::Nothing, None);
        font.text(out, "Personajes", 8.0, 3.0, YELLOW);
        if rows.is_empty() {
            font.text(out, "Todavia no hay personajes. Crea uno con NUEVO.", 8.0, LIST_Y + 10.0, GREY);
        }
        for (row, i) in (self.roster_scroll..rows.len().min(self.roster_scroll + visible)).enumerate() {
            let c = &rows[i];
            let r = [8.0, LIST_Y + row as f32 * (row_h + 2.0), w - 16.0, row_h];
            let bg = if self.roster_sel == Some(i) { rgba(0x99, 0x88, 0x33, 0xAA) } else { SLOT_BG };
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], bg));
            if let Some(sheet) = c.sheet {
                out.push(piece(atlas, sheet, [1, 0, 11, 15], [r[0] + 4.0, r[1] + ((row_h - 15.0) / 2.0).floor()], [11.0, 15.0], WHITE));
            }
            if narrow {
                font.text(out, &c.name, r[0] + 20.0, r[1] + 2.0, YELLOW);
                font.text(out, &c.status, r[0] + 20.0, r[1] + 11.0, GREY);
                for (k, l) in wrap(font, &c.stats, r[2] - 24.0).iter().take(2).enumerate() {
                    font.text(out, l, r[0] + 20.0, r[1] + 20.0 + k as f32 * 9.0, WHITE);
                }
            } else {
                font.text(out, &c.name, r[0] + 20.0, r[1] + 4.0, YELLOW);
                font.text(out, &c.status, r[0] + 100.0, r[1] + 4.0, GREY);
                font.text(out, &c.stats, r[0] + 20.0, r[1] + 17.0, WHITE);
            }
            self.hot(r, UiAction::SelectChar(i), None);
        }
        let has = self.roster_sel.is_some_and(|s| s < rows.len());
        let buttons = [
            ("JUGAR", has.then_some(UiAction::PlayChar)),
            ("NUEVO", Some(UiAction::Create(Cr::New))),
            (if self.confirm { "CONFIRMAR" } else { "BORRAR" }, has.then_some(UiAction::DeleteChar)),
        ];
        // red LAN: con el personaje elegido
        let lan = [
            ("CREAR LAN", has.then_some(UiAction::HostLan)),
            ("UNIRSE", has.then_some(UiAction::OpenJoin)),
            ("VOLVER", Some(UiAction::Close)),
        ];
        let bw2 = ((w - 16.0 - 4.0) / 3.0).floor();
        for (k, (label, act)) in lan.into_iter().enumerate() {
            self.draw_button(atlas, font, out, [8.0 + k as f32 * (bw2 + 2.0), h - 38.0, bw2, 16.0], label, act);
        }
        let bw = ((w - 16.0 - 4.0) / 3.0).floor();
        for (k, (label, act)) in buttons.into_iter().enumerate() {
            self.draw_button(atlas, font, out, [8.0 + k as f32 * (bw + 2.0), h - 20.0, bw, 16.0], label, act);
        }
    }

    /// Creación de personaje (createScreen.py, con su mismo layout en 320x180): nombre, clases, las 6
    /// características, las 7 tiradas, ayuda y TIRAR / LIMPIAR / EMPEZAR. Click / toque en una tirada y luego en una
    /// característica la asigna; click en una característica sin tirada elegida deja escribir el número. Esc vuelve.
    /// Arrastrar (mouse o dedo): una tirada a una característica, una característica a otra (se cambian) o afuera (se
    /// quita). Lo arrastrado sigue al puntero.
    fn draw_create(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, classes: &[RosterRow]) {
        const SEL_BG: [f32; 4] = rgba(0x99, 0x88, 0x33, 0xAA);
        const BOX_BG: [f32; 4] = rgba(0x53, 0x56, 0x4D, 0x99);
        let bg = |on: bool| if on { SEL_BG } else { BOX_BG };
        let center = |out: &mut Vec<Sprite>, t: &str, r: [f32; 4], y: f32, col| {
            font.text(out, t, (r[0] + r[2] / 2.0 - (font.width(t) / 2.0).floor()).floor(), y, col)
        };
        out.push(atlas.rect([0.0, 0.0], [w, h], [0.0, 0.0, 0.0, 1.0]));
        self.hot([0.0, 0.0, w, h], UiAction::Create(Cr::Unfocus), None);
        let st = self.create.clone();
        font.text(out, "Crear personaje", 8.0, 3.0, YELLOW);

        // nombre
        let narrow = w < 300.0;
        let r = if narrow { [8.0, 13.0, w - 16.0, 11.0] } else { [110.0, 1.0, w - 118.0, 11.0] };
        let top = if narrow { 27.0 } else { 15.0 };
        out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], bg(st.focus == Some(Focus::Name))));
        let shown = format!("{}{}", st.name, if st.focus == Some(Focus::Name) { "_" } else { "" });
        let (t, col) = if shown.is_empty() { ("Nombre: (click para escribir)".to_string(), GREY) } else { (format!("Nombre: {shown}"), WHITE) };
        font.text(out, &t, r[0] + 2.0, r[1] + 2.0, col);
        self.hot(r, UiAction::Create(Cr::Name), None);

        // clases (4 columnas; en vertical no entran: 2)
        let cols = if w < 300.0 { 2 } else { 4 };
        let step = ((w - 12.0) / cols as f32).floor();
        for (k, c) in classes.iter().enumerate() {
            let r = [8.0 + (k % cols) as f32 * step, top + (k / cols) as f32 * 38.0, step - 4.0, 34.0];
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], bg(k == st.class)));
            if let Some(sheet) = c.sheet {
                out.push(piece(atlas, sheet, [1, 0, 11, 15], [r[0] + 4.0, r[1] + ((r[3] - 15.0) / 2.0).floor()], [11.0, 15.0], WHITE));
            }
            font.text(out, &c.name, r[0] + 18.0, r[1] + 6.0, WHITE);
            font.text(out, &c.status, r[0] + 18.0, r[1] + 18.0, GREY);
            self.hot(r, UiAction::Create(Cr::Class(k)), None);
        }
        let y = top + classes.len().div_ceil(cols) as f32 * 38.0; // 53 con 4 clases en 320

        // características
        let step = ((w - 12.0) / 6.0).floor();
        for (a, name) in ABILITY_NAMES.iter().enumerate() {
            let r = [8.0 + a as f32 * step, y, step - 4.0, 32.0];
            let focused = st.focus == Some(Focus::Stat(a));
            let dragging = st.drag.filter(DragState::moved);
            let drop_here = dragging.is_some_and(|d| inside(r, d.pos));
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], bg(focused || drop_here)));
            center(out, name, r, r[1] + 3.0, YELLOW);
            let shown = st.scores[a].filter(|_| dragging.is_none_or(|d| d.what != Drag::Stat(a))); // la arrastrada, vacía
            if focused {
                center(out, if st.typed.is_empty() { "_" } else { &st.typed }, r, r[1] + 13.0, WHITE);
            } else if let Some(v) = shown {
                center(out, &v.to_string(), r, r[1] + 13.0, WHITE);
            }
            if let (false, Some(v)) = (focused, shown) {
                center(out, &format!("{:+}", dice::modifier(v)), r, r[1] + 22.0, GREY);
            }
            self.hot(r, UiAction::Create(Cr::Stat(a)), None);
        }

        // tiradas, y el detalle de la elegida a la derecha
        font.text(out, "Tiradas (4d6, se quita el menor) - arrastralas:", 8.0, y + 37.0, WHITE);
        let step = ((w - 12.0) / 7.0).floor().min(30.0);
        let cy = y + 47.0;
        for (i, (v, _)) in st.pool.iter().enumerate() {
            let r = [8.0 + i as f32 * step, cy, step - 4.0, 16.0];
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], bg(st.chip == Some(i))));
            center(out, &v.to_string(), r, r[1] + 4.0, if st.source.contains(&Some(i)) { GREY } else { WHITE });
            self.hot(r, UiAction::Create(Cr::Chip(i)), None);
        }
        let mut hy = y + 68.0;
        if let Some(i) = st.chip {
            let (v, d) = st.pool[i];
            let low = d.iter().position(|x| x == d.iter().min().unwrap()).unwrap();
            let parts: Vec<String> = d.iter().enumerate().map(|(j, x)| if j == low { format!("[{x}]") } else { x.to_string() }).collect();
            let detail = format!("{} = {v}", parts.join(" "));
            let x = 8.0 + 7.0 * step + 4.0;
            if x + font.width(&detail) <= w {
                font.text(out, &detail, x, cy + 4.0, GREY);
            } else {
                font.text(out, &detail, 8.0, hy, GREY); // vertical: debajo
                hy += LINE_H;
            }
        }
        let help = "Arrastra una tirada a una caracteristica (sobra 1), o entre caracteristicas para cambiarlas. Click en una caracteristica para escribir el numero (1-20).";
        for line in wrap(font, help, w - 16.0) {
            if hy + LINE_H > h - 21.0 {
                break;
            }
            font.text(out, &line, 8.0, hy, GREY);
            hy += LINE_H;
        }

        // botones
        let ready = dice::valid_scores(&st.scores).is_some();
        let buttons = [
            ("TIRAR", Some(UiAction::Create(Cr::Roll))),
            ("LIMPIAR", Some(UiAction::Create(Cr::Clear))),
            (if ready { "EMPEZAR" } else { "FALTAN STATS" }, ready.then_some(UiAction::Create(Cr::Start))),
        ];
        let bw = ((w - 16.0 - 4.0) / 3.0).floor();
        for (k, (label, act)) in buttons.into_iter().enumerate() {
            self.draw_button(atlas, font, out, [8.0 + k as f32 * (bw + 2.0), h - 20.0, bw, 16.0], label, act);
        }

        // lo que se arrastra sigue al puntero (desde que se movió: un click no lo muestra)
        if let Some(d) = st.drag.filter(DragState::moved) {
            let v = match d.what {
                Drag::Chip(i) => st.pool[i].0,
                Drag::Stat(a) => st.scores[a].unwrap_or(0),
            };
            let r = [(d.pos[0] - 13.0).floor(), (d.pos[1] - 8.0).floor(), 26.0, 16.0];
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], SEL_BG));
            center(out, &v.to_string(), r, r[1] + 4.0, WHITE);
        }

        // teclado en pantalla: Android (NativeActivity) no le pasa a la app las letras del teclado del sistema
        if let (true, Some(f)) = (cfg!(target_os = "android"), st.focus) {
            let rows: &[&str] = if f == Focus::Name { &["1234567890", "QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"] } else { &["1234567890"] };
            self.draw_keys(atlas, font, out, w, h, rows, f == Focus::Name, UiAction::Create);
        }
    }

    /// Teclado en pantalla abajo: las filas, ESPACIO (si `space`), BORRAR y LISTO; cada tecla es `act(Cr::...)`.
    #[allow(clippy::too_many_arguments)]
    fn draw_keys(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, rows: &[&str], space: bool, act: fn(Cr) -> UiAction) {
        let (kw, kh) = (((w - 8.0) / 10.0).floor(), 18.0);
        let total = (rows.len() + 1) as f32 * (kh + 2.0) + 2.0;
        let ktop = h - total;
        out.push(atlas.rect([0.0, ktop - 2.0], [w, total + 2.0], [0.0, 0.0, 0.0, 0.94]));
        self.hot([0.0, ktop - 2.0, w, total + 2.0], UiAction::Nothing, None);
        for (ri, row) in rows.iter().enumerate() {
            let x0 = ((w - row.len() as f32 * kw) / 2.0).floor();
            for (ci, ch) in row.chars().enumerate() {
                let r = [x0 + ci as f32 * kw + 1.0, ktop + ri as f32 * (kh + 2.0), kw - 2.0, kh];
                self.draw_button(atlas, font, out, r, &ch.to_string(), Some(act(Cr::Type(ch))));
            }
        }
        let mut last = vec![("BORRAR", Cr::Back), ("LISTO", Cr::Done)];
        if space {
            last.insert(0, ("ESPACIO", Cr::Type(' ')));
        }
        let y = ktop + rows.len() as f32 * (kh + 2.0);
        let bw = ((w - 8.0 - 2.0 * (last.len() as f32 - 1.0)) / last.len() as f32).floor();
        for (k, (label, cr)) in last.into_iter().enumerate() {
            self.draw_button(atlas, font, out, [4.0 + k as f32 * (bw + 2.0), y, bw, kh], label, Some(act(cr)));
        }
    }

    /// Unirse a una partida LAN: las partidas que contestaron (gris = otra versión o contenido) y el estado.
    #[allow(clippy::too_many_arguments)]
    fn draw_join(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, games: &[(String, bool)], status: &str) {
        out.push(atlas.rect([0.0, 0.0], [w, h], [0.0, 0.0, 0.0, 1.0]));
        self.hot([0.0, 0.0, w, h], UiAction::Nothing, None);
        font.text(out, "Unirse a una partida LAN", 8.0, 3.0, YELLOW);
        if games.is_empty() {
            font.text(out, "Buscando partidas en la red...", 8.0, 18.0, GREY);
        }
        for (i, (label, ok)) in games.iter().enumerate() {
            let r = [8.0, 15.0 + i as f32 * 18.0, w - 16.0, 16.0];
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], SLOT_BG));
            font.text(out, label, r[0] + 4.0, r[1] + 4.0, if *ok { WHITE } else { GREY });
            self.hot(r, UiAction::JoinGame(i), None);
        }
        let help = format!("{status} Juegas con el personaje elegido en la lista. La partida la abre otro con CREAR LAN (misma red Wi-Fi).");
        let lines = wrap(font, help.trim(), w - 16.0);
        for (k, l) in lines.iter().enumerate() {
            let y = h - 24.0 - (lines.len() - k) as f32 * LINE_H;
            font.text(out, l, 8.0, y, if k == 0 && !status.is_empty() { YELLOW } else { GREY });
        }
        self.draw_button(atlas, font, out, [8.0, h - 20.0, w - 16.0, 16.0], "VOLVER", Some(UiAction::OpenRoster));
    }

    /// Sala de espera del host: quién se unió, la IP, EMPEZAR y VOLVER (cierra la partida).
    #[allow(clippy::too_many_arguments)]
    fn draw_lobby(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, players: &[(String, bool)], status: &str) {
        out.push(atlas.rect([0.0, 0.0], [w, h], [0.0, 0.0, 0.0, 1.0]));
        self.hot([0.0, 0.0, w, h], UiAction::Nothing, None);
        font.text(out, "Sala de espera", 8.0, 3.0, YELLOW);
        font.text(out, status, 8.0, 15.0, GREY);
        for (i, (name, _)) in players.iter().enumerate() {
            let r = [8.0, 28.0 + i as f32 * 14.0, w - 16.0, 12.0];
            out.push(atlas.rect([r[0], r[1]], [r[2], r[3]], SLOT_BG));
            let label = if i == 0 { format!("{name} (host)") } else { name.clone() };
            font.text(out, &label, r[0] + 4.0, r[1] + 2.0, if i == 0 { YELLOW } else { WHITE });
        }
        let help = "Los demas entran con UNIRSE desde su lista de personajes (misma red Wi-Fi). Tambien pueden entrar con la partida empezada.";
        let lines = wrap(font, help, w - 16.0);
        for (k, l) in lines.iter().enumerate() {
            font.text(out, l, 8.0, h - 24.0 - (lines.len() - k) as f32 * LINE_H, GREY);
        }
        let bw = ((w - 16.0 - 2.0) / 2.0).floor();
        self.draw_button(atlas, font, out, [8.0, h - 20.0, bw, 16.0], "EMPEZAR", Some(UiAction::StartLan));
        self.draw_button(atlas, font, out, [8.0 + bw + 2.0, h - 20.0, bw, 16.0], "VOLVER", Some(UiAction::OpenRoster));
    }

    /// Pantalla de texto (Mods, Acerca de): párrafos centrados sobre negro; click en cualquier lado vuelve.
    fn draw_page(&mut self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32, paras: &[(String, [f32; 4])]) {
        out.push(atlas.rect([0.0, 0.0], [w, h], [0.0, 0.0, 0.0, 1.0]));
        let mut lines: Vec<(String, [f32; 4])> = Vec::new();
        for (text, color) in paras {
            lines.extend(wrap(font, text, w - 24.0).into_iter().map(|l| (l, *color)));
            lines.push((String::new(), WHITE));
        }
        lines.push(("Esc o click para volver".into(), GREY));
        let mut y = ((h - lines.len() as f32 * 10.0) / 2.0).floor();
        for (line, color) in lines {
            font.text(out, &line, ((w - font.width(&line)) / 2.0).floor(), y, color);
            y += 10.0;
        }
        self.hot([0.0, 0.0, w, h], UiAction::Close, None);
    }

    /// Pantalla de título (TitleScene.java): arcos de fondo que se desplazan (el de adelante al doble), el banner con
    /// los letreros que laten, el dashboard (Jugar, Acerca de), configuración arriba a la izquierda y la versión.
    /// ponytail: sin Mods (1.x); los letreros se suman con alfa, no con mezcla aditiva
    #[allow(clippy::too_many_arguments)]
    pub fn draw_title(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, cfg: &Config, out: &mut Vec<Sprite>, w: f32, h: f32, dt: f32, classes: &[RosterRow], roster: &[RosterRow], lan: &[(String, bool)], lan_status: &str) {
        self.t += dt;
        self.hot.clear();
        // Archs: textura repetida, desplazada en Y (SCROLL_SPEED = 20; la de adelante, 40)
        for (sheet, speed) in [(ARCS_BG, 20.0), (ARCS_FG, 40.0)] {
            let [_, _, tw, th] = atlas.rects[sheet];
            let (tw, th) = (tw as f32, th as f32);
            let off_x = (tw / 4.0).floor() - (w % tw / 2.0).floor();
            let mut y = -((self.t * speed) % th);
            while y < h {
                let mut x = -(off_x.rem_euclid(tw));
                while x < w {
                    out.push(piece(atlas, sheet, [0, 0, tw as u32, th as u32], [x, y.floor()], [tw, th], WHITE));
                    x += tw;
                }
                y += th;
            }
        }
        let item = (((w - 4.0) / 4.0).floor()).min(48.0); // DashboardItem.SIZE (más angosto en vertical)
        let layout_h = 70.0 + item;
        let (tx, ty) = (((w - 128.0) / 2.0).floor(), ((h - layout_h) / 2.0).floor());
        out.push(piece(atlas, BANNERS, [0, 0, 128, 70], [tx, ty], [128.0, 70.0], WHITE));
        let k = (-self.t).sin().max(0.0); // letreros: alfa = sin(-t), negativo = invisible
        if k > 0.0 {
            out.push(piece(atlas, BANNERS, [0, 161, 128, 57], [tx, ty], [128.0, 57.0], [1.0, 1.0, 1.0, k]));
        }
        let iy = ((h + layout_h) / 2.0).floor() - item;
        // separación: el texto más largo + 8 px de aire (antes "Acerca de" y "Opciones" se tocaban); si no entra, lo que haya
        let widest = ["Jugar", "Mods", "Acerca de", "Opciones"].iter().map(|l| font.width(l)).fold(0.0, f32::max);
        let step = (widest + 8.0).max(item).min(((w - 4.0) / 4.0).floor());
        let x0 = (w / 2.0).floor() - step * 2.0;
        for (icon, label, x, act) in [
            // Rankings y Badges del original se reemplazaron por Mods; la configuración (PrefsButton) es un botón más
            (0u32, "Jugar", x0, UiAction::OpenRoster),
            (2, "Mods", x0 + step, UiAction::Open(Window::Page(vec![
                ("Mods".into(), YELLOW),
                ("No hay mods instalados. El sistema de mods todavia no esta disponible.".into(), WHITE),
                ("Los mods vienen de fuentes externas: ten cuidado de donde los descargas y revisalos antes de instalarlos.".into(), rgba(190, 190, 190, 255)),
                ("Los mods que traen codigo pueden cambiar el juego de forma considerable y hacer cualquier cosa en tu equipo: instalalos solo si confias en su origen.".into(), rgba(255, 120, 80, 255)),
            ]))),
            (1, "Acerca de", x0 + step * 2.0, UiAction::Open(Window::Page(vec![
                (format!("PixelDungeon-RollInitiative v{}", game::VERSION), YELLOW),
                ("Roguelike basado en Pixel Dungeon de Watabou (GPL v3), con las reglas de D&D 5e y multijugador cooperativo.".into(), WHITE),
                ("Arte y sonido del original. Todo el proyecto se distribuye como GPL v3.".into(), rgba(190, 190, 190, 255)),
            ]))),
            (u32::MAX, "Opciones", x0 + step * 3.0, UiAction::Open(Window::Settings)),
        ] {
            let ix = x + ((step - 32.0) / 2.0).floor();
            if icon == u32::MAX {
                // ponytail: el ícono de PrefsButton (Icons.PREFS, 16x16) al doble hasta que haya uno de 32x32
                out.push(piece(atlas, ICONS, [30, 0, 16, 16], [ix, iy], [32.0, 32.0], WHITE));
            } else {
                out.push(piece(atlas, DASHBOARD, [icon * 32, 0, 32, 32], [ix, iy], [32.0, 32.0], WHITE));
            }
            font.text(out, label, x + ((step - font.width(label)) / 2.0).floor(), iy + 34.0, WHITE);
            self.hot([x, iy, step, item], act, None);
        }
        out.push(piece(atlas, ICONS, ICON_EXIT, [w - 17.0, 1.0], [16.0, 16.0], WHITE));
        self.hot([w - 18.0, 0.0, 18.0, 18.0], UiAction::Quit, Some("Salir"));
        let v = format!("v {}", game::VERSION);
        font.text(out, &v, w - font.width(&v) - 1.0, h - font.height() - 1.0, GREY);
        match self.window.clone() {
            Some(Window::Settings) => self.draw_settings(atlas, font, cfg, out, w, h),
            Some(Window::Controls) => self.draw_controls(atlas, font, cfg, out, w, h),
            Some(Window::Page(paras)) => self.draw_page(atlas, font, out, w, h, &paras),
            Some(Window::Info { icon, title, text, hp }) => self.draw_info(atlas, font, chars, d, out, w, h, icon, &title, &text, hp),
            Some(Window::Create) => self.draw_create(atlas, font, out, w, h, classes),
            Some(Window::Roster) => self.draw_roster(atlas, font, out, w, h, roster),
            Some(Window::Join) => self.draw_join(atlas, font, out, w, h, lan, lan_status),
            Some(Window::Lobby) => self.draw_lobby(atlas, font, out, w, h, lan, lan_status),
            _ => self.draw_tooltip(atlas, font, out, w, h),
        }
    }

    /// Texto de la zona bajo el mouse (la de más arriba), encima del mouse sin salirse de la pantalla.
    fn draw_tooltip(&self, atlas: &Atlas, font: &Font, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let Some(m) = self.mouse else { return };
        let Some(tip) = self.hot.iter().rev().find(|t| inside(t.rect, m)).and_then(|t| t.tip.clone()) else { return };
        let (bw, bh) = (font.width(&tip) + 6.0, font.height() + 4.0);
        let x = (m[0] - bw / 2.0).floor().clamp(0.0, (w - bw).max(0.0));
        let y = (m[1] - 4.0 - bh).floor().clamp(0.0, (h - bh).max(0.0));
        out.push(atlas.rect([x, y], [bw, bh], TIP_BG));
        font.text(out, &tip, x + 3.0, y + 2.0, WHITE);
    }
}

/// Un ítem de 16x16 (ItemSpriteSheet): su recorte de items.png (8 columnas) o, sin sprite, su nombre achicado para
/// entrar en 16x16, una palabra por línea (TextIcon).
pub fn item_icon(out: &mut Vec<Sprite>, atlas: &Atlas, font: &Font, image: &Image, pos: [f32; 2], alpha: f32) {
    match image {
        Image::Sheet(i) => {
            let i = *i as u32;
            out.push(piece(atlas, ITEMS, [(i % 8) * 16, (i / 8) * 16, 16, 16], pos, [16.0, 16.0], [1.0, 1.0, 1.0, alpha]));
        }
        Image::Text(name) => {
            // a tamaño real (achicar una fuente pixel la vuelve ilegible): hasta 2 palabras, recortadas para entrar en
            // 16 px; el nombre completo está en el tooltip y en la ventana del ítem
            let lines: Vec<String> = name
                .split_whitespace()
                .filter(|w| w.chars().count() > 2 || name.split_whitespace().count() == 1) // sin "de", "la"
                .take(2)
                .map(|w| {
                    let mut cut: String = w.to_string();
                    while font.width(&cut) > 16.0 && cut.chars().count() > 1 {
                        cut.pop();
                    }
                    cut
                })
                .collect();
            let th = lines.len().max(1) as f32 * font.height();
            let mut y = (pos[1] + (16.0 - th) / 2.0).floor();
            for line in lines {
                let x = (pos[0] + (16.0 - font.width(&line)) / 2.0).floor();
                font.text(out, &line, x, y, [1.0, 1.0, 1.0, alpha]);
                y += font.height();
            }
        }
    }
}

/// Los montones de ítems del piso: se recuerdan donde ya se vio (la niebla los oscurece).
pub fn draw_heaps(out: &mut Vec<Sprite>, atlas: &Atlas, font: &Font, d: &Dungeon, ox: f32, oy: f32) {
    let level = d.level();
    for (&pos, heap) in &level.heaps {
        if level.visited[pos as usize] || level.mapped[pos as usize] {
            let at = [ox + (pos % game::geom::WIDTH) as f32 * 16.0, oy + (pos / game::geom::WIDTH) as f32 * 16.0];
            item_icon(out, atlas, font, &d.heap_image(heap), at, 1.0);
        }
    }
}

/// Borde de 1 pixel (opción elegida, tecla que se está cambiando).
fn outline(out: &mut Vec<Sprite>, atlas: &Atlas, r: [f32; 4], color: [f32; 4]) {
    out.push(atlas.rect([r[0], r[1]], [r[2], 1.0], color));
    out.push(atlas.rect([r[0], r[1] + r[3] - 1.0], [r[2], 1.0], color));
    out.push(atlas.rect([r[0], r[1]], [1.0, r[3]], color));
    out.push(atlas.rect([r[0] + r[2] - 1.0, r[1]], [1.0, r[3]], color));
}

/// Una fila de la lista de personajes (la arma la app al abrirla).
pub struct RosterRow {
    pub name: String,
    /// "en partida - piso 2, nivel 3" o la clase de 5e y su dado de golpe.
    pub status: String,
    pub stats: String,
    /// Hoja del atlas de su clase (el avatar); None = clase que no está cargada.
    pub sheet: Option<usize>,
}

/// Qué recibe lo que se escribe en la creación de personaje.
#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Name,
    Stat(usize),
}

/// Clicks de la creación de personaje.
#[derive(Clone)]
pub enum Cr {
    /// Abrir la creación desde cero.
    New,
    Name,
    Class(usize),
    Stat(usize),
    Chip(usize),
    Roll,
    Clear,
    Unfocus,
    Start,
    /// Teclado en pantalla: una tecla, borrar, listo.
    Type(char),
    Back,
    Done,
}

const NAME_MAX: usize = 16;

/// Estado de la creación de personaje (CreateState del prototipo).
#[derive(Clone, Default)]
pub struct Create {
    pub name: String,
    pub class: usize,
    pub scores: [Option<i32>; 6],
    /// La tirada asignada a cada característica (None = escrita a mano o vacía).
    source: [Option<usize>; 6],
    pool: Vec<(i32, [i32; 4])>,
    /// Tirada elegida con el primer click (el segundo, en una característica, la asigna).
    chip: Option<usize>,
    pub focus: Option<Focus>,
    typed: String,
    /// Lo que se está arrastrando (apretado sobre una tirada o una característica con valor).
    pub drag: Option<DragState>,
}

/// Qué se arrastra.
#[derive(Clone, Copy, PartialEq)]
pub enum Drag {
    Chip(usize),
    Stat(usize),
}

#[derive(Clone, Copy)]
pub struct DragState {
    pub what: Drag,
    pub start: [f32; 2],
    pub pos: [f32; 2],
}

/// Menos que esto (pixeles de la UI) entre apretar y soltar = click (como el prototipo).
const DRAG_MIN: f32 = 3.0;

impl DragState {
    fn moved(&self) -> bool {
        (self.pos[0] - self.start[0]).abs() + (self.pos[1] - self.start[1]).abs() >= DRAG_MIN
    }
}

impl Create {
    /// Un click. `seed` sólo se usa al tirar.
    pub fn apply(&mut self, c: Cr, seed: i64) {
        match c {
            Cr::New => *self = Create::default(),
            Cr::Name => (self.focus, self.chip) = (Some(Focus::Name), None),
            Cr::Class(k) => self.class = k,
            Cr::Unfocus => self.focus = None,
            Cr::Clear => {
                // como el prototipo: borra las características; nombre, clase y tiradas quedan
                (self.scores, self.source, self.chip, self.focus) = ([None; 6], [None; 6], None, None);
                self.typed.clear();
            }
            Cr::Roll => {
                self.pool = dice::roll_ability_pool(&mut game::rng::Rng::new(seed));
                for a in 0..6 {
                    if self.source[a].take().is_some() {
                        self.scores[a] = None; // lo asignado desde tiradas viejas se borra
                    }
                }
                self.chip = None;
            }
            Cr::Chip(i) => (self.chip, self.focus) = (if self.chip == Some(i) { None } else { Some(i) }, None),
            Cr::Stat(a) => match self.chip.take() {
                Some(i) => self.assign(i, a),
                None => (self.focus, self.typed) = (Some(Focus::Stat(a)), String::new()),
            },
            Cr::Start => {}
            Cr::Type(c) => {
                // nombre: mayúscula al empezar cada palabra
                let up = self.focus != Some(Focus::Name) || self.name.is_empty() || self.name.ends_with(' ');
                let c = if up { c } else { c.to_ascii_lowercase() };
                self.key(winit::keyboard::KeyCode::KeyA, Some(&c.to_string()));
            }
            Cr::Back => _ = self.key(winit::keyboard::KeyCode::Backspace, None),
            Cr::Done => self.focus = None,
        }
    }

    /// Pone la tirada `i` en `a`; si estaba en otra característica, se mueve.
    fn assign(&mut self, i: usize, a: usize) {
        if let Some(o) = self.source.iter().position(|s| *s == Some(i)) {
            (self.scores[o], self.source[o]) = (None, None);
        }
        (self.scores[a], self.source[a]) = (Some(self.pool[i].0), Some(i));
    }

    /// Apretar: sobre una tirada, o sobre una característica con valor sin tirada elegida, empieza a arrastrar.
    pub fn press(&mut self, under: Option<&UiAction>, p: [f32; 2]) {
        let what = match under {
            Some(UiAction::Create(Cr::Chip(i))) => Drag::Chip(*i),
            Some(UiAction::Create(Cr::Stat(a))) if self.scores[*a].is_some() && self.chip.is_none() => Drag::Stat(*a),
            _ => return,
        };
        self.drag = Some(DragState { what, start: p, pos: p });
    }

    pub fn moved_to(&mut self, p: [f32; 2]) {
        if let Some(d) = &mut self.drag {
            d.pos = p;
        }
    }

    /// Soltar sobre `under`. Sin moverse es un click (el de la tirada o la característica). Devuelve false si no se
    /// estaba arrastrando (el click sigue su camino normal).
    pub fn release(&mut self, under: Option<&UiAction>, p: [f32; 2]) -> bool {
        let Some(mut d) = self.drag.take() else { return false };
        d.pos = p;
        let target = match under {
            Some(UiAction::Create(Cr::Stat(b))) => Some(*b),
            _ => None,
        };
        match (d.moved(), d.what, target) {
            (false, Drag::Chip(i), _) => self.apply(Cr::Chip(i), 0),
            (false, Drag::Stat(a), _) => self.apply(Cr::Stat(a), 0),
            (true, Drag::Chip(i), Some(b)) => {
                self.assign(i, b);
                self.chip = None;
            }
            (true, Drag::Stat(a), Some(b)) if a != b => {
                self.scores.swap(a, b);
                self.source.swap(a, b);
            }
            (true, Drag::Stat(_), Some(_)) => {}
            // arrastrada afuera: se quita (la tirada vuelve a la lista)
            (true, Drag::Stat(a), None) => (self.scores[a], self.source[a]) = (None, None),
            (true, Drag::Chip(_), None) => {}
        }
        true
    }

    /// Una tecla y lo que escribe (`text`). False si no había nada enfocado (la tecla sigue su camino).
    pub fn key(&mut self, code: winit::keyboard::KeyCode, text: Option<&str>) -> bool {
        use winit::keyboard::KeyCode as K;
        let next = matches!(code, K::Enter | K::NumpadEnter | K::Tab);
        match self.focus {
            None => return false,
            Some(Focus::Name) => match code {
                K::Backspace => _ = self.name.pop(),
                _ if next => (self.focus, self.typed) = (Some(Focus::Stat(0)), String::new()),
                _ => {
                    for ch in text.unwrap_or("").chars().filter(|c| !c.is_control()) {
                        if self.name.chars().count() < NAME_MAX {
                            self.name.push(ch);
                        }
                    }
                }
            },
            Some(Focus::Stat(a)) => {
                if next {
                    (self.focus, self.typed) = (Some(Focus::Stat((a + 1) % 6)), String::new());
                    return true;
                }
                match (code, text.and_then(|t| t.chars().next())) {
                    (K::Backspace, _) => _ = self.typed.pop(),
                    (_, Some(d)) if d.is_ascii_digit() && self.typed.len() < 2 => self.typed.push(d),
                    _ => return true,
                }
                self.scores[a] = self.typed.parse().ok().filter(|v| (dice::MIN_SCORE..=dice::MAX_SCORE).contains(v));
                self.source[a] = None;
            }
        }
        true
    }
}
