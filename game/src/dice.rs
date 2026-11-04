//! `game/dice.py`: dados de D&D 5e (notación "NdM+K", d20 con ventaja / desventaja, modificadores).
//! Todas las tiradas reciben el rng de la partida (determinismo para el multijugador).
use crate::rng::Rng;

pub const ABILITIES: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
/// Abreviaturas en español, en el orden de `ABILITIES`.
pub const ABILITY_NAMES: [&str; 6] = ["FUE", "DES", "CON", "INT", "SAB", "CAR"];
/// Array estándar de 5e, en el orden de `ABILITIES` (partida sin personaje, y la de fondo del título).
pub const DEFAULT_SCORES: [i32; 6] = [15, 14, 13, 12, 10, 8];
/// Rango de una característica escrita a mano en la creación de personaje.
pub const MIN_SCORE: i32 = 1;
pub const MAX_SCORE: i32 = 20;

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

/// 4d6 quitando el menor. Devuelve (valor, los 4 dados) para mostrar cuál se descartó.
pub fn roll_ability_score(rng: &mut Rng) -> (i32, [i32; 4]) {
    let dice = [(); 4].map(|_| rng.int_range_incl(1, 6));
    (dice.iter().sum::<i32>() - dice.iter().min().unwrap(), dice)
}

/// Creación de personaje: tantas tiradas como características + 1 (7). Se asignan 6 a voluntad y sobra 1.
pub fn roll_ability_pool(rng: &mut Rng) -> Vec<(i32, [i32; 4])> {
    (0..=ABILITIES.len()).map(|_| roll_ability_score(rng)).collect()
}

/// Las 6 características puestas y dentro de rango.
pub fn valid_scores(scores: &[Option<i32>; 6]) -> Option<[i32; 6]> {
    let mut out = [0; 6];
    for (o, s) in out.iter_mut().zip(scores) {
        *o = s.filter(|v| (MIN_SCORE..=MAX_SCORE).contains(v))?;
    }
    Some(out)
}
