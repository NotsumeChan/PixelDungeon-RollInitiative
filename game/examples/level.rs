//! Volcado ASCII de un piso, misma salida que el prototipo (`python -m game.level <seed> <piso>`):
//!     cargo run -q -p game --example level -- <seed> <piso>
use game::level::{to_ascii, Level};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: i64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let depth: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let l = Level::new(seed, depth);
    println!(
        "seed={seed} depth={depth} feeling={} salas={} puertas secretas={}",
        l.feeling.name(),
        l.rooms.len(),
        l.secret_doors
    );
    println!("{}", to_ascii(&l));
    println!("# pared  % pared deco  . suelo  , suelo especial  \" pasto  & pasto alto  ~ agua  (espacio) abismo");
    println!("+ puerta  S puerta secreta  ^ trampa  < entrada  > salida  ? cartel  : brasas  B estante  P pedestal");
}
