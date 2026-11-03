"""Guardado de partidas en JSON (0.8): una por personaje, en <datos del usuario>/saves/<id>.json.
Sin pickle: un archivo de guardado nunca ejecuta código (las partidas del host se compartirán en red).
Los niveles no se guardan enteros: se regeneran con la seed y se pisa sólo lo que cambia (terreno, lo visto,
montones), más los mobs y el planificador de cada piso visitado. Antes de guardar se aplica lo que esperaba a la
animación del dado (dungeon.pending), así el archivo queda entre turnos."""
import json
from datetime import datetime

from .actor import Scheduler
from .buffs import Buff
from .characters import data_dir
from .dungeon import Dungeon, Respawner
from .hero import Hero
from .items import Item, Heap
from .level import SewerLevel
from .mobs import Mob

VERSION = 2  # subirla si cambia el formato (y migrar en _upgrade); uno que no se puede leer se aparta (no se borra)


def _path(char_id: str):
    folder = data_dir() / "saves"
    folder.mkdir(exist_ok=True)
    return folder / f"{char_id}.json"


def _item(i: Item) -> list:
    return [i.id, i.quantity, i.level, i.cursed, i.level_known, i.cursed_known]


def _unitem(v: list) -> Item:
    i = Item(v[0], v[1])
    i.level, i.cursed, i.level_known, i.cursed_known = v[2:6]
    return i


def _floor(d, level, mobs, sched) -> dict:
    """Actores en orden (en empates gana el primero). Los de un héroe llevan su índice en d.heroes; el dueño de un
    estado es ["hero", i] o el índice del mob."""
    hs = d.heroes
    ticks = {id(t): (kind, i) for i, h in enumerate(hs) for kind, t in zip(("regen", "hunger"), h.ticks)}
    actors = []
    for a in sched.actors:
        if a in hs:
            actors.append(["hero", hs.index(a), a.time])
        elif isinstance(a, Mob):
            if a in mobs:
                actors.append(["mob", mobs.index(a), a.time])
        elif id(a) in ticks:
            actors.append([*ticks[id(a)], a.time])
        elif isinstance(a, Respawner):
            actors.append(["respawner", a.time])
        elif isinstance(a, Buff):
            owner = ["hero", hs.index(a.target)] if a.target in hs else mobs.index(a.target) if a.target in mobs else None
            if owner is not None:
                actors.append(["buff", owner, a.id, a.left, a.level, a.dc, a.time])
    return {"now": sched.now, "map": level.map, "visited": level.visited, "mapped": level.mapped,
            "heaps": [[h.pos, h.type, [_item(i) for i in h.items]] for h in level.heaps.values()],
            "mobs": [[m.key, m.pos, m.HP, m.state, m.target, m.enemy_seen, m.alerted] for m in mobs],
            "actors": actors}


def _hero(h, floors: dict) -> dict:
    marked = next(([depth, f[1].index(h.marked)] for depth, f in floors.items() if h.marked in f[1]), None)
    owned = h.all_items()
    quick = [q if q is None or isinstance(q, str) else (["item", owned.index(q)] if q in owned else None)
             for q in h.quickslots]
    return {"name": h.name, "class": h.class_key, "abilities": h.abilities, "lvl": h.lvl, "exp": h.exp,
            "HT": h.HT, "HP": h.HP, "pos": h.pos, "depth": h.depth, "hunger": h.hunger, "gold": h.gold,
            "weapon": _item(h.weapon) if h.weapon else None, "armor": _item(h.armor) if h.armor else None,
            "backpack": [_item(i) for i in h.backpack], "quick": quick, "second_wind": h.second_wind,
            "slots": h.slots, "free_casts": h.free_casts, "marked": marked,
            "dead": h.dead, "saves": h.saves, "stable": h.stable}


def dump(d) -> dict:
    return {"version": VERSION, "saved": datetime.now().isoformat(timespec="seconds"),
            "seed": d.seed, "rng": d.rng.getstate(), "messages": d.messages,
            "known": sorted(d.known), "generator": d.generator.weights,
            "quotas": [d.potions_of_strength, d.scrolls_of_upgrade], "rules": d.rules,
            "heroes": [_hero(h, d.floors) for h in d.heroes], "local": d.heroes.index(d.hero),
            "floors": {str(k): _floor(d, *f) for k, f in d.floors.items()}}


def _upgrade(data: dict) -> dict:
    """v1 (un solo héroe) -> v2. En v1 cada piso visitado tenía su propia regeneración y hambre (sólo corría la del
    piso activo); en v2 viajan con el héroe: se queda la del piso donde está."""
    if data.get("version") == 1:
        hd = data.pop("hero")
        hd["depth"] = data.pop("depth")
        data.update(version=2, heroes=[hd], local=0)
        for key, f in data["floors"].items():
            here = int(key) == hd["depth"]
            f["actors"] = [[a[0], 0, *a[1:]] if a[0] == "hero" or (here and a[0] in ("regen", "hunger"))
                           else ["buff", ["hero", 0], *a[2:]] if a[0] == "buff" and a[1] == "hero"
                           else a
                           for a in f["actors"] if here or a[0] not in ("regen", "hunger")]
    return data


