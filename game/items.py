"""Ítems: items/*.java + Generator.java + Heap.java + ItemStatusHandler (apariencias sin identificar).
Todo el contenido usa IDs de texto "namespace:nombre" ("pddnd:" = juego base; un mod usaría "mimod:...").
Los efectos se referencian por NOMBRE (EFFECTS): un mod de datos sólo puede usar efectos que ya existen."""

from . import registry
from .lang import t

GOLD = "pddnd:gold"

# --- registro de ítems ---
# image = índice en Assets/Items/items.png (ItemSpriteSheet.java); pociones/pergaminos usan su apariencia.
# gen_weight = peso dentro de su categoría en el Generator (0 = no sale al azar).









ITEMS: dict[str, dict] = registry.load("items")  # data/items.json
# ponytail: faltan pociones/pergaminos con buffs o gases, varitas, anillos, semillas, armas arrojadizas, bombas

# Generator.Category: peso de cada categoría (se divide a la mitad cada vez que sale, se resetea por piso)
CATEGORY_WEIGHTS = {"weapon": 10, "missile": 5, "armor": 10, "potion": 50, "scroll": 40, "food": 0, "gold": 50}

# Apariencias sin identificar (Potion.colors / Scroll.runes), con su índice en items.png
_LOOKS = [("turquesa", "turquoise"), ("carmesí", "crimson"), ("celeste", "azure"), ("jade", "jade"), ("dorada", "golden"),
          ("magenta", "magenta"), ("negra", "charcoal"), ("marfil", "ivory"), ("ámbar", "amber"), ("parda", "bistre"),
          ("índigo", "indigo"), ("plateada", "silver")]
POTION_LOOKS = [({"es": es, "en": en}, 56 + i) for i, (es, en) in enumerate(_LOOKS)]
POTION_NAME = {"es": "poción {}", "en": "{} potion"}
SCROLL_NAME = {"es": 'pergamino "{}"', "en": 'scroll "{}"'}
SCROLL_LOOKS = [("KAUNAN", 40), ("SOWILO", 41), ("LAGUZ", 42), ("YNGVI", 43), ("GYFU", 44), ("RAIDO", 45),
                ("ISAZ", 46), ("MANNAZ", 47), ("NAUDIZ", 76), ("BERKANAN", 77), ("ODAL", 78), ("TIWAZ", 79)]

# Íconos de ranura vacía (ItemSpriteSheet.WEAPON / ARMOR / RING)
SLOT_WEAPON, SLOT_ARMOR, SLOT_RING = 5, 6, 7


class Item:
    __slots__ = ("id", "quantity", "level", "cursed", "level_known", "cursed_known")

    def __init__(self, item_id: str, quantity: int = 1) -> None:
        if item_id not in ITEMS:
            raise KeyError(f"ítem desconocido: {item_id}")
        self.id = item_id
        self.quantity = quantity
        self.level = 0
        self.cursed = False
        self.level_known = False
        self.cursed_known = False

    @property
    def data(self) -> dict:
        return ITEMS[self.id]

    @property
    def category(self) -> str:
        return self.data["category"]

    def stackable(self) -> bool:
        return self.data.get("stackable", False)

    def is_equipment(self) -> bool:
        return self.category in ("weapon", "armor")

    def split(self, n: int = 1) -> "Item":
        """Saca n de una pila (el original queda con el resto)."""
        part = Item(self.id, n)
        self.quantity -= n
        return part


# --- stats de equipo (5e) ---

def weapon_dice(w: Item) -> str:
    """Dados de daño con el nivel del arma sumado: +1 sword -> '1d8+1'."""
    return w.data["dice"] + (f"{w.level:+d}" if w.level else "")


def armor_ac(a: Item | None, dex_mod: int) -> int:
    """CA con esta armadura (None = sin armadura: 10 + DES). El nivel de la armadura suma a la CA."""
    if a is None:
        return 10 + dex_mod
    d = a.data
    dex = dex_mod if d["dex_max"] is None else min(dex_mod, d["dex_max"])
    return d["ac"] + dex + a.level


# --- nombres y textos según lo que el jugador sabe ---

def is_known(item: Item, dungeon) -> bool:
    if item.category in ("potion", "scroll"):
        return item.id in dungeon.known
    if item.is_equipment():
        return item.level_known and item.cursed_known
    return True


def item_name(item: Item, dungeon) -> str:
    if item.category in ("potion", "scroll") and item.id not in dungeon.known:
        look = t(dungeon.looks[item.id][0])
        return t(POTION_NAME if item.category == "potion" else SCROLL_NAME).format(look)
    name = t(item.data["name"])
    if (item.is_equipment() or item.data.get("upgradable")) and item.level_known and item.level:
        name = f"{item.level:+d} {name}"
    return name


