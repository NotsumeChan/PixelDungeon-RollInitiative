"""actors/buffs/*.java: estados alterados (veneno, sangrado, quemadura, paralisis, lisiado...).
Son DATOS (BUFFS, IDs "namespace:nombre") + una clase Buff generica, igual que MOBS y HERO_CLASSES:
- un mod de datos agrega o pisa entradas de BUFFS combinando lo que ya existe (TICKS por nombre, daño con dados,
  velocidad, flags, salvacion para terminarlo). Nunca apunta a codigo.
- un mod con codigo puede registrar comportamientos nuevos en TICKS y usarlos por nombre."""
from . import dice, registry
from .actor import Actor, TICK
from .lang import t

# Campos (todos opcionales salvo name):
#   name       texto de la ficha y el log: str o dict por idioma ({"es": ..., "en": ...}, ver game/lang.py)
#   icon       indice en Assets/buffs.png (BuffIndicator.java, 7x7); sin icon no se muestra
#   duration   turnos por defecto; sin duracion dura hasta que su tick lo termine (left = None)
#   tick       comportamiento por turno: clave de TICKS
#   damage     dados de daño por turno ("1d4"): daño continuo sin escribir codigo
#   speed      multiplicador de velocidad (0.5 = la mitad)
#   flags      estados que consulta el juego. "paralysed": no actua, falla sola las salvaciones de FUE y DES y
#              le atacan con ventaja
#   fx         efecto visual sobre el personaje mientras dura: "flame" (echa llamas)
#   save_ends  caracteristica de la salvacion (5e) que se repite al final de cada turno; si la supera, se termina.
#              La CD es la del efecto que lo causo (Buff.dc)
BUFFS: dict[str, dict] = registry.load("buffs")  # data/buffs.json


def _poison(b: "Buff") -> None:
    """Poison.java: (turnos restantes / 3) + 1 de daño por turno."""
    b.target.damage(int((b.left or 0) // 3) + 1, b)  # sin duración (un mod): 1 por turno


def _bleeding(b: "Buff") -> None:
    """Bleeding.java: cada turno el nivel baja al azar entre la mitad y el total; en 0 se termina."""
    b.level = b.target.dungeon.rng.IntRange(b.level // 2, b.level)
    if b.level > 0:
        b.target.damage(b.level, b)
    else:
        b.left = 0


def _burning(b: "Buff") -> None:
    """Burning.java: 1-5 por turno; se apaga en el agua o al azar (mas facil cuanta menos vida queda)."""
    t, d = b.target, b.target.dungeon
    t.damage(d.rng.IntRange(1, 5), b)
    if (d.level.water[t.pos] and not t.flying) or d.rng.Float() > (2 + t.HP / t.HT) / 3:
        b.left = 0
    # ponytail: no quema pergaminos de la mochila ni prende el pasto (falta Fire / Blob)


TICKS: dict = {"poison": _poison, "bleeding": _bleeding, "burning": _burning}


class Buff(Actor):
    """Un estado sobre un personaje; actua en el planificador del piso una vez por turno.
    left = turnos restantes (None = sin limite), level = intensidad (sangrado), dc = CD de save_ends."""

    def __init__(self, target, buff_id: str, duration: float | None = None, level: int = 0, dc: int = 0) -> None:
        super().__init__()
        self.target = target
        self.id = buff_id
        self.data = BUFFS[buff_id]
        self.left = duration if duration is not None else self.data.get("duration")
        self.level = level
        self.dc = dc

    @property
    def name(self) -> str:
        return t(self.data["name"])

    def act(self) -> None:
        t = self.target
        if t.is_alive():
            tick = self.data.get("tick")
            if tick:
                TICKS[tick](self)
            if "damage" in self.data and t.is_alive():
                t.damage(dice.roll(t.dungeon.rng, self.data["damage"])[0], self)
        self.spend(TICK)
        if self.left is not None:
            self.left -= TICK
        ability = self.data.get("save_ends")
        if ability and t.is_alive() and (self.left is None or self.left > 0) and t.saving_throw(ability, self.dc, self.name.lower())[0]:
            self.left = 0
        if not t.is_alive() or (self.left is not None and self.left <= 0):
            self.detach()

    def detach(self) -> None:
        t = self.target
        if self in t.buffs:
            t.buffs.remove(self)
            if t.is_alive() and t.dungeon.level.visible[t.pos]:
                t.dungeon.log(f"{t.name}: ya no esta {self.name.lower()}")
        t.dungeon.scheduler.remove(self)