def _load_hero(h, hd: dict) -> None:
    for k in ("name", "lvl", "exp", "HT", "HP", "pos", "depth", "hunger", "gold", "second_wind", "slots",
              "free_casts"):
        setattr(h, k, hd[k])
    # salvaciones contra la muerte: un guardado anterior no las trae (ahí 0 PV = muerto)
    h.dead, h.saves, h.stable = hd.get("dead", hd["HP"] <= 0), hd.get("saves", [0, 0]), hd.get("stable", False)
    h.update_awareness()
    h.weapon = _unitem(hd["weapon"]) if hd["weapon"] else None
    h.armor = _unitem(hd["armor"]) if hd["armor"] else None
    h.backpack = [_unitem(v) for v in hd["backpack"]]
    owned = h.all_items()
    h.quickslots = [owned[q[1]] if isinstance(q, list) else q for q in hd["quick"]]
    h.buffs = []


def load(data: dict) -> Dungeon:
    hds = data["heroes"]
    # mismas apariencias de pociones/pergaminos (salen de la seed). El piso 1 que genera se descarta abajo
    d = Dungeon(data["seed"], hds[0]["class"], hds[0]["abilities"])
    for hd in hds[1:]:
        d.heroes.append(Hero(d, hd["class"], -1, hd["abilities"]))
    v, state, gauss = data["rng"]
    d.rng.setstate((v, tuple(state), gauss))
    d.messages, d.known = data["messages"], set(data["known"])
    d.generator.weights = data["generator"]
    d.potions_of_strength, d.scrolls_of_upgrade = data["quotas"]
    d.rules.update(data.get("rules", {}))  # un guardado anterior: reglas por defecto (5e)
    hs = d.heroes
    for h, hd in zip(hs, hds):
        _load_hero(h, hd)
    d.floors = {}
    for key, f in data["floors"].items():
        depth = int(key)
        level = SewerLevel(d.seed, depth)  # mismo nivel (sale de la seed); se pisa lo que cambió
        level.map = f["map"]
        level.build_flag_maps()
        level.visited, level.mapped = f["visited"], f["mapped"]
        for pos, type_, its in f["heaps"]:
            heap = level.heaps[pos] = Heap(pos, type_)
            heap.items = [_unitem(v) for v in its]
        mobs = []
        for mkey, pos, HP, st, target, seen, alerted in f["mobs"]:
            m = Mob(d, mkey, pos, st)
            m.HP, m.target, m.enemy_seen, m.alerted = HP, target, seen, alerted
            mobs.append(m)
        sched = Scheduler()
        sched.now = f["now"]
        for a in f["actors"]:
            kind = a[0]
            if kind == "hero":
                actor = hs[a[1]]
            elif kind == "mob":
                actor = mobs[a[1]]
            elif kind == "regen":
                actor = hs[a[1]].ticks[0]
            elif kind == "hunger":
                actor = hs[a[1]].ticks[1]
            elif kind == "respawner":
                actor = Respawner(d)
            else:  # buff
                target = hs[a[1][1]] if isinstance(a[1], list) else mobs[a[1]]
                actor = Buff(target, a[2], a[3], a[4], a[5])
                target.buffs.append(actor)
            actor.time = a[-1]
            sched.actors.append(actor)
        d.floors[depth] = (level, mobs, sched)
    for h, hd in zip(hs, hds):
        if hd["marked"]:
            depth, i = hd["marked"]
            h.marked = d.floors[depth][1][i]
    for depth in dict.fromkeys(h.depth for h in hs):
        d.enter(depth)
        d.observe()
    d.set_local(hs[data["local"]])
    return d


def save_run(d, char_id: str) -> None:
    """Guarda con escritura atómica. Si el héroe murió, borra el guardado (como el original)."""
    if not any(h.is_alive() for h in d.heroes):
        delete_run(char_id)
        return
    d.resolve_pending()
    path = _path(char_id)
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(dump(d), ensure_ascii=False), encoding="utf-8")
    tmp.replace(path)


def load_run(char_id: str) -> Dungeon | None:
    """La partida guardada o None. Un guardado roto, de otra versión o con contenido que ya no existe (un mod que se
    quitó) se aparta con otro nombre, sin borrarlo."""
    path = _path(char_id)
    try:
        data = _upgrade(json.loads(path.read_text(encoding="utf-8")))
        if data.get("version") != VERSION:
            raise ValueError(f"versión {data.get('version')}")
        return load(data)
    except FileNotFoundError:
        return None
    except (ValueError, KeyError, TypeError, IndexError, UnicodeDecodeError):
        path.replace(path.with_name(f"{path.name}.roto-{datetime.now():%Y%m%d%H%M%S}"))
        return None


def delete_run(char_id: str) -> None:
    _path(char_id).unlink(missing_ok=True)
