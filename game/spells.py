"""Conjuros de D&D 5e (el original no tiene un sistema así; la línea de tiro es mechanics/Ballistica.java).
Son DATOS (SPELLS, IDs "namespace:nombre"), como MOBS y BUFFS:
- un mod de datos agrega conjuros combinando los tipos que ya existen (KINDS, por nombre); nunca apunta a código.
- un mod con código puede registrar tipos nuevos en KINDS: fn(héroe, conjuro, objetivo, celda, niveles extra)."""
from . import dice, ballistica, registry
from .char import Char, distance, adjacent
from .lang import t
from .mobs import SLEEPING

# Campos de SPELLS:
#   name, desc  str o dict por idioma
#   level       0 = truco (sin espacio; los dados se multiplican en los niveles 5, 11 y 17); 1+ = gasta un espacio de ese
#               nivel o mayor (el menor que haya)
#   kind        "attack": tirada de ataque de conjuro contra la CA (20 crítico, 1 falla); damage, buff opcional al acertar
#               "auto": darts dardos que aciertan solos, damage cada uno
#               "sleep": reserva de pool PV; duerme a los mobs de menos vida a radius casillas de la celda
#               "heal": cura damage + modificador al lanzador
#               "mark": pone el estado buff al objetivo (quita la marca anterior)
#   target      "enemy" (criatura visible con línea de tiro) | "cell" (celda visible) | "self" | "touch" (uno mismo o un
#               aliado al lado)
#   upcast      por cada nivel de espacio por encima del del conjuro: {"darts": n} | {"pool": "2d8"} | {"damage": "2d8"}
#   bonus       True = acción adicional (no gasta tiempo)
#   fx          proyectil o efecto del render: "fire" | "frost" | "missile" | "sleep" | "mark"
SPELLS: dict[str, dict] = registry.load("spells")  # data/spells.json

