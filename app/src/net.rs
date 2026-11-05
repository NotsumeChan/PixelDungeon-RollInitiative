//! Red LAN (R6, `net/` del prototipo), sólo std: descubrimiento por UDP y la partida por TCP, un JSON por línea.
//! Host autoritativo: corre la partida y manda el estado completo (`save::dump`) a cada cliente; los clientes dibujan
//! y mandan acciones. Los hilos de red hablan con el juego sólo por canales: nunca tocan `game` ni el render.
//! Descubrimiento: el cliente manda "discover" por broadcast y el host le contesta directo (unicast). Así el teléfono
//! encuentra la partida de la PC sin MulticastLock (recibir broadcasts en Android lo pide; recibir unicast no).
//! Mensajes: cliente -> host `hello` (versión, hash del contenido, personaje) y `action`; host -> cliente `state`
//! (`you` = su índice en `heroes`) y `refused`. Un mensaje nunca ejecuta código: es JSON y se valida al aplicarlo.
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use game::actor::ActorId;
use game::dungeon::{DiceEvent, Dungeon, Fx, Tone};
use game::geom::LENGTH;
use game::hero::{Action, Hero, Slot, Verb};
use game::{characters, save};
use serde_json::{json, Map, Value};

pub const UDP_PORT: u16 = 47810;
pub const TCP_PORT: u16 = 47811;
const MAGIC: &str = "pdri";
/// Un estado completo entra holgado; una línea más larga corta la conexión.
const MAX_LINE: u64 = 8 * 1024 * 1024;
/// Sin respuesta en este tiempo, la partida deja de aparecer en la lista.
const GAME_TTL: Duration = Duration::from_secs(3);
const DISCOVER_EVERY: Duration = Duration::from_secs(1);
/// Como mucho un estado cada tanto (y sólo si cambió).
/// ponytail: estado completo; con mapas grandes o internet, mandar sólo diferencias
const STATE_EVERY: Duration = Duration::from_millis(50);
const NAME_MAX: usize = 16;

fn line(msg: &Value) -> Vec<u8> {
    let mut v = serde_json::to_vec(msg).unwrap_or_default();
    v.push(b'\n');
    v
}

/// Un hilo que lee líneas JSON de `stream` y las pasa por `tx`; al cortarse (o con basura) manda {"t": "leave"}.
fn read_loop<T: Send + 'static>(stream: TcpStream, tx: Sender<T>, wrap: impl Fn(Value) -> T + Send + 'static) {
    std::thread::spawn(move || {
        let mut r = BufReader::new(stream);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match r.by_ref().take(MAX_LINE).read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) if buf.last() != Some(&b'\n') => break, // más larga que MAX_LINE
                Ok(_) => {}
            }
            let Ok(msg) = serde_json::from_slice::<Value>(&buf) else { break };
            if !msg["t"].is_string() || msg["t"] == "leave" {
                break; // sin tipo o haciéndose pasar por la red
            }
            if tx.send(wrap(msg)).is_err() {
                return; // el juego ya no escucha
            }
        }
        let _ = tx.send(wrap(json!({"t": "leave"})));
    });
}

/// La IP de esta máquina en la LAN (para mostrarla). UDP `connect` no manda nada: sólo elige la interfaz.
pub fn local_ip() -> String {
    UdpSocket::bind(("0.0.0.0", 0))
        .and_then(|s| {
            s.connect(("8.8.8.8", 80))?;
            s.local_addr()
        })
        .map_or_else(|_| "?".to_string(), |a| a.ip().to_string())
}

// --- transporte ---

/// Acepta clientes por TCP y contesta los "discover" por UDP. inbox: (id de cliente, mensaje); al conectarse uno
/// llega {"t": "join"} y al irse {"t": "leave"}.
struct Host {
    inbox: Receiver<(u32, Value)>,
    writers: Arc<Mutex<BTreeMap<u32, TcpStream>>>,
    info: Arc<Mutex<Value>>,
    stop: Arc<AtomicBool>,
}

