//! UI de la partida (`screens/gameUi.py` + el log y examinar de `screens/gameScreen.py`) con los assets originales:
//! panel de estado, carrusel de iniciativa, barra de herramientas, log, tooltips y ventanas (examinar, ficha, menú).
//! UI inmediata: cada frame se dibuja todo y se registran zonas clickeables; el click busca de arriba hacia abajo.
//! R5: mochila (WndBag), ventana del ítem (WndItem), accesos rápidos e ícono de hambre.
//! ponytail: sin íconos de estados ni configuración todavía (R5)
use game::actor::ActorId;
use game::dice::{ABILITIES, ABILITY_NAMES};
use game::dungeon::Dungeon;
use game::geom::LENGTH;
use game::hero::{Action, Slot, Verb, BACKPACK_SIZE, HUNGRY};
use game::items::{weapon_dice, Image, Item, SLOT_ARMOR, SLOT_RING, SLOT_WEAPON};
use game::registry::text;
use game::terrain;

use crate::atlas::Atlas;
use crate::font::Font;
use crate::gpu::Sprite;
use crate::sprites::Chars;
use crate::{BUFFS, CHROME, EXP_BAR, HP_BAR, ICONS, ITEMS, STATUS, TILES, TOOLBAR, WATER};

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
const SLOT_BG: [f32; 4] = rgba(0x53, 0x56, 0x4D, 0x99);
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
    /// Mouse en la capa de UI (tooltips); None en táctil.
    pub mouse: Option<[f32; 2]>,
    hot: Vec<Hot>,
    car_cur: Option<ActorId>,
    car_off: f32,
    t: f32,
}

