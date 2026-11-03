"""Dados de D&D 5e: notación "NdM+K", d20 con ventaja/desventaja, modificadores y tirada de características.
Todas las tiradas reciben el rng explícito (determinismo para el P2P)."""
import re

ABILITIES = ["STR", "DEX", "CON", "INT", "WIS", "CHA"]
ABILITY_NAMES = {"STR": "FUE", "DEX": "DES", "CON": "CON", "INT": "INT", "WIS": "SAB", "CHA": "CAR"}
MIN_SCORE, MAX_SCORE = 1, 20

_NOTATION = re.compile(r"^\s*(\d*)d(\d+)\s*([+-]\s*\d+)?\s*$")


def parse(notation: str) -> tuple[int, int, int]:
    """'2d6+3' -> (2, 6, 3). '1d8' -> (1, 8, 0)."""
    m = _NOTATION.match(notation)
    if not m:
        raise ValueError(f"notación de dados inválida: {notation}")
    n, sides, bonus = m.groups()
    return int(n or 1), int(sides), int(bonus.replace(" ", "")) if bonus else 0


def roll_dice(rng, n: int, sides: int) -> list[int]:
    return [rng.IntRange(1, sides) for _ in range(n)]


def roll(rng, notation: str, crit: bool = False) -> tuple[int, str, list[int], int]:
    """Tira la notación. crit = dados dobles (golpe crítico). Devuelve (total, texto para el log, dados, caras)."""
    n, sides, bonus = parse(notation)
    dice = roll_dice(rng, n * (2 if crit else 1), sides)
    total = sum(dice) + bonus
    text = f"{n * (2 if crit else 1)}d{sides}({'+'.join(map(str, dice))})"
    if bonus:
        text += f"{bonus:+d}"
    return total, text, dice, sides


def d20(rng, advantage: int = 0) -> tuple[int, str]:
    """advantage: 1 = ventaja (mejor de 2), -1 = desventaja (peor de 2), 0 = normal."""
    if advantage == 0:
        r = rng.IntRange(1, 20)
        return r, str(r)
    a, b = rng.IntRange(1, 20), rng.IntRange(1, 20)
    r = max(a, b) if advantage > 0 else min(a, b)
    return r, f"{r}[{a}/{b}{' V' if advantage > 0 else ' D'}]"


def modifier(score: int) -> int:
    """(característica - 10) / 2 redondeado hacia abajo: 8 -> -1, 10 -> 0, 15 -> +2."""
    return (score - 10) // 2


def proficiency(level: int) -> int:
    """Bonificador de competencia: +2 en niveles 1-4, +3 en 5-8, ..."""
    return 2 + (level - 1) // 4


def roll_ability_score(rng) -> tuple[int, list[int]]:
    """4d6 quitando el menor. Devuelve (valor, los 4 dados) para mostrar cuál se descartó."""
    dice = roll_dice(rng, 4, 6)
    return sum(dice) - min(dice), dice


def roll_ability_pool(rng) -> list[tuple[int, list[int]]]:
    """Creación de personaje: tantas tiradas como características + 1 (7). Se asignan 6 a voluntad y sobra 1."""
    return [roll_ability_score(rng) for _ in range(len(ABILITIES) + 1)]


def valid_scores(scores: dict) -> bool:
    return all(isinstance(scores.get(a), int) and MIN_SCORE <= scores[a] <= MAX_SCORE for a in ABILITIES)


DEFAULT_SCORES = {"STR": 15, "DEX": 14, "CON": 13, "INT": 12, "WIS": 10, "CHA": 8}  # array estándar de 5e
