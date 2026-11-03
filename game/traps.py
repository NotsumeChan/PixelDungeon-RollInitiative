"""levels/traps/*.java + Level.press / mobPress: las 8 trampas de las Alcantarillas, con salvaciones de D&D 5e.
Los efectos aplican estados de game/buffs.py por ID."""
from . import terrain as T
from .char import distance
from .dice import ABILITY_NAMES
from .geom import NEIGHBOURS8
from .mobs import Mob, pick_mob, WANDERING
# ponytail: el gas toxico, el paralizante y el fuego no forman nubes que se expanden (Blob): afectan una vez al
# area 3x3 alrededor de la trampa. Agregar Blob cuando lleguen las pociones de gas.

SUMMON_DELAY = 2.0


def trap_dc(depth: int) -> int:
    """CD de las salvaciones contra trampas: 11 en los pisos 2-3, 12 en el 4 (trampas "molestas" de la DMG)."""
    return 10 + depth // 2


class Source:
    """Origen de daño que no es un personaje (para el log y "Has muerto (...)")."""

    def __init__(self, name: str) -> None:
        self.name = name


LIGHTNING = Source("trampa electrica")


def _area(d, cell: int) -> list:
    return [c for c in d.chars() if c.is_alive() and distance(c.pos, cell) <= 1]


def _save(d, ch, ability: str, effect, half: bool = False) -> None:
    """Tira la salvacion ya (el orden del rng no depende del render) y aplica effect(superada) cuando la animacion
    del dado la muestra (dungeon.pending). effect no debe usar el rng."""
    ok, detail = ch.saving_throw(ability, trap_dc(d.depth), "trampa")
    seen = d.sees(ch.pos)

    def apply() -> None:
        if not ch.is_alive():
            return
        if seen:
            msg = f"{ch.name} {'supera' if ok else 'falla'} la salvacion de {ABILITY_NAMES[ability]}"
            if ok and half:  # half = trampa con "mitad si supera" (regla de la partida "save_half")
                msg += ": la mitad del efecto" if d.rules["save_half"] else ": sin efecto"
            d.log(f"{msg} ({detail})" if d.verbose_rolls else msg)
            d.status(ch, f"{ABILITY_NAMES[ability]} {'supera' if ok else 'falla'}", "pos" if ok else "neg")
        effect(ok)

    if seen:
        d.pending.append(apply)
    else:
        apply()


def _say(d, cell: int, msg: str) -> None:
    if d.level.visible[cell]:
        d.log(msg)


def toxic(d, cell: int, ch) -> None:
    """ToxicTrap: CON; falla = envenenado 4 + piso turnos, supera = la mitad."""
    _say(d, cell, "Una nube de gas toxico brota de la trampa")
    n = 4 + d.depth
    for c in _area(d, cell):
        _save(d, c, "CON", lambda ok, c=c: (not ok or d.rules["save_half"]) and c.add_buff("pddnd:poison", n // 2 if ok else n),
              half=True)


def fire(d, cell: int, ch) -> None:
    """FireTrap: DES; falla = en llamas."""
    _say(d, cell, "La trampa estalla en llamas")
    for c in _area(d, cell):
        def effect(ok, c=c):
            if not ok:
                c.add_buff("pddnd:burning")
        _save(d, c, "DEX", effect)


def paralytic(d, cell: int, ch) -> None:
    """ParalyticTrap: CON; falla = paralizado (repite la salvacion cada turno para librarse)."""
    _say(d, cell, "Una nube de gas paralizante brota de la trampa")
    dc = trap_dc(d.depth)
    for c in _area(d, cell):
        def effect(ok, c=c):
            if not ok:
                c.add_buff("pddnd:paralysis", dc=dc)
        _save(d, c, "CON", effect)


def poison(d, cell: int, ch) -> None:
    """PoisonTrap: dardo; CON; falla = envenenado 4 + piso/2 turnos."""
    if ch is None:
        return
    def effect(ok):
        if not ok:
            ch.add_buff("pddnd:poison", 4 + d.depth // 2)
    _say(d, cell, "Un dardo envenenado sale disparado")
    _save(d, ch, "CON", effect)


def alarm(d, cell: int, ch) -> None:
    """AlarmTrap: sin salvacion; todos los mobs del piso van hacia la trampa."""
    for m in d.mobs:
        if m is not ch:
            m.beckon(cell)
    _say(d, cell, "La trampa suelta un chillido que resuena por toda la mazmorra")


def lightning(d, cell: int, ch) -> None:
    """LightningTrap: entre 1/3 y 2/3 de la vida actual; DES, supera = mitad de daño (5e)."""
    if ch is None:
        return
    dmg = max(1, d.rng.IntRange(ch.HP // 3, 2 * ch.HP // 3))
    _say(d, cell, "Una descarga electrica sale de la trampa")
    _save(d, ch, "DEX", lambda ok: ch.damage(dmg if not ok else dmg // 2 if d.rules["save_half"] else 0, LIGHTNING),
          half=True)


def gripping(d, cell: int, ch) -> None:
    """GrippingTrap: DES; falla = sangrando (piso + 3) y lisiado."""
    if ch is None:
        return
    def effect(ok):
        if not ok:
            ch.add_buff("pddnd:bleeding", level=d.depth + 3)
            ch.add_buff("pddnd:cripple")
    _say(d, cell, "Unas garras de metal se cierran sobre la trampa")
    _save(d, ch, "DEX", effect)


def summoning(d, cell: int, ch) -> None:
    """SummoningTrap: sin salvacion; 1 a 3 mobs del piso aparecen alrededor, despiertos."""
    rng, level = d.rng, d.level
    n = 1
    if rng.Int(2) == 0:
        n += 1
        if rng.Int(2) == 0:
            n += 1
    cands = [p for p in (cell + o for o in NEIGHBOURS8)
             if d.find_char(p) is None and (level.passable[p] or level.avoid[p])]
    while n and cands:
        p = cands.pop(rng.Int(len(cands)))
        mob = Mob(d, pick_mob(rng, d.depth), p, WANDERING)
        d.add_mob(mob, SUMMON_DELAY)
        d.effect("appear", mob)
        n -= 1
    _say(d, cell, "Aparecen monstruos de la nada")


TRAPS: dict = {T.TOXIC_TRAP: toxic, T.FIRE_TRAP: fire, T.PARALYTIC_TRAP: paralytic, T.POISON_TRAP: poison,
               T.ALARM_TRAP: alarm, T.LIGHTNING_TRAP: lightning, T.GRIPPING_TRAP: gripping,
               T.SUMMONING_TRAP: summoning}


def press(d, cell: int, ch) -> None:
    """Level.press (heroe) / mobPress (mobs): los mobs solo activan trampas visibles. La trampa queda inactiva."""
    level = d.level
    terr = level.map[cell]
    trap = T.discover(terr)
    if trap not in TRAPS:
        return
    hero = ch in d.heroes
    if terr != trap:
        if not hero:
            return
        d.log("Una placa oculta hace click")
    level.set(cell, T.INACTIVE_TRAP)
    if hero:
        ch.interrupt()
    d.effect("trap", trap, cell)  # el render dibuja el efecto (gas, llamas, rayo...)
    TRAPS[trap](d, cell, ch)
