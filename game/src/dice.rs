//! `game/dice.py`: dados de D&D 5e (notación "NdM+K", d20 con ventaja / desventaja, modificadores).
//! Todas las tiradas reciben el rng de la partida (determinismo para el multijugador).
use crate::rng::Rng;

pub const ABILITIES: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
/// Array estándar de 5e, en el orden de `ABILITIES`. ponytail: hasta la creación de personaje (R5)
pub const DEFAULT_SCORES: [i32; 6] = [15, 14, 13, 12, 10, 8];

/// "2d6+3" -> (2, 6, 3). "1d8" -> (1, 8, 0). "d4" -> (1, 4, 0).
pub fn parse(notation: &str) -> Option<(i32, i32, i32)> {
    let s: String = notation.chars().filter(|c| !c.is_whitespace()).collect();
    let (n, rest) = s.split_once('d')?;
    let n = if n.is_empty() { 1 } else { n.parse().ok()? };
    let cut = rest.find(['+', '-']).unwrap_or(rest.len());
    let sides = rest[..cut].parse().ok()?;
    let bonus = if cut < rest.len() { rest[cut..].parse().ok()? } else { 0 };
    Some((n, sides, bonus))
}

/// Tira la notación. crit = dados dobles (golpe crítico). Devuelve (total, texto para el log, dados, caras).
pub fn roll(rng: &mut Rng, notation: &str, crit: bool) -> (i32, String, Vec<i32>, i32) {
    let (n, sides, bonus) = parse(notation).unwrap_or_else(|| panic!("notación de dados inválida: {notation}"));
    let n = n * if crit { 2 } else { 1 };
    let dice: Vec<i32> = (0..n).map(|_| rng.int_range_incl(1, sides)).collect();
    let list: Vec<String> = dice.iter().map(i32::to_string).collect();
    let mut text = format!("{n}d{sides}({})", list.join("+"));
    if bonus != 0 {
        text += &format!("{bonus:+}");
    }
    (dice.iter().sum::<i32>() + bonus, text, dice, sides)
}

/// advantage: 1 = ventaja (mejor de 2), -1 = desventaja (peor de 2), 0 = normal. Devuelve (dado, texto).
pub fn d20(rng: &mut Rng, advantage: i32) -> (i32, String) {
    if advantage == 0 {
        let r = rng.int_range_incl(1, 20);
        return (r, r.to_string());
    }
    let (a, b) = (rng.int_range_incl(1, 20), rng.int_range_incl(1, 20));
    let r = if advantage > 0 { a.max(b) } else { a.min(b) };
    (r, format!("{r}[{a}/{b}{}]", if advantage > 0 { " V" } else { " D" }))
}

/// (característica - 10) / 2 redondeado hacia abajo: 8 -> -1, 10 -> 0, 15 -> +2.
pub fn modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

/// Bonificador de competencia: +2 en niveles 1-4, +3 en 5-8, ...
pub fn proficiency(level: i32) -> i32 {
    2 + (level - 1).div_euclid(4)
}