impl Host {
    fn new(name: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", TCP_PORT))?;
        listener.set_nonblocking(true)?; // para poder cerrar el hilo
        let udp = UdpSocket::bind(("0.0.0.0", UDP_PORT))?;
        udp.set_read_timeout(Some(Duration::from_millis(300)))?;
        let info = json!({"magic": MAGIC, "t": "game", "name": name, "players": 1, "version": game::VERSION,
                          "hash": game::registry::content_hash(), "port": TCP_PORT});
        let info = Arc::new(Mutex::new(info));
        let writers: Arc<Mutex<BTreeMap<u32, TcpStream>>> = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, inbox) = channel();
        {
            let (writers, stop) = (writers.clone(), stop.clone());
            std::thread::spawn(move || {
                let mut next = 1u32;
                while !stop.load(Ordering::Relaxed) {
                    let Ok((s, _)) = listener.accept() else {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    };
                    let _ = s.set_nonblocking(false);
                    let _ = s.set_nodelay(true);
                    // un cliente que no lee no frena al host para siempre
                    let _ = s.set_write_timeout(Some(Duration::from_secs(2)));
                    let Ok(w) = s.try_clone() else { continue };
                    let cid = next;
                    next += 1;
                    writers.lock().unwrap().insert(cid, w);
                    let _ = tx.send((cid, json!({"t": "join"})));
                    read_loop(s, tx.clone(), move |m| (cid, m));
                }
            });
        }
        {
            let (info, stop) = (info.clone(), stop.clone());
            std::thread::spawn(move || {
                let mut buf = [0u8; 2048];
                while !stop.load(Ordering::Relaxed) {
                    let Ok((n, from)) = udp.recv_from(&mut buf) else { continue };
                    let asks = serde_json::from_slice::<Value>(&buf[..n]).is_ok_and(|m| m["magic"] == MAGIC && m["t"] == "discover");
                    if asks {
                        let msg = info.lock().unwrap().to_string();
                        let _ = udp.send_to(msg.as_bytes(), from);
                    }
                }
            });
        }
        Ok(Self { inbox, writers, info, stop })
    }

    fn send_raw(&self, cid: u32, data: &[u8]) {
        let mut w = self.writers.lock().unwrap();
        if let Some(s) = w.get_mut(&cid)
            && s.write_all(data).is_err()
        {
            w.remove(&cid);
        }
    }

    fn send(&self, cid: u32, msg: &Value) {
        self.send_raw(cid, &line(msg));
    }

    fn kick(&self, cid: u32) {
        if let Some(s) = self.writers.lock().unwrap().remove(&cid) {
            let _ = s.shutdown(Shutdown::Both);
        }
    }

    fn set_players(&self, n: usize) {
        self.info.lock().unwrap()["players"] = json!(n);
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        for s in self.writers.lock().unwrap().values() {
            let _ = s.shutdown(Shutdown::Both);
        }
    }
}

/// Una partida encontrada en la LAN.
pub struct Game {
    pub addr: SocketAddr,
    pub info: Value,
}

/// Busca partidas: manda "discover" por broadcast cada segundo y junta las respuestas.
pub struct Browser {
    seen: Arc<Mutex<Vec<(Instant, SocketAddr, Value)>>>,
    stop: Arc<AtomicBool>,
}

impl Browser {
    pub fn new() -> std::io::Result<Self> {
        let s = UdpSocket::bind(("0.0.0.0", 0))?;
        s.set_broadcast(true)?;
        s.set_read_timeout(Some(Duration::from_millis(250)))?;
        let seen: Arc<Mutex<Vec<(Instant, SocketAddr, Value)>>> = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (seen2, stop2) = (seen.clone(), stop.clone());
        std::thread::spawn(move || {
            let ask = json!({"magic": MAGIC, "t": "discover"}).to_string();
            let mut last: Option<Instant> = None;
            let mut buf = [0u8; 2048];
            while !stop2.load(Ordering::Relaxed) {
                if last.is_none_or(|t| t.elapsed() >= DISCOVER_EVERY) {
                    let _ = s.send_to(ask.as_bytes(), ("255.255.255.255", UDP_PORT)); // sin red: se reintenta
                    last = Some(Instant::now());
                }
                let Ok((n, from)) = s.recv_from(&mut buf) else { continue };
                let Ok(info) = serde_json::from_slice::<Value>(&buf[..n]) else { continue };
                let Some(port) = info["port"].as_u64().and_then(|p| u16::try_from(p).ok()) else { continue };
                if info["magic"] == MAGIC && info["t"] == "game" {
                    let addr = SocketAddr::new(from.ip(), port);
                    let mut v = seen2.lock().unwrap();
                    v.retain(|e| e.1 != addr);
                    v.push((Instant::now(), addr, info));
                }
            }
        });
        Ok(Self { seen, stop })
    }