def item_image(item: Item, dungeon):
    """Entero (items.png del original), [archivo, índice, nombre] (hoja propia o de un mod: el nombre es el fallback si
    no carga) o ["text", nombre] si no tiene sprite (se dibuja el nombre hasta que haya uno hecho por una persona)."""
    if item.category in ("potion", "scroll"):
        return dungeon.looks[item.id][1]
    image = item.data.get("image")
    if image is None:
        return ["text", item_name(item, dungeon)]
    return image if isinstance(image, int) else [*image[:2], item_name(item, dungeon)]


def item_info(item: Item, dungeon, hero) -> str:
    d = item.data
    kinds = {"simple": "simple", "martial": "marcial", "clothing": "ropa", "light": "ligera", "medium": "media",
             "heavy": "pesada"}
    if item.category == "weapon":
        dice = weapon_dice(item) if item.level_known else d["dice"]
        if d.get("ranged"):  # arco: a distancia con DES y gasta munición
            ammo = sum(i.quantity for i in hero.backpack if i.id == d["ammo"])
            text = (f"Arma a distancia {kinds[d['kind']]} (DES). Daño {dice}. Alcance {d['range'][0]}/{d['range'][1]} casillas"
                    f" (más lejos, desventaja). {t(ITEMS[d['ammo']]['name']).capitalize()}s: {ammo}."
                    f" Cuerpo a cuerpo golpea como un garrote (1d4)")
        else:
            text = f"Arma {kinds[d['kind']]}{', sutil (FUE o DES)' if d['finesse'] else ''}. Daño {dice}"
        if item.level_known and item.level:
            text += f", {item.level:+d} al ataque"
        if d["dly"] != 1:
            text += f". Velocidad x{1 / d['dly']:g}"
        text += f". Requiere FUE {d['STR']}"
        if d["STR"] > hero.STR:
            text += f" (te falta fuerza: desventaja y -{d['STR'] - hero.STR} al ataque)"
        if not hero.proficient_weapon(item):
            text += ". No eres competente: sin bonificador de competencia"
    elif item.category == "missile" and d.get("thrown"):
        r = d["range"]
        text = (f"Arma arrojadiza {kinds[d['kind']]}{', sutil (FUE o DES)' if d['finesse'] else ' (FUE)'}. "
                f"Daño {weapon_dice(item)}. Alcance {r[0]}/{r[1]} casillas (más lejos, desventaja)")
        if d["dly"] != 1:
            text += f". Velocidad x{1 / d['dly']:g}"
        text += ". Vuelve a tu mano" if d.get("returns") else ". Si acierta se pierde; si falla, cae al suelo"
        if d.get("desc"):
            text += ". " + t(d["desc"]).rstrip(".")
        text += f". Requiere FUE {d['STR']}"
        if d["STR"] > hero.STR:
            text += " (te falta fuerza: desventaja)"
        if not hero.proficient_weapon(item):
            text += ". No eres competente: sin bonificador de competencia"
        text += "."
    elif item.category == "armor":
        dex = "+ DES" if d["dex_max"] is None else "" if d["dex_max"] == 0 else f"+ DES (max {d['dex_max']})"
        bonus = f" {item.level:+d}" if item.level_known and item.level else ""
        text = f"Armadura {kinds[d['kind']]}. CA {d['ac']}{bonus} {dex}"
        if d["STR"]:
            text += f". Requiere FUE {d['STR']}"
        if d["STR"] > hero.STR:
            text += " (te falta fuerza: más lento y desventaja al atacar)"
        if not hero.proficient_armor(item):
            text += ". No eres competente: atacas con desventaja"
    elif item.category in ("potion", "scroll") and item.id not in dungeon.known:
        text = "No sabes qué hace. Hay una sola forma de averiguarlo."
    elif item.category == "gold":
        text = f"{item.quantity} monedas de oro."
    else:
        text = t(d.get("desc", ""))
    if item.is_equipment():
        if not item.level_known:
            text += ". No conoces su nivel ni si está maldita"
        elif item.cursed and item.cursed_known:
            text += ". Está maldita: no te la podrás quitar"
        text += "."
    return text


# --- Generator ---

def roll_equipment(item: Item, rng) -> Item:
    """Weapon.random / Armor.random: 40% de tener nivel; la mitad de esos son negativos y malditos."""
    if rng.Float() < 0.4:
        n = 1
        if rng.Int(3) == 0:
            n += 1
            if rng.Int(3) == 0:
                n += 1
        if rng.Int(2) == 0:
            item.level += n
        else:
            item.level -= n
            item.cursed = True
    return item


def make_gold(rng, depth: int) -> Item:
    return Item(GOLD, rng.Int(20 + depth * 10, 40 + depth * 20))


