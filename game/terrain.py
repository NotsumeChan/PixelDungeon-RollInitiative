from enum import Enum

# Valores idénticos a Terrain.java: el render (fase 2) usa el valor como índice de tile en tiles0.png.
CHASM = 0
EMPTY = 1
GRASS = 2
EMPTY_WELL = 3
WALL = 4
DOOR = 5
OPEN_DOOR = 6
ENTRANCE = 7
EXIT = 8
EMBERS = 9
LOCKED_DOOR = 10
PEDESTAL = 11
WALL_DECO = 12
BARRICADE = 13
EMPTY_SP = 14
HIGH_GRASS = 15
EMPTY_DECO = 24
LOCKED_EXIT = 25
UNLOCKED_EXIT = 26
SIGN = 29
WELL = 34
STATUE = 35
STATUE_SP = 36
BOOKSHELF = 41
ALCHEMY = 42
CHASM_FLOOR = 43
CHASM_FLOOR_SP = 44
CHASM_WALL = 45
CHASM_WATER = 46

SECRET_DOOR = 16
TOXIC_TRAP = 17
SECRET_TOXIC_TRAP = 18
FIRE_TRAP = 19
SECRET_FIRE_TRAP = 20
PARALYTIC_TRAP = 21
SECRET_PARALYTIC_TRAP = 22
INACTIVE_TRAP = 23
POISON_TRAP = 27
SECRET_POISON_TRAP = 28
ALARM_TRAP = 30
SECRET_ALARM_TRAP = 31
LIGHTNING_TRAP = 32
SECRET_LIGHTNING_TRAP = 33
GRIPPING_TRAP = 37
SECRET_GRIPPING_TRAP = 38
SUMMONING_TRAP = 39
SECRET_SUMMONING_TRAP = 40

WATER_TILES = 48  # 48..63: agua "cosida" con sus bordes (Level.getWaterTile)
WATER = 63

PASSABLE = 0x01
LOS_BLOCKING = 0x02
FLAMABLE = 0x04
SECRET = 0x08
SOLID = 0x10
AVOID = 0x20
LIQUID = 0x40
PIT = 0x80
UNSTITCHABLE = 0x100

flags = [0] * 256
flags[CHASM] = AVOID | PIT | UNSTITCHABLE
flags[EMPTY] = PASSABLE
flags[GRASS] = PASSABLE | FLAMABLE
flags[EMPTY_WELL] = PASSABLE
flags[WATER] = PASSABLE | LIQUID | UNSTITCHABLE
flags[WALL] = LOS_BLOCKING | SOLID | UNSTITCHABLE
flags[DOOR] = PASSABLE | LOS_BLOCKING | FLAMABLE | SOLID | UNSTITCHABLE
flags[OPEN_DOOR] = PASSABLE | FLAMABLE | UNSTITCHABLE
flags[ENTRANCE] = PASSABLE
flags[EXIT] = PASSABLE
flags[EMBERS] = PASSABLE
flags[LOCKED_DOOR] = LOS_BLOCKING | SOLID | UNSTITCHABLE
flags[PEDESTAL] = PASSABLE | UNSTITCHABLE
flags[WALL_DECO] = flags[WALL]
flags[BARRICADE] = FLAMABLE | SOLID | LOS_BLOCKING
flags[EMPTY_SP] = flags[EMPTY] | UNSTITCHABLE
flags[HIGH_GRASS] = PASSABLE | LOS_BLOCKING | FLAMABLE
flags[EMPTY_DECO] = flags[EMPTY]
flags[LOCKED_EXIT] = SOLID
flags[UNLOCKED_EXIT] = PASSABLE
flags[SIGN] = PASSABLE | FLAMABLE
flags[WELL] = AVOID
flags[STATUE] = SOLID
flags[STATUE_SP] = flags[STATUE] | UNSTITCHABLE
flags[BOOKSHELF] = flags[BARRICADE] | UNSTITCHABLE
flags[ALCHEMY] = PASSABLE
flags[CHASM_WALL] = flags[CHASM]
flags[CHASM_FLOOR] = flags[CHASM]
flags[CHASM_FLOOR_SP] = flags[CHASM]
flags[CHASM_WATER] = flags[CHASM]
flags[SECRET_DOOR] = flags[WALL] | SECRET | UNSTITCHABLE
flags[INACTIVE_TRAP] = flags[EMPTY]
for _trap, _secret in ((TOXIC_TRAP, SECRET_TOXIC_TRAP), (FIRE_TRAP, SECRET_FIRE_TRAP),
                       (PARALYTIC_TRAP, SECRET_PARALYTIC_TRAP), (POISON_TRAP, SECRET_POISON_TRAP),
                       (ALARM_TRAP, SECRET_ALARM_TRAP), (LIGHTNING_TRAP, SECRET_LIGHTNING_TRAP),
                       (GRIPPING_TRAP, SECRET_GRIPPING_TRAP), (SUMMONING_TRAP, SECRET_SUMMONING_TRAP)):
    flags[_trap] = AVOID
    flags[_secret] = flags[EMPTY] | SECRET