    pub fn games(&self) -> Vec<Game> {
        let v = self.seen.lock().unwrap();
        v.iter().filter(|e| e.0.elapsed() < GAME_TTL).map(|e| Game { addr: e.1, info: e.2.clone() }).collect()
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

// --- acciones como datos (frontera de confianza: lo que no valida se ignora) ---

const VERBS: [Verb; 7] = [Verb::Eat, Verb::Drink, Verb::Read, Verb::Equip, Verb::Unequip, Verb::Throw, Verb::Drop];

fn slot_v(s: Slot) -> Value {
    match s {
        Slot::Weapon => json!("w"),
        Slot::Armor => json!("a"),
        Slot::Pack(k) => json!(k),
    }
}

fn slot_of(v: &Value) -> Option<Slot> {
    match v.as_str() {
        Some("w") => Some(Slot::Weapon),
        Some("a") => Some(Slot::Armor),
        Some(_) => None,
        None => v.as_u64().map(|k| Slot::Pack(k as usize)),
    }
}

fn actor_v(a: ActorId) -> Value {
    match a {
        ActorId::Hero(i) => json!(["h", i]),
        ActorId::Mob(id) => json!(["m", id]),
        _ => Value::Null,
    }
}

fn actor_of(v: &Value) -> Option<ActorId> {
    let n = v[1].as_u64()?;
    match v[0].as_str()? {
        "h" => Some(ActorId::Hero(n as usize)),
        "m" => Some(ActorId::Mob(u32::try_from(n).ok()?)),
        _ => None,
    }
}

pub fn action_v(a: Action) -> Value {
    match a {
        Action::Move(c) => json!(["move", c]),
        Action::Attack(e) => json!(["attack", actor_v(e)]),
        Action::Help(j) => json!(["help", j]),
        Action::Rest => json!(["rest"]),
        Action::Search => json!(["search"]),
        Action::Descend(c) => json!(["descend", c]),
        Action::Ascend(c) => json!(["ascend", c]),
        Action::PickUp(c) => json!(["pickup", c]),
        Action::Open(c) => json!(["open", c]),
        Action::Use(s, v, t) => json!(["use", slot_v(s), VERBS.iter().position(|x| *x == v), t.map(slot_v)]),
        Action::Cast(k, c) => json!(["cast", k, c]),
        Action::SecondWind => json!(["second_wind"]),
        Action::Throw(s, c) => json!(["throw", slot_v(s), c]),
    }
}

pub fn action_of(v: &Value) -> Option<Action> {
    let cell = |k: usize| v[k].as_i64().and_then(|c| i32::try_from(c).ok()).filter(|c| (0..LENGTH).contains(c));
    let idx = |k: usize| v[k].as_u64().map(|x| x as usize);
    Some(match v[0].as_str()? {
        "move" => Action::Move(cell(1)?),
        "attack" => Action::Attack(actor_of(&v[1])?),
        "help" => Action::Help(idx(1)?),
        "rest" => Action::Rest,
        "search" => Action::Search,
        "descend" => Action::Descend(cell(1)?),
        "ascend" => Action::Ascend(cell(1)?),
        "pickup" => Action::PickUp(cell(1)?),
        "open" => Action::Open(cell(1)?),
        "use" => {
            let target = if v[3].is_null() { None } else { Some(slot_of(&v[3])?) };
            Action::Use(slot_of(&v[1])?, *VERBS.get(idx(2)?)?, target)
        }
        "cast" => Action::Cast(idx(1)?, cell(2)?),
        "second_wind" => Action::SecondWind,
        "throw" => Action::Throw(slot_of(&v[1])?, cell(2)?),
        _ => return None,
    })
}

// --- efectos y dados (el cliente los ve como el host: animaciones, números flotantes, dados) ---

const ANIMS: [&str; 4] = ["attack", "operate", "read", "die"];
const TONES: [Tone; 5] = [Tone::Default, Tone::Pos, Tone::Neg, Tone::Warn, Tone::Neutral];

fn fx_v(f: &Fx) -> Value {
    match f {
        Fx::Anim { who, kind, cell } => json!(["anim", actor_v(*who), kind, cell]),
        Fx::Status { who, text, tone } => json!(["status", actor_v(*who), text, TONES.iter().position(|t| t == tone)]),
        Fx::Die { who, cell, stats } => json!(["die", actor_v(*who), cell, stats]),
        Fx::Checked(c) => json!(["checked", c]),
        Fx::Leaves(c) => json!(["leaves", c]),
    }
}

fn fx_of(v: &Value) -> Option<Fx> {
    let cell = |k: usize| v[k].as_i64().and_then(|c| i32::try_from(c).ok()).filter(|c| (0..LENGTH).contains(c));
    Some(match v[0].as_str()? {
        "anim" => {
            let kind = ANIMS.iter().copied().find(|k| v[2] == *k)?;
            Fx::Anim { who: actor_of(&v[1])?, kind, cell: cell(3)? }
        }
        "status" => {
            let tone = *TONES.get(v[3].as_u64()? as usize)?;
            Fx::Status { who: actor_of(&v[1])?, text: v[2].as_str()?.chars().take(40).collect(), tone }
        }
        "die" => Fx::Die { who: actor_of(&v[1])?, cell: cell(2)?, stats: v[3].clone() },
        "checked" => Fx::Checked(cell(1)?),
        "leaves" => Fx::Leaves(cell(1)?),
        _ => return None,
    })
}

fn dice_v(e: &DiceEvent) -> Value {
    json!([e.label, e.value, e.lo, e.hi, e.ok])
}

fn dice_of(v: &Value) -> Option<DiceEvent> {
    let int = |k: usize| v[k].as_i64().and_then(|x| i32::try_from(x).ok());
    let (lo, hi) = (int(2)?, int(3)?);
    if !(0..=1000).contains(&lo) || !(lo..=1000).contains(&hi) {
        return None; // un dado de mentira no traba la animación
    }
    Some(DiceEvent { label: v[0].as_str()?.chars().take(24).collect(), value: int(1)?, lo, hi, ok: v[4].as_bool() })
}

/// ¿Puede el héroe `i` pedir esto? (el juego vuelve a validar al actuar, como con el input local)
fn valid(d: &Dungeon, i: usize, a: Action) -> bool {
    let h = &d.heroes[i];
    let has = |s: Slot| h.item(s).is_some();
    h.is_alive()
        && match a {
            Action::Attack(e) => matches!(e, ActorId::Mob(_)),
            Action::Help(j) => j < d.heroes.len() && j != i,
            Action::Use(s, _, t) => has(s) && t.is_none_or(has),
            Action::Throw(s, _) => has(s),
            Action::Cast(k, _) => k < h.spell_ids().len(),
            _ => true,
        }
}

// --- sesiones ---

/// Lado host: `hello` -> se suma un héroe (o vuelve al suyo si se reconecta); `action` -> a SU héroe; `leave` -> su
/// héroe queda descansando (la app lo hace esperar). Tras cada `pump`, si el estado cambió, se le manda a cada uno.
pub struct HostGame {
    host: Host,
    /// id de cliente -> índice de su héroe en `d.heroes`.
    players: BTreeMap<u32, usize>,
    /// id del personaje -> su héroe: si se reconecta en la misma sesión vuelve al mismo (con lo que tenía acá, no con
    /// lo que manda). ponytail: al irse su héroe queda en la partida (descansando)
    by_char: BTreeMap<String, usize>,
    last: String,
    sent: Instant,
    /// false = sala de espera: se suman jugadores pero la partida no corre ni se manda (EMPEZAR la arranca).
    started: bool,
    /// Hay que mandar la lista de la sala (alguien entró o se fue).
    lobby_dirty: bool,
}

/// ¿Espera la partida a que este héroe elija qué hacer? (vivo, sin acción, sin animación pendiente, ni caído ni
/// paralizado). Es lo que muestra la lista de "quién falta".
pub fn needs_input(h: &Hero) -> bool {
    h.is_alive() && !h.animating && h.waiting()
}

impl HostGame {
    pub fn new(name: &str) -> std::io::Result<Self> {
        Ok(Self {
            host: Host::new(name)?,
            players: BTreeMap::new(),
            by_char: BTreeMap::new(),
            last: String::new(),
            sent: Instant::now(),
            started: false,
            lobby_dirty: true,
        })
    }

    /// EMPEZAR: sale de la sala de espera y manda la partida a todos.
    pub fn start(&mut self) {
        self.started = true;
        self.last.clear();
    }

    /// Los de la sala: el host y los que se unieron.
    pub fn names(&self, d: &Dungeon) -> Vec<String> {
        std::iter::once(d.local().ch.name.clone()).chain(self.players.values().map(|&i| d.heroes[i].ch.name.clone())).collect()
    }

    /// ¿Hay un cliente conectado manejando al héroe `j`?
    pub fn controls(&self, j: usize) -> bool {
        self.players.values().any(|&i| i == j)
    }

    fn refuse(msg: &Value, classes: &Map<String, Value>, d: &Dungeon) -> Option<&'static str> {
        if msg["version"] != game::VERSION {
            return Some("version distinta");
        }
        if msg["hash"] != game::registry::content_hash().as_str() {
            return Some("contenido distinto (data/ o mods)");
        }
        let ok = msg["class"].as_str().is_some_and(|c| classes.contains_key(c))
            && abilities(msg).is_some()
            && (msg["hero"].is_null() || save::portable_ok(d, &msg["hero"]));
        (!ok).then_some("personaje invalido")
    }

    pub fn pump(&mut self, d: &mut Dungeon, classes: &Map<String, Value>) {
        while let Ok((cid, msg)) = self.host.inbox.try_recv() {
            match msg["t"].as_str().unwrap_or("") {
                "hello" if !self.players.contains_key(&cid) => {
                    if let Some(why) = Self::refuse(&msg, classes, d) {
                        self.host.send(cid, &json!({"t": "refused", "why": why}));
                        self.host.kick(cid);
                        continue;
                    }
                    let name: String = msg["name"].as_str().unwrap_or("?").chars().take(NAME_MAX).collect();
                    let char_id = msg["char"].as_str().unwrap_or("").to_string();
                    let back = self.by_char.get(&char_id).copied().filter(|&i| i < d.heroes.len() && i != d.hero && !self.controls(i));
                    let i = match back {
                        Some(i) => i,
                        None => {
                            let i = d.add_hero(msg["class"].as_str().unwrap_or(""), abilities(&msg).unwrap_or_default());
                            d.heroes[i].ch.name = name.clone();
                            // personaje portátil: nivel, XP, oro y equipo (ya validado en `refuse`)
                            save::set_portable(d, i, &msg["hero"]);
                            d.depth = d.local().depth; // add_hero entra al piso 1: vuelve el piso del host
                            d.view_changed = true;
                            i
                        }
                    };
                    if !char_id.is_empty() {
                        self.by_char.insert(char_id, i);
                    }
                    self.players.insert(cid, i);
                    d.log(&format!("{name} se une a la partida"));
                    self.host.set_players(1 + self.players.len());
                    self.last.clear(); // que reciba el estado ya
                    self.lobby_dirty = true;
                }
                "action" => {
                    if let Some(&i) = self.players.get(&cid)
                        && let Some(a) = action_of(&msg["a"])
                        && valid(d, i, a)
                    {
                        d.heroes[i].action = Some(a);
                    }
                }
                "leave" => {
                    self.host.kick(cid);
                    if let Some(i) = self.players.remove(&cid) {
                        let name = d.heroes[i].ch.name.clone();
                        d.log(&format!("{name} se va de la partida"));
                        self.host.set_players(1 + self.players.len());
                        self.lobby_dirty = true;
                    }
                }
                _ => {}
            }
        }
        // los efectos y dados de cada piso, a los que están en ese piso (en el acto: no esperan al estado)
        d.record_all = true;
        let (fx, dice) = (std::mem::take(&mut d.net_fx), std::mem::take(&mut d.net_dice));
        if self.started && (!fx.is_empty() || !dice.is_empty()) {
            for (&cid, &i) in &self.players {
                let depth = d.heroes[i].depth;
                let f: Vec<Value> = fx.iter().filter(|e| e.0 == depth).map(|e| fx_v(&e.1)).collect();
                let r: Vec<Value> = dice.iter().filter(|e| e.0 == depth).map(|e| dice_v(&e.1)).collect();
                if !f.is_empty() || !r.is_empty() {
                    self.host.send(cid, &json!({"t": "fx", "fx": f, "dice": r}));
                }
            }
        }
        if !self.started {
            if self.lobby_dirty {
                self.lobby_dirty = false;
                let msg = json!({"t": "lobby", "players": self.names(d)});
                for &cid in self.players.keys() {
                    self.host.send(cid, &msg);
                }
            }
            return;
        }
        if self.players.is_empty() || self.sent.elapsed() < STATE_EVERY {
            return;
        }
        // el estado y quién falta elegir (las acciones no van en el guardado)
        let wait = json!(d.heroes.iter().map(needs_input).collect::<Vec<_>>()).to_string();
        let data = format!("{},\"wait\":{wait}", save::dump(d));
        if data == self.last {
            return;
        }
        self.sent = Instant::now();
        let (save, _) = data.split_once(",\"wait\":").unwrap_or((&data, ""));
        for (&cid, &i) in &self.players {
            let msg = format!("{{\"t\":\"state\",\"you\":{i},\"wait\":{wait},\"save\":{save}}}\n");
            self.host.send_raw(cid, msg.as_bytes());
        }
        self.last = data;
    }
}

/// Las características del hello, validadas (6, entre 1 y 20).
fn abilities(msg: &Value) -> Option<[i32; 6]> {
    let a = msg["abilities"].as_array().filter(|a| a.len() == 6)?;
    let scores: [Option<i32>; 6] = std::array::from_fn(|k| a[k].as_i64().and_then(|v| i32::try_from(v).ok()));
    game::dice::valid_scores(&scores)
}

/// Lado cliente: manda `hello` al conectar; `pump` devuelve la partida armada con el último estado (None si no llegó
/// nada nuevo). `refused` = el motivo si el host lo rechazó; `closed` = se cortó la conexión.
pub struct ClientGame {
    stream: TcpStream,
    inbox: Receiver<Value>,
    pub refused: Option<String>,
    pub closed: bool,
    /// Sala de espera: los nombres que mandó el host (se toma al mostrarlo).
    pub lobby: Option<Vec<String>>,
    /// Efectos y dados del host para el piso de este jugador (la app los pasa a su partida).
    pub fx: Vec<Fx>,
    pub dice: Vec<DiceEvent>,
}

impl ClientGame {
    /// `c` = el personaje (characters.json); `hero` = `save::portable` de su héroe (None = entra con nivel 1).
    pub fn connect(addr: SocketAddr, c: &Value, hero: Option<Value>) -> std::io::Result<Self> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))?;
        stream.set_nodelay(true)?;
        let (tx, inbox) = channel();
        read_loop(stream.try_clone()?, tx, |m| m);
        let mut g = Self { stream, inbox, refused: None, closed: false, lobby: None, fx: Vec::new(), dice: Vec::new() };
        g.send(&json!({"t": "hello", "version": game::VERSION, "hash": game::registry::content_hash(),
                       "name": c["name"], "char": characters::id(c), "class": c["class"],
                       "abilities": characters::abilities(c), "hero": hero}));
        Ok(g)
    }

    fn send(&mut self, msg: &Value) {
        if self.stream.write_all(&line(msg)).is_err() {
            self.closed = true;
        }
    }

    pub fn send_action(&mut self, a: Action) {
        self.send(&json!({"t": "action", "a": action_v(a)}));
    }

    /// La partida del último estado y quién falta elegir (por héroe).
    pub fn pump(&mut self) -> Option<(Dungeon, Vec<bool>)> {
        let mut latest = None;
        while let Ok(m) = self.inbox.try_recv() {
            match m["t"].as_str().unwrap_or("") {
                "state" => latest = Some(m),
                "refused" => self.refused = Some(m["why"].as_str().unwrap_or("rechazado").to_string()),
                "fx" => {
                    let list = |k: &str| m[k].as_array().cloned().unwrap_or_default();
                    self.fx.extend(list("fx").iter().filter_map(fx_of));
                    self.dice.extend(list("dice").iter().filter_map(dice_of));
                }
                "lobby" => {
                    let names = m["players"].as_array().map(|a| a.iter().filter_map(|n| n.as_str().map(str::to_string)).collect());
                    self.lobby = names;
                }
                "leave" => self.closed = true,
                _ => {}
            }
        }
        let mut m = latest?;
        let you = m["you"].as_u64()?;
        m["save"]["local"] = json!(you);
        let wait = m["wait"].as_array().map_or(Vec::new(), |a| a.iter().map(|b| b.as_bool().unwrap_or(false)).collect());
        // ponytail: se rearma la partida entera (regenera los pisos con la seed) en cada estado; sin dados ni efectos
        // del host todavía (el cliente ve el resultado en el log)
        save::load(&m["save"]).map(|d| (d, wait))
    }
}