impl Ui {
    pub fn new() -> Self {
        Self { window: None, examining: false, mouse: None, hot: Vec::new(), car_cur: None, car_off: 0.0, t: 0.0 }
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
    pub fn draw(&mut self, atlas: &Atlas, font: &Font, chars: &Chars, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32, dt: f32, fps: u32) {
        self.t += dt;
        self.hot.clear();
        self.draw_log(font, d, out, w, h);
        self.draw_status(atlas, font, chars, d, out, w, dt);
        font.text(out, &format!("FPS {fps}"), 2.0, 34.0, GREY);
        self.draw_toolbar(atlas, font, d, out, w, h);
        match self.window.clone() {
            Some(Window::Info { icon, title, text, hp }) => self.draw_info(atlas, font, chars, d, out, w, h, icon, &title, &text, hp),
            Some(Window::Hero) => self.draw_hero(atlas, font, d, out, w, h),
            Some(Window::Menu) => self.draw_menu(atlas, font, d, out, w, h),
            Some(Window::Bag(mode)) => self.draw_bag(atlas, font, d, out, w, h, &mode),
            Some(Window::Item(slot)) => self.draw_item(atlas, font, d, out, w, h, slot),
            None => {
                self.draw_tooltip(atlas, font, out, w, h);
                if self.examining {
                    let msg = "Toca una casilla para examinarla";
                    let tw = font.width(msg);
                    let x = ((w - tw - 8.0) / 2.0).floor();
                    out.push(atlas.rect([x, 34.0], [tw + 8.0, font.height() + 4.0], PROMPT_BG));
                    font.text(out, msg, x + 4.0, 36.0, WHITE);
                }
            }
        }
    }

    fn draw_log(&self, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let bottom = h - TOOLBAR_H - 1.0;
        let first = d.messages.len().saturating_sub(LOG_LINES);
        for (i, msg) in d.messages[first..].iter().enumerate() {
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
        let rows = 1 + BACKPACK_SIZE.div_ceil(BAG_COLS);
        let cw = SLOT_SIZE * BAG_COLS as f32 + SLOT_MARGIN * (BAG_COLS - 1) as f32;
        let ch = BAG_TITLE_H + SLOT_SIZE * rows as f32 + SLOT_MARGIN * (rows - 1) as f32 + BAG_GAP;
        let c = [((w - cw) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), cw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None); // click afuera = cerrar
        Self::draw_window(atlas, out, c);
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
            slots.push((cell(k % BAG_COLS, 1 + k / BAG_COLS), (k < me.backpack.len()).then_some(Slot::Pack(k)), None));
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
        Self::draw_slot(atlas, font, d, out, cell(BAG_COLS - 1, 0), gold.as_ref(), Some(14), true);
    }

    /// WndItem: ícono, nombre, descripción con los números de 5e y un botón por acción.
    #[allow(clippy::too_many_arguments)]
    fn draw_item(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32, slot: Slot) {
        let me = d.local();
        let Some(item) = me.item(slot) else {
            self.window = None; // se gastó o se soltó
            return;
        };
        let name = wrap(font, &d.item_title(item), ITEM_WND_W - 20.0);
        let info = wrap(font, &d.item_info(item, me), ITEM_WND_W);
        let btn_h = 16.0;
        let head = (name.len() as f32 * 9.0).max(16.0);
        let ch = head + 4.0 + info.len() as f32 * 9.0 + 4.0 + btn_h;
        let c = [((w - ITEM_WND_W) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), ITEM_WND_W, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None);
        Self::draw_window(atlas, out, c);
        self.hot(c, UiAction::Nothing, None);
        item_icon(out, atlas, font, &d.item_image(item), [c[0], c[1]], 1.0);
        for (k, line) in name.iter().enumerate() {
            font.text(out, line, c[0] + 20.0, c[1] + 4.0 + k as f32 * 9.0, YELLOW);
        }
        for (k, line) in info.iter().enumerate() {
            font.text(out, line, c[0], c[1] + head + 4.0 + k as f32 * 9.0, WHITE);
        }
        let verbs = me.verbs(slot);
        let bw = ((ITEM_WND_W - (verbs.len() as f32 - 1.0) * 2.0) / verbs.len() as f32).floor();
        for (k, v) in verbs.into_iter().enumerate() {
            let rect = [c[0] + k as f32 * (bw + 2.0), c[1] + ch - btn_h, bw, btn_h];
            self.draw_button(atlas, font, out, rect, v.label(), Some(UiAction::Verb(slot, v)));
        }
    }

    fn draw_window(atlas: &Atlas, out: &mut Vec<Sprite>, content: [f32; 4]) {
        let (region, m) = CHROME_WINDOW;
        nine(out, atlas, region, m, [content[0] - m, content[1] - m, content[2] + 2.0 * m, content[3] + 2.0 * m], WHITE);
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
        if !me.ch.buffs.is_empty() {
            let names: Vec<String> = me.ch.buffs.iter().map(|b| b.name()).collect();
            lines.push((format!("Estados: {}", names.join(", ")), STATES));
        }
        let cw = lines.iter().map(|l| font.width(&l.0)).fold(0.0, f32::max);
        let ch = lines.len() as f32 * LINE_H;
        let c = [((w - cw) / 2.0).floor(), ((h - TOOLBAR_H - ch) / 2.0).floor(), cw, ch];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None);
        Self::draw_window(atlas, out, c);
        for (i, (line, color)) in lines.iter().enumerate() {
            font.text(out, line, c[0], c[1] + i as f32 * LINE_H, *color);
        }
    }

    /// WndGame: menú de la partida (botón de menú o Esc). Nueva partida sólo si el héroe murió, como el original.
    /// ponytail: sin configuración, reglas ni menú principal todavía (R5)
    fn draw_menu(&mut self, atlas: &Atlas, font: &Font, d: &Dungeon, out: &mut Vec<Sprite>, w: f32, h: f32) {
        let mut rows: Vec<(&str, UiAction)> = Vec::new();
        if !d.local().is_alive() {
            rows.push(("Nueva partida", UiAction::NewGame));
        }
        rows.push(("Salir", UiAction::Quit));
        rows.push(("Volver al juego", UiAction::Close));
        let mh = rows.len() as f32 * (MENU_BTN_H + MENU_GAP) - MENU_GAP;
        let c = [((w - MENU_W) / 2.0).floor(), ((h - TOOLBAR_H - mh) / 2.0).floor(), MENU_W, mh];
        self.hot([0.0, 0.0, w, h], UiAction::Close, None); // click afuera cierra
        self.hot(c, UiAction::Nothing, None);
        Self::draw_window(atlas, out, c);
        for (r, (label, act)) in rows.into_iter().enumerate() {
            let rect = [c[0], c[1] + r as f32 * (MENU_BTN_H + MENU_GAP), MENU_W, MENU_BTN_H];
            self.draw_button(atlas, font, out, rect, label, Some(act));
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
            let words: Vec<&str> = name.split_whitespace().collect();
            let tw = words.iter().map(|w| font.width(w)).fold(1.0, f32::max);
            let th = words.len().max(1) as f32 * font.height();
            let f = (16.0 / tw).min(16.0 / th).min(1.0);
            let mut y = pos[1] + (16.0 - th * f) / 2.0;
            for word in words {
                let x = pos[0] + (16.0 - font.width(word) * f) / 2.0;
                font.text_scaled(out, word, x, y, [1.0, 1.0, 1.0, alpha], f);
                y += font.height() * f;
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