for _i in range(WATER_TILES, WATER_TILES + 16):
    flags[_i] = flags[WATER]

# Terreno secreto -> su versión descubierta
_DISCOVER = {SECRET_DOOR: DOOR, SECRET_FIRE_TRAP: FIRE_TRAP, SECRET_PARALYTIC_TRAP: PARALYTIC_TRAP,
             SECRET_TOXIC_TRAP: TOXIC_TRAP, SECRET_POISON_TRAP: POISON_TRAP, SECRET_ALARM_TRAP: ALARM_TRAP,
             SECRET_LIGHTNING_TRAP: LIGHTNING_TRAP, SECRET_GRIPPING_TRAP: GRIPPING_TRAP,
             SECRET_SUMMONING_TRAP: SUMMONING_TRAP}


def discover(terr: int) -> int:
    return _DISCOVER.get(terr, terr)


class Feeling(Enum):
    """Level.Feeling: tema del piso, elegido al crear el nivel."""
    NONE = 0
    CHASM = 1
    WATER = 2
    GRASS = 3

# --- Level.tileName / tileDesc (+ SewerLevel) para la ventana de examinar (str o dict por idioma, ver game/lang.py) ---
from .lang import t as _t

_FLOOR, _WALL, _STATUE = {"es": "Suelo", "en": "Floor"}, {"es": "Pared", "en": "Wall"}, {"es": "Estatua", "en": "Statue"}
TILE_NAMES = {
    CHASM: {"es": "Abismo", "en": "Chasm"}, EMPTY: _FLOOR, EMPTY_SP: _FLOOR, EMPTY_DECO: _FLOOR,
    GRASS: {"es": "Pasto", "en": "Grass"}, WATER: {"es": "Agua turbia", "en": "Murky water"},
    WALL: _WALL, WALL_DECO: _WALL, SECRET_DOOR: _WALL,
    DOOR: {"es": "Puerta cerrada", "en": "Closed door"}, OPEN_DOOR: {"es": "Puerta abierta", "en": "Open door"},
    ENTRANCE: {"es": "Entrada del piso", "en": "Depth entrance"}, EXIT: {"es": "Salida del piso", "en": "Depth exit"},
    EMBERS: {"es": "Brasas", "en": "Embers"}, LOCKED_DOOR: {"es": "Puerta cerrada con llave", "en": "Locked door"},
    PEDESTAL: {"es": "Pedestal", "en": "Pedestal"}, BARRICADE: {"es": "Barricada", "en": "Barricade"},
    HIGH_GRASS: {"es": "Pasto alto", "en": "High grass"}, LOCKED_EXIT: {"es": "Salida bloqueada", "en": "Locked depth exit"},
    UNLOCKED_EXIT: {"es": "Salida desbloqueada", "en": "Unlocked depth exit"}, SIGN: {"es": "Cartel", "en": "Sign"},
    WELL: {"es": "Pozo", "en": "Well"}, EMPTY_WELL: {"es": "Pozo seco", "en": "Empty well"},
    STATUE: _STATUE, STATUE_SP: _STATUE,
    TOXIC_TRAP: {"es": "Trampa de gas tóxico", "en": "Toxic gas trap"}, FIRE_TRAP: {"es": "Trampa de fuego", "en": "Fire trap"},
    PARALYTIC_TRAP: {"es": "Trampa de gas paralizante", "en": "Paralytic gas trap"},
    POISON_TRAP: {"es": "Trampa de dardo envenenado", "en": "Poison dart trap"},
    ALARM_TRAP: {"es": "Trampa de alarma", "en": "Alarm trap"}, LIGHTNING_TRAP: {"es": "Trampa eléctrica", "en": "Lightning trap"},
    GRIPPING_TRAP: {"es": "Trampa de garras", "en": "Gripping trap"},
    SUMMONING_TRAP: {"es": "Trampa de invocación", "en": "Summoning trap"},
    INACTIVE_TRAP: {"es": "Trampa activada", "en": "Triggered trap"}, BOOKSHELF: {"es": "Estantería", "en": "Bookshelf"},
    ALCHEMY: {"es": "Caldero de alquimia", "en": "Alchemy pot"},
}
for _secret in (SECRET_TOXIC_TRAP, SECRET_FIRE_TRAP, SECRET_PARALYTIC_TRAP, SECRET_POISON_TRAP, SECRET_ALARM_TRAP,
                SECRET_LIGHTNING_TRAP, SECRET_GRIPPING_TRAP, SECRET_SUMMONING_TRAP):
    TILE_NAMES[_secret] = _FLOOR  # una trampa oculta se ve como suelo