impl Drop for ClientGame {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acciones_ida_y_vuelta() {
        let all = [
            Action::Move(5),
            Action::Attack(ActorId::Mob(7)),
            Action::Help(1),
            Action::Rest,
            Action::Search,
            Action::Descend(10),
            Action::Ascend(11),
            Action::PickUp(12),
            Action::Open(13),
            Action::Use(Slot::Pack(2), Verb::Read, Some(Slot::Weapon)),
            Action::Use(Slot::Armor, Verb::Unequip, None),
            Action::Cast(1, 40),
            Action::SecondWind,
            Action::Throw(Slot::Pack(0), 99),
        ];
        for a in all {
            assert_eq!(action_of(&action_v(a)), Some(a), "{a:?}");
        }
    }

    #[test]
    fn efectos_ida_y_vuelta() {
        let fx = [
            Fx::Anim { who: ActorId::Mob(3), kind: "attack", cell: 40 },
            Fx::Status { who: ActorId::Hero(1), text: "-5".into(), tone: Tone::Neg },
            Fx::Die { who: ActorId::Mob(9), cell: 41, stats: json!({"name": "rata"}) },
            Fx::Checked(42),
            Fx::Leaves(43),
        ];
        for f in fx {
            let back = fx_of(&fx_v(&f)).unwrap();
            assert_eq!(fx_v(&back), fx_v(&f));
        }
        let e = DiceEvent { label: "d20".into(), value: 17, lo: 1, hi: 20, ok: Some(true) };
        assert_eq!(dice_v(&dice_of(&dice_v(&e)).unwrap()), dice_v(&e));
        assert!(dice_of(&json!(["d20", 1, 5, 2, null])).is_none());
        assert!(fx_of(&json!(["anim", ["m", 1], "rm -rf", 3])).is_none());
    }