# Espacios de conjuro por nivel de personaje (5e): lanzador completo (Mago) y medio lanzador (Explorador, reglas 2024)
SLOT_TABLES: dict[str, list[list[int]]] = {
    "full": [[2], [3], [4, 2], [4, 3], [4, 3, 2], [4, 3, 3], [4, 3, 3, 1], [4, 3, 3, 2], [4, 3, 3, 3, 1], [4, 3, 3, 3, 2]],
    "half": [[2], [2], [3], [3], [4, 2], [4, 2], [4, 3], [4, 3], [4, 3, 2], [4, 3, 2]],
}
# Rasgos que dan lanzamientos gratis (sin espacio): rasgo -> (conjuro, usos según el nivel)
FREE_CASTS: dict[str, tuple] = {"pddnd:favored_enemy": ("pddnd:hunters_mark", lambda lvl: 2 + (lvl - 1) // 4)}
TIME_TO_CAST = 1.0


# --- números del lanzador ---
def spellcasting(hero) -> dict | None:
    return hero.cls.get("spellcasting")


def spell_bonus(hero) -> int:
    """Bono de ataque de conjuro: competencia + modificador de la característica de lanzamiento."""
    return hero.proficiency() + hero.mod(spellcasting(hero)["ability"])


def spell_dc(hero) -> int:
    return 8 + spell_bonus(hero)


def max_slots(hero, lvl: int | None = None) -> list[int]:
    sc = spellcasting(hero)
    if not sc:
        return []
    table = SLOT_TABLES[sc["slots"]]
    return list(table[min(lvl or hero.lvl, len(table)) - 1])


def max_free(hero, lvl: int | None = None) -> dict[str, int]:
    return {sid: uses(lvl or hero.lvl) for trait, (sid, uses) in FREE_CASTS.items() if hero.has_trait(trait)}


def level_up(hero) -> None:
    """Al subir de nivel se suman los espacios y usos gratis nuevos (los gastados siguen gastados)."""
    old, new = max_slots(hero, hero.lvl - 1), max_slots(hero)
    hero.slots += [0] * (len(new) - len(hero.slots))
    for i, n in enumerate(new):
        hero.slots[i] += n - (old[i] if i < len(old) else 0)
    old_free = max_free(hero, hero.lvl - 1)
    for sid, n in max_free(hero).items():
        hero.free_casts[sid] = hero.free_casts.get(sid, 0) + n - old_free.get(sid, 0)


def _slot_for(hero, level: int) -> int:
    """El menor nivel de espacio disponible >= level (0 si no hay)."""
    return next((i + 1 for i in range(level - 1, len(hero.slots)) if hero.slots[i] > 0), 0)


def can_cast(hero, sid: str) -> bool:
    sc, s = spellcasting(hero), SPELLS[sid]
    if not sc or sid not in sc["spells"] or not hero.proficient_armor(hero.armor):
        return False
    return s["level"] == 0 or hero.free_casts.get(sid, 0) > 0 or _slot_for(hero, s["level"]) > 0


def _cantrip_mult(hero) -> int:
    return 1 + (hero.lvl >= 5) + (hero.lvl >= 11) + (hero.lvl >= 17)


def _more(notation: str, extra: str, times: int) -> str:
    """'5d8' + 2 veces '2d8' -> '9d8' (mismo dado)."""
    n, sides, b = dice.parse(notation)
    if times:
        n += dice.parse(extra)[0] * times
    return f"{n}d{sides}" + (f"{b:+d}" if b else "")


# --- lanzar ---
def cast(hero, sid: str, cell: int) -> None:
    """Valida y lanza. Si no se puede, avisa en el log y no gasta tiempo."""
    d, s = hero.dungeon, SPELLS[sid]
    sc = spellcasting(hero)
    if not sc or sid not in sc["spells"]:
        return
    if not hero.proficient_armor(hero.armor):
        d.log("No puedes lanzar conjuros con una armadura sin competencia")
        return
    target, cover = None, 0
    if s["target"] == "enemy":
        target = d.find_char(cell)
        if target is None or target is hero or not hero.fov[cell]:
            d.log("Necesitas un enemigo a la vista")
            return
        stop, cover = d.shot(hero.pos, cell)  # pasa por los aliados; con uno en medio, media cobertura
        if stop != cell:
            d.log("No hay línea de tiro")
            return
    elif s["target"] == "cell" and not hero.fov[cell]:
        d.log("Necesitas una casilla a la vista")
        return
    elif s["target"] == "touch":  # uno mismo o un aliado al lado (también caído)
        target = d.find_char(cell)
        if target is not hero and not (target in d.heroes and adjacent(hero.pos, cell)):
            d.log("Tócate a ti o a un aliado que esté al lado")
            return
    extra = 0
    if s["level"] > 0:
        if hero.free_casts.get(sid, 0) > 0:
            hero.free_casts[sid] -= 1
        else:
            slot = _slot_for(hero, s["level"])
            if not slot:
                d.log("No te quedan espacios de conjuro")
                return
            hero.slots[slot - 1] -= 1
            extra = slot - s["level"]
    if s["target"] in ("self", "touch"):
        d.effect("anim", hero, "operate", cell if s["target"] == "touch" else hero.pos)
    else:
        d.effect("anim", hero, "attack", cell)  # HeroSprite: zap = la animación de ataque
        if s.get("fx"):
            d.effect("bolt", hero.pos, cell, s["fx"])
    hero.cover = cover  # sólo mientras se resuelve el conjuro
    try:
        KINDS[s["kind"]](hero, s, target, cell, extra)
    finally:
        hero.cover = 0
    if not s.get("bonus"):
        hero.spend(TIME_TO_CAST)


def _attack(hero, s, target, cell, extra) -> None:
    d = hero.dungeon
    name = t(s["name"])
    nat, text = dice.d20(d.rng, Char.advantage(hero, target))  # sólo lo del objetivo (desprevenido, paralizado)
    bonus, ac = spell_bonus(hero) - hero.cover, target.armor_class()
    hit = nat == 20 or (nat != 1 and nat + bonus >= ac)
    d.roll_event("d20", nat, 1, 20)
    line = f"{name}: d20 {text}{bonus:+d}={nat + bonus} vs CA {ac}"
    if not hit:
        d.defer((hero, target, False, 0, line + ", falla", f"{name} falla"))
        return
    n, sides, b = dice.parse(s["damage"])
    n *= _cantrip_mult(hero) if s["level"] == 0 else 1
    total, dtext, rolled, _ = dice.roll(d.rng, f"{n}d{sides}" + (f"{b:+d}" if b else ""), nat == 20)
    d.roll_event(f"{len(rolled)}d{sides}", sum(rolled), len(rolled), len(rolled) * sides)
    crit = " (critico)" if nat == 20 else ""
    d.defer((hero, target, True, total, f"{line}, {dtext}={total}", f"{name} golpea a {target.name}: {total}{crit}"))
    if s.get("buff"):
        buff_id, duration = s["buff"]
        d.defer(lambda: target.is_alive() and target.add_buff(buff_id, duration))


def _auto(hero, s, target, cell, extra) -> None:
    d = hero.dungeon
    name = t(s["name"])
    darts = s["darts"] + extra * s.get("upcast", {}).get("darts", 0)
    rolls = [dice.roll(d.rng, s["damage"])[0] for _ in range(darts)]
    total = sum(rolls)
    n, sides, b = dice.parse(s["damage"])
    d.roll_event(f"{darts}x{s['damage']}", total, darts * (n + b), darts * (n * sides + b))
    d.defer((hero, target, True, total, f"{name}: {darts} dardos ({'+'.join(map(str, rolls))})={total}",
                      f"{name}: {total} a {target.name}"))


def _sleep(hero, s, target, cell, extra) -> None:
    d = hero.dungeon
    name = t(s["name"])
    pool_dice = _more(s["pool"], s.get("upcast", {}).get("pool", s["pool"]), extra)
    pool, _, rolled, sides = dice.roll(d.rng, pool_dice)
    d.roll_event(pool_dice, pool, len(rolled), len(rolled) * sides)
    # de menor a mayor vida; en empate, el orden de la lista (determinista)
    victims = sorted((m for m in d.mobs if m.is_alive() and distance(m.pos, cell) <= s["radius"]),
                     key=lambda m: (m.HP, d.mobs.index(m)))

    def apply() -> None:
        left, slept = pool, []
        for m in victims:
            if not m.is_alive():
                continue
            if m.HP > left:
                break
            left -= m.HP
            m.state, m.enemy_seen, m.alerted = SLEEPING, False, False
            d.status(m, "duerme", "neutral")
            slept.append(m.name)
        d.log(f"{name}: " + (", ".join(slept) + " se duermen" if slept else "nadie se duerme"))

    d.defer(apply)


def _heal(hero, s, target, cell, extra) -> None:
    d = hero.dungeon
    name = t(s["name"])
    notation = _more(s["damage"], s.get("upcast", {}).get("damage", s["damage"]), extra)
    roll, _, rolled, sides = dice.roll(d.rng, notation)
    m = hero.mod(spellcasting(hero)["ability"])
    d.roll_event(notation, sum(rolled), len(rolled), len(rolled) * sides)

    who = target or hero  # "touch": uno mismo o un aliado al lado

    def apply() -> None:
        if not who.is_alive():
            return
        heal = who.heal(roll + m)
        d.status(who, f"+{heal}", "pos")
        d.log(f"{name}: +{heal} PV" + (f" a {who.name}" if who is not hero else ""))

    d.defer(apply)


def _mark(hero, s, target, cell, extra) -> None:
    buff_id, duration = s["buff"]
    old = hero.marked
    if old is not None and old is not target and old.buff(buff_id):
        old.buff(buff_id).detach()
    target.add_buff(buff_id, duration)
    hero.marked = target
    hero.dungeon.log(f"{t(s['name'])}: marcas a {target.name}")


KINDS: dict = {"attack": _attack, "auto": _auto, "sleep": _sleep, "heal": _heal, "mark": _mark}
