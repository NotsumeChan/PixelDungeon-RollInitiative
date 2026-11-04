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
use game::dungeon::Dungeon;
use game::geom::LENGTH;
use game::hero::{Action, Slot, Verb};
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
    /// id del personaje -> su héroe: si se reconecta en la misma sesión vuelve al mismo.
    /// ponytail: el personaje entra con nivel 1 y el equipo inicial; el personaje portátil (paso 5) y sacar al héroe
    /// de la partida al irse llegan después
    by_char: BTreeMap<String, usize>,
    last: String,
    sent: Instant,
}

impl HostGame {
    pub fn new(name: &str) -> std::io::Result<Self> {
        Ok(Self { host: Host::new(name)?, players: BTreeMap::new(), by_char: BTreeMap::new(), last: String::new(), sent: Instant::now() })
    }

    /// ¿Hay un cliente conectado manejando al héroe `j`?
    pub fn controls(&self, j: usize) -> bool {
        self.players.values().any(|&i| i == j)
    }

    fn refuse(msg: &Value, classes: &Map<String, Value>) -> Option<&'static str> {
        if msg["version"] != game::VERSION {
            return Some("version distinta");
        }
        if msg["hash"] != game::registry::content_hash().as_str() {
            return Some("contenido distinto (data/ o mods)");
        }
        let ok = msg["class"].as_str().is_some_and(|c| classes.contains_key(c)) && abilities(msg).is_some();
        (!ok).then_some("personaje invalido")
    }

    pub fn pump(&mut self, d: &mut Dungeon, classes: &Map<String, Value>) {
        while let Ok((cid, msg)) = self.host.inbox.try_recv() {
            match msg["t"].as_str().unwrap_or("") {
                "hello" if !self.players.contains_key(&cid) => {
                    if let Some(why) = Self::refuse(&msg, classes) {
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
                    }
                }
                _ => {}
            }
        }
        if self.players.is_empty() || self.sent.elapsed() < STATE_EVERY {
            return;
        }
        let data = save::dump(d).to_string();
        if data == self.last {
            return;
        }
        self.sent = Instant::now();
        for (&cid, &i) in &self.players {
            let msg = format!("{{\"t\":\"state\",\"you\":{i},\"save\":{data}}}\n");
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
}

impl ClientGame {
    /// `c` = el personaje (characters.json).
    pub fn connect(addr: SocketAddr, c: &Value) -> std::io::Result<Self> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))?;
        stream.set_nodelay(true)?;
        let (tx, inbox) = channel();
        read_loop(stream.try_clone()?, tx, |m| m);
        let mut g = Self { stream, inbox, refused: None, closed: false };
        g.send(&json!({"t": "hello", "version": game::VERSION, "hash": game::registry::content_hash(),
                       "name": c["name"], "char": characters::id(c), "class": c["class"],
                       "abilities": characters::abilities(c)}));
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

    pub fn pump(&mut self) -> Option<Dungeon> {
        let mut latest = None;
        while let Ok(m) = self.inbox.try_recv() {
            match m["t"].as_str().unwrap_or("") {
                "state" => latest = Some(m),
                "refused" => self.refused = Some(m["why"].as_str().unwrap_or("rechazado").to_string()),
                "leave" => self.closed = true,
                _ => {}
            }
        }
        let mut m = latest?;
        let you = m["you"].as_u64()?;
        m["save"]["local"] = json!(you);
        // ponytail: se rearma la partida entera (regenera los pisos con la seed) en cada estado; sin dados ni efectos
        // del host todavía (el cliente ve el resultado en el log)
        save::load(&m["save"])
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
        let c = characters::new_character("Cliente", "pdri:mage", game::dice::DEFAULT_SCORES);
        let mut cl = ClientGame::connect(SocketAddr::from(([127, 0, 0, 1], TCP_PORT)), &c).unwrap();
        let t0 = Instant::now();
        let got = loop {
            host.pump(&mut d, &classes);
            if let Some(cd) = cl.pump() {
                break cd;
            }
            assert!(t0.elapsed() < Duration::from_secs(5), "no llegó el estado");
            std::thread::sleep(Duration::from_millis(20));
        };
        assert_eq!(d.heroes.len(), 2);
        assert_eq!((got.hero, got.heroes.len()), (1, 2));
        assert_eq!(got.heroes[1].ch.name, "Cliente");
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