class Generator:
    """Generator.java. Estado por partida (los pesos de categoría se gastan al salir)."""

    def __init__(self) -> None:
        self.weights: dict[str, float] = {}
        self.reset()

    def reset(self) -> None:
        self.weights = dict(CATEGORY_WEIGHTS)

    def random(self, rng, dungeon) -> Item:
        cats = list(self.weights)
        return self.random_of(cats[rng.chances([self.weights[c] for c in cats])], rng, dungeon)

    def random_of(self, cat: str, rng, dungeon) -> Item:
        self.weights[cat] /= 2
        if cat == "gold":
            return make_gold(rng, dungeon.depth)
        if cat in ("weapon", "armor"):
            # saca dos y se queda con el de FUE más cercana a la del héroe
            hero = getattr(dungeon, "hero", None)
            cur_str = hero.STR if hero else 10 + dungeon.potions_of_strength
            a, b = (roll_equipment(Item(self._pick(cat, rng)), rng) for _ in range(2))
            return a if abs(cur_str - a.data["STR"]) < abs(cur_str - b.data["STR"]) else b
        item = Item(self._pick(cat, rng))
        if item.data.get("qty"):  # arrojadizas y flechas salen en montón
            item.quantity = rng.Int(*item.data["qty"])
        return item

    @staticmethod
    def _pick(cat: str, rng) -> str:
        ids = [i for i, d in ITEMS.items() if d["category"] == cat]
        return ids[rng.chances([ITEMS[i].get("gen_weight", 0) for i in ids])]


# --- montones en el suelo (Heap.java) ---

HEAP, CHEST, SKELETON, TOMB = "heap", "chest", "skeleton", "tomb"
HEAP_IMAGES = {CHEST: 11, TOMB: 13, SKELETON: 0}  # HEAP muestra el ítem de arriba


class Heap:
    def __init__(self, pos: int, type_: str = HEAP) -> None:
        self.pos = pos
        self.type = type_
        self.items: list[Item] = []  # items[0] = el de arriba

    def add(self, item: Item) -> None:
        if item.stackable():
            for i in self.items:
                if i.id == item.id:
                    i.quantity += item.quantity
                    self.items.remove(i)
                    self.items.insert(0, i)
                    return
        self.items.insert(0, item)

    def image(self, dungeon) -> int:
        if self.type in HEAP_IMAGES:
            return HEAP_IMAGES[self.type]
        return item_image(self.items[0], dungeon)


# --- efectos (pociones / pergaminos). Firma: (hero, item, target) ---

def _heal(hero, item, target) -> None:
    hero.heal(hero.HT)
    hero.dungeon.log("Tus heridas se cierran")


def _strength(hero, item, target) -> None:
    hero.abilities["STR"] = min(hero.abilities["STR"] + 1, 20)
    hero.dungeon.log(f"Te sientes más fuerte: FUE {hero.STR}")


def _experience(hero, item, target) -> None:
    hero.earn_exp(hero.max_exp() - hero.exp)


def _identify(hero, item, target) -> None:
    target.level_known = target.cursed_known = True
    hero.dungeon.known.add(target.id)
    hero.dungeon.log(f"Es {item_name(target, hero.dungeon)}")


def _upgrade(hero, item, target) -> None:
    target.level += 1
    target.cursed = False
    target.cursed_known = True
    hero.dungeon.log(f"Tu {t(target.data['name'])} brilla")


def _remove_curse(hero, item, target) -> None:
    cursed = [i for i in hero.all_items() if i.cursed]
    for i in cursed:
        i.cursed = False
        i.cursed_known = True
    hero.dungeon.log("Una luz limpia tu equipo" if cursed else "Sientes una calma extraña")


def _magic_mapping(hero, item, target) -> None:
    level = hero.dungeon.level
    for i, d in enumerate(level.discoverable):
        if d:
            level.mapped[i] = True
            if level.secret[i]:
                level.discover(i)
    hero.dungeon.observe()
    hero.dungeon.log("Recuerdas el mapa de este piso")


def _teleport(hero, item, target) -> None:
    d = hero.dungeon
    pos = d.level.random_respawn_cell(d.rng, d.find_char)
    if pos == -1:
        d.log("No pasa nada")
        return
    hero.move(pos)
    d.observe()
    hero.interrupt()
    d.log("Apareces en otro lugar")


EFFECTS = {"heal": _heal, "strength": _strength, "experience": _experience, "identify": _identify,
           "upgrade": _upgrade, "remove_curse": _remove_curse, "magic_mapping": _magic_mapping,
           "teleport": _teleport}

# Qué ítems acepta un pergamino con objetivo (data["target"])
TARGETS = {
    "unknown": lambda item, dungeon: not is_known(item, dungeon),
    "upgradable": lambda item, dungeon: item.is_equipment() or item.data.get("upgradable", False),
}