    #[test]
    fn acciones_invalidas() {
        for v in [json!(["move", -1]), json!(["move", LENGTH]), json!(["nada"]), json!(["use", "x", 0, null]),
                  json!(["use", 0, 99, null]), json!(["attack", ["h"]]), json!(null), json!("move")] {
            assert_eq!(action_of(&v), None, "{v}");
        }
    }

    /// Un host y un cliente de verdad por loopback: el cliente entra, recibe la partida con su héroe y su acción
    /// llega a SU héroe en el host. (Usa los puertos del juego: falla si hay una partida LAN abierta en esta PC.)
    #[test]
    fn partida_por_lan() {
        let classes = game::registry::load("classes").unwrap();
        let mut d = Dungeon::new_with(1, "pdri:warrior", game::dice::DEFAULT_SCORES);
        let mut host = HostGame::new("prueba").unwrap();
        host.start();
        let c = characters::new_character("Cliente", "pdri:mage", game::dice::DEFAULT_SCORES);
        // personaje portátil: entra con el nivel, el oro y el equipo de su partida
        let solo = Dungeon::new_with(2, "pdri:mage", game::dice::DEFAULT_SCORES);
        let mut p = save::portable(solo.local());
        p["lvl"] = json!(3);
        p["gold"] = json!(50);
        let ht1 = solo.local().ch.ht;
        let mut cl = ClientGame::connect(SocketAddr::from(([127, 0, 0, 1], TCP_PORT)), &c, Some(p)).unwrap();
        let t0 = Instant::now();
        let got = loop {
            host.pump(&mut d, &classes);
            if let Some((cd, wait)) = cl.pump() {
                assert_eq!(wait.len(), cd.heroes.len());
                break cd;
            }
            assert!(t0.elapsed() < Duration::from_secs(5), "no llegó el estado");
            std::thread::sleep(Duration::from_millis(20));
        };
        assert_eq!(d.heroes.len(), 2);
        assert_eq!((got.hero, got.heroes.len()), (1, 2));
        assert_eq!(got.heroes[1].ch.name, "Cliente");
        let h = &d.heroes[1];
        assert_eq!((h.lvl, h.gold, h.backpack.len()), (3, 50, solo.local().backpack.len()));
        assert!(h.ch.ht > ht1 && h.ch.hp == h.ch.ht);
        cl.send_action(Action::Search);
        while d.heroes[1].action != Some(Action::Search) {
            host.pump(&mut d, &classes);
            assert!(t0.elapsed() < Duration::from_secs(5), "no llegó la acción");
            std::thread::sleep(Duration::from_millis(20));
        }
        // una acción a nombre de otro héroe no se acepta
        cl.send_action(Action::Help(7));
        drop(cl);
        while host.controls(1) {
            host.pump(&mut d, &classes);
            assert!(t0.elapsed() < Duration::from_secs(5), "no se fue");
            std::thread::sleep(Duration::from_millis(20));
        }
        // al jugar solo después, su héroe sale de la partida sin dejar rastros en el planificador
        d.remove_hero(1);
        assert_eq!((d.heroes.len(), d.hero), (1, 0));
        let s = &d.floor().scheduler;
        assert!(s.actors.iter().all(|e| !matches!(e.0, ActorId::Hero(1) | ActorId::Regen(1) | ActorId::Hunger(1))));
        assert!(s.contains(ActorId::Hero(0)));
    }

