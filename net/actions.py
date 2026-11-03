"""Acciones del jugador como datos (fase 5, paso 4). El cliente manda lo que tocó en la UI y el host lo aplica a SU
héroe. Nada de referencias a objetos: celdas e índices de ítems (en hero.all_items()). Es una frontera de
confianza: lo que no valida se ignora (y el juego vuelve a validar al actuar, como con el input local)."""
from game.geom import LENGTH
from game.spells import SPELLS

SIMPLE = ("rest", "search", "second_wind")
VERBS = ("equip", "unequip", "eat", "drink", "read", "throw", "drop")  # gameUi.Verbs


# --- cliente: armar el mensaje ---
def handle(cell: int, stairs: bool = False) -> dict:
    return {"t": "action", "a": "handle", "cell": cell, "stairs": stairs}


def simple(kind: str) -> dict:
    return {"t": "action", "a": kind}


def use(hero, item, verb: str, target=None) -> dict:
    owned = hero.all_items()
    return {"t": "action", "a": "use", "item": owned.index(item), "verb": verb,
            "target": owned.index(target) if target is not None else None}


def throw(hero, item, cell: int) -> dict:
    return {"t": "action", "a": "throw", "item": hero.all_items().index(item), "cell": cell}


def cast(sid: str, cell: int) -> dict:
    return {"t": "action", "a": "cast", "spell": sid, "cell": cell}


# --- host: validar y aplicar ---
def _cell(msg: dict) -> int | None:
    c = msg.get("cell")
    return c if type(c) is int and 0 <= c < LENGTH else None


def _item(hero, i):
    owned = hero.all_items()
    return owned[i] if type(i) is int and 0 <= i < len(owned) else None


def apply(hero, msg: dict) -> bool:
    """Aplica la acción al héroe del cliente. False = inválida (se ignora)."""
    if not hero.is_alive():
        return False
    a = msg.get("a")
    if a == "handle":
        cell = _cell(msg)
        if cell is None:
            return False
        hero.handle(cell, stairs=msg.get("stairs") is True)
    elif a in SIMPLE:
        if a == "second_wind" and not hero.has_trait("pddnd:second_wind"):
            return False
        hero.action = (a,)
    elif a == "use":
        item, verb = _item(hero, msg.get("item")), msg.get("verb")
        if item is None or verb not in VERBS:
            return False
        target = None if msg.get("target") is None else _item(hero, msg.get("target"))
        if msg.get("target") is not None and target is None:
            return False
        hero.use(item, verb, target)
    elif a == "throw":
        item, cell = _item(hero, msg.get("item")), _cell(msg)
        if item is None or cell is None:
            return False
        hero.action = ("throw", item, cell)
    elif a == "cast":
        sid, cell = msg.get("spell"), _cell(msg)
        if sid not in SPELLS or cell is None:
            return False
        hero.action = ("cast", sid, cell)
    else:
        return False
    return True