_TRAP_DESC = {"es": "Pisar la placa de presión activa la trampa.",
              "en": "Stepping onto a hidden pressure plate will activate the trap."}
_DOWN = {"es": "La escalera baja al piso de abajo.", "en": "Stairs lead down to the lower depth."}
_STATUE_DESC = {"es": "Alguien quiso adornar este lugar, pero es obvio que no lo logró.",
                "en": "Someone wanted to adorn this place, but failed, obviously."}
TILE_DESCS = {
    CHASM: {"es": "No se ve el fondo.", "en": "You can't see the bottom."},
    WATER: {"es": "Si te prendes fuego, métete en el agua para apagarlo.",
            "en": "In case of burning step into the water to extinguish the fire."},
    ENTRANCE: {"es": "La escalera sube al piso de arriba.", "en": "Stairs lead up to the upper depth."},
    EXIT: _DOWN, UNLOCKED_EXIT: _DOWN,
    EMBERS: {"es": "Brasas cubren el suelo.", "en": "Embers cover the floor."},
    HIGH_GRASS: {"es": "La vegetación densa tapa la vista.", "en": "Dense vegetation blocks the view."},
    LOCKED_DOOR: {"es": "Esta puerta está cerrada con llave: necesitas la llave que le corresponde.",
                  "en": "This door is locked, you need a matching key to unlock it."},
    LOCKED_EXIT: {"es": "Unas rejas pesadas bloquean la escalera que baja.", "en": "Heavy bars block the stairs leading down."},
    BARRICADE: {"es": "La barricada de madera está firme, pero se secó con los años. ¿Arderá?",
                "en": "The wooden barricade is firmly set but has dried over the years. Might it burn?"},
    SIGN: {"es": "Desde aquí no se puede leer.", "en": "You can't read the text from here."},
    INACTIVE_TRAP: {"es": "La trampa ya se activó y no es peligrosa.",
                    "en": "The trap has been triggered before and it's not dangerous anymore."},
    STATUE: _STATUE_DESC, STATUE_SP: _STATUE_DESC,
    ALCHEMY: {"es": "Deja unas semillas aquí para preparar una poción.", "en": "Drop some seeds here to cook a potion."},
    EMPTY_WELL: {"es": "El pozo se secó.", "en": "The well has run dry."},
    EMPTY_DECO: {"es": "Un musgo húmedo y amarillento cubre el suelo.", "en": "Wet yellowish moss covers the floor."},
    BOOKSHELF: {"es": "La estantería está llena de libros baratos e inútiles. ¿Arderá?",
                "en": "The bookshelf is packed with cheap useless books. Might it burn?"},
}
for _trap in (TOXIC_TRAP, FIRE_TRAP, PARALYTIC_TRAP, POISON_TRAP, ALARM_TRAP, LIGHTNING_TRAP, GRIPPING_TRAP,
              SUMMONING_TRAP):
    TILE_DESCS[_trap] = _TRAP_DESC


def tile_info(terr: int, water: bool, pit: bool) -> tuple[int, str, str]:
    """WndInfoCell: (tile a mostrar, nombre, descripción) en el idioma actual. El agua y el abismo se nombran por su flag."""
    tile = WATER if water else CHASM if pit else terr
    return tile, _t(TILE_NAMES.get(tile, "???")), _t(TILE_DESCS.get(tile, ""))