    #[test]
    fn personaje_portatil_invalido() {
        let d = Dungeon::new_with(1, "pdri:warrior", game::dice::DEFAULT_SCORES);
        let ok = save::portable(d.local());
        assert!(save::portable_ok(&d, &ok));
        let bad = |k: &str, v: Value| {
            let mut p = ok.clone();
            p[k] = v;
            !save::portable_ok(&d, &p)
        };
        assert!(bad("lvl", json!(99)));
        assert!(bad("exp", json!(10_000)));
        assert!(bad("gold", json!(-1)));
        assert!(bad("weapon", ok["armor"].clone())); // armadura en la ranura del arma
        assert!(bad("backpack", json!(vec![ok["armor"].clone(); 20]))); // más que la mochila
        assert!(bad("backpack", json!([["pdri:no_existe", 1, 0, false, true, true]])));
        assert!(bad("quick", json!([])));
    }

    #[test]
    fn host_y_cliente_por_tcp() {
        // un mensaje de ida y vuelta por loopback, con el formato de una línea por JSON
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = l.local_addr().unwrap();
        let t = std::thread::spawn(move || {
            let (s, _) = l.accept().unwrap();
            let (tx, rx) = channel();
            read_loop(s.try_clone().unwrap(), tx, |m| m);
            let got = rx.recv_timeout(Duration::from_secs(3)).unwrap();
            let mut w = s;
            w.write_all(&line(&json!({"t": "eco", "a": got["a"]}))).unwrap();
        });
        let mut s = TcpStream::connect(addr).unwrap();
        let (tx, rx) = channel();
        read_loop(s.try_clone().unwrap(), tx, |m| m);
        s.write_all(&line(&json!({"t": "action", "a": action_v(Action::Move(3))}))).unwrap();
        let back = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(action_of(&back["a"]), Some(Action::Move(3)));
        t.join().unwrap();
    }
}
