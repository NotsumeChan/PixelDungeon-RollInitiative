"""painters/*.java: cada tipo de sala sabe "pintarse" sobre level.map.
Paso A: STANDARD, ENTRANCE, EXIT, TUNNEL. Las especiales se agregan en el paso B."""
from . import terrain as T
from .geom import WIDTH
from .room import Room, RoomType, DoorType
from .terrain import Feeling


# --- Painter.java ---

def set_cell(level, x: int, y: int, value: int) -> None:
    level.map[x + y * WIDTH] = value


def fill(level, x: int, y: int, w: int, h: int, value: int) -> None:
    for j in range(y, y + h):
        start = x + j * WIDTH
        level.map[start:start + max(w, 0)] = [value] * max(w, 0)


def fill_room(level, room: Room, m: int, value: int) -> None:
    """fill(level, rect, m, value): el rect incluye sus paredes; m = margen hacia adentro."""
    fill(level, room.left + m, room.top + m, room.width() + 1 - m * 2, room.height() + 1 - m * 2, value)


def draw_inside(level, room: Room, frm, n: int, value: int) -> tuple[int, int]:
    """Desde una puerta (en la pared), avanza n celdas hacia el interior pintando value."""
    if frm.x == room.left:
        sx, sy = 1, 0
    elif frm.x == room.right:
        sx, sy = -1, 0
    elif frm.y == room.top:
        sx, sy = 0, 1
    else:
        sx, sy = 0, -1
    x, y = frm.x + sx, frm.y + sy
    for _ in range(n):
        set_cell(level, x, y, value)
        x, y = x + sx, y + sy
    return x, y


# --- EntrancePainter / ExitPainter ---

def _stairs_room(level, room: Room, tile: int) -> int:
    fill_room(level, room, 0, T.WALL)
    fill_room(level, room, 1, T.EMPTY)
    for door in room.connected.values():
        door.set(DoorType.REGULAR)
    pos = room.random(level.rng, 1)
    level.map[pos] = tile
    return pos


def paint_entrance(level, room: Room) -> None:
    level.entrance = _stairs_room(level, room, T.ENTRANCE)


def paint_exit(level, room: Room) -> None:
    level.exit = _stairs_room(level, room, T.EXIT)


# --- TunnelPainter ---

def paint_tunnel(level, room: Room) -> None:
    rng = level.rng
    floor = level.tunnel_tile()
    cx, cy = room.center(rng)
    doors = list(room.connected.values())
    if room.width() > room.height() or (room.width() == room.height() and rng.Int(2) == 0):
        frm, to = room.right - 1, room.left + 1
        for d in doors:
            step = 1 if d.y < cy else -1
            if d.x == room.left:
                frm = room.left + 1
                for i in range(d.y, cy, step):
                    set_cell(level, frm, i, floor)
            elif d.x == room.right:
                to = room.right - 1
                for i in range(d.y, cy, step):
                    set_cell(level, to, i, floor)
            else:
                frm, to = min(frm, d.x), max(to, d.x)
                for i in range(d.y + step, cy, step):
                    set_cell(level, d.x, i, floor)
        for i in range(frm, to + 1):
            set_cell(level, i, cy, floor)
    else:
        frm, to = room.bottom - 1, room.top + 1
        for d in doors:
            step = 1 if d.x < cx else -1
            if d.y == room.top:
                frm = room.top + 1
                for i in range(d.x, cx, step):
                    set_cell(level, i, frm, floor)
            elif d.y == room.bottom:
                to = room.bottom - 1
                for i in range(d.x, cx, step):
                    set_cell(level, i, to, floor)
            else:
                frm, to = min(frm, d.y), max(to, d.y)
                for i in range(d.x + step, cx, step):
                    set_cell(level, i, d.y, floor)
        for i in range(frm, to + 1):
            set_cell(level, cx, i, floor)
    for d in doors:
        d.set(DoorType.TUNNEL)


# --- StandardPainter ---

def paint_standard(level, room: Room) -> None:
    rng = level.rng
    fill_room(level, room, 0, T.WALL)
    for door in room.connected.values():
        door.set(DoorType.REGULAR)

    if not level.boss and rng.Int(5) == 0:
        c = rng.Int(6)
        w, h = room.width(), room.height()
        # El switch de Java tiene fall-through: case 0 con GRASS cae a case 1, case 4 con WATER cae a case 5.
        if c == 0 and level.feeling is not Feeling.GRASS:
            if min(w, h) >= 4 and max(w, h) >= 6:
                return _graveyard(level, room)
        elif c in (0, 1):
            if level.depth > 1:
                return _burned(level, room)
        elif c == 2:
            if max(w, h) >= 4:
                return _striped(level, room)
        elif c == 3:
            if w >= 6 and h >= 6:
                return _study(level, room)
        elif c == 4 and level.feeling is not Feeling.WATER:
            if len(room.connected) == 2 and w >= 4 and h >= 4:
                return _bridge(level, room)
        elif c in (4, 5):
            if not level.boss and not level.boss_next and min(w, h) >= 5:
                return _fissure(level, room)

    fill_room(level, room, 1, T.EMPTY)


def _burned(level, room: Room) -> None:
    pick = (T.EMPTY, T.FIRE_TRAP, T.SECRET_FIRE_TRAP, T.INACTIVE_TRAP, T.EMBERS)
    for y in range(room.top + 1, room.bottom):
        for x in range(room.left + 1, room.right):
            level.map[x + y * WIDTH] = pick[level.rng.Int(5)]


def _graveyard(level, room: Room) -> None:
    rng = level.rng
    fill(level, room.left + 1, room.top + 1, room.width() - 1, room.height() - 1, T.GRASS)
    w, h = room.width() - 1, room.height() - 1
    n_graves = max(w, h) // 2
    index = rng.Int(n_graves)
    shift = rng.Int(2)
    for i in range(n_graves):
        if w > h:
            pos = room.left + 1 + shift + i * 2 + (room.top + 2 + rng.Int(h - 2)) * WIDTH
        else:
            pos = (room.left + 2 + rng.Int(w - 2)) + (room.top + 1 + shift + i * 2) * WIDTH
        # ponytail: tumbas como (celda, tiene_item); el drop real (item / oro, Heap TOMB) llega con los ítems
        level.tombs.append((pos, i == index))


def _striped(level, room: Room) -> None:
    fill(level, room.left + 1, room.top + 1, room.width() - 1, room.height() - 1, T.EMPTY_SP)
    if room.width() > room.height():
        for x in range(room.left + 2, room.right, 2):
            fill(level, x, room.top + 1, 1, room.height() - 1, T.HIGH_GRASS)
    else:
        for y in range(room.top + 2, room.bottom, 2):
            fill(level, room.left + 1, y, room.width() - 1, 1, T.HIGH_GRASS)


def _study(level, room: Room) -> None:
    fill(level, room.left + 1, room.top + 1, room.width() - 1, room.height() - 1, T.BOOKSHELF)
    fill(level, room.left + 2, room.top + 2, room.width() - 3, room.height() - 3, T.EMPTY_SP)
    for d in room.connected.values():
        if d.x == room.left:
            set_cell(level, d.x + 1, d.y, T.EMPTY)
        elif d.x == room.right:
            set_cell(level, d.x - 1, d.y, T.EMPTY)
        elif d.y == room.top:
            set_cell(level, d.x, d.y + 1, T.EMPTY)
        elif d.y == room.bottom:
            set_cell(level, d.x, d.y - 1, T.EMPTY)
    set_cell(level, *room.center(level.rng), T.PEDESTAL)


def _bridge(level, room: Room) -> None:
    rng = level.rng
    chasm = not level.boss and not level.boss_next and rng.Int(3) == 0
    fill(level, room.left + 1, room.top + 1, room.width() - 1, room.height() - 1, T.CHASM if chasm else T.WATER)
    d1, d2 = list(room.connected.values())
    sp = T.EMPTY_SP
    if {d1.x, d2.x} == {room.left, room.right}:
        s = room.width() // 2
        draw_inside(level, room, d1, s, sp)
        draw_inside(level, room, d2, s, sp)
        fill(level, room.center(rng)[0], min(d1.y, d2.y), 1, abs(d1.y - d2.y) + 1, sp)
    elif {d1.y, d2.y} == {room.top, room.bottom}:
        s = room.height() // 2
        draw_inside(level, room, d1, s, sp)
        draw_inside(level, room, d2, s, sp)
        fill(level, min(d1.x, d2.x), room.center(rng)[1], abs(d1.x - d2.x) + 1, 1, sp)
    elif d1.x == d2.x:
        fill(level, room.left + 1 if d1.x == room.left else room.right - 1, min(d1.y, d2.y),
             1, abs(d1.y - d2.y) + 1, sp)
    elif d1.y == d2.y:
        fill(level, min(d1.x, d2.x), room.top + 1 if d1.y == room.top else room.bottom - 1,
             abs(d1.x - d2.x) + 1, 1, sp)
    elif d1.y in (room.top, room.bottom):
        draw_inside(level, room, d1, abs(d1.y - d2.y), sp)
        draw_inside(level, room, d2, abs(d1.x - d2.x), sp)
    elif d1.x in (room.left, room.right):
        draw_inside(level, room, d1, abs(d1.x - d2.x), sp)
        draw_inside(level, room, d2, abs(d1.y - d2.y), sp)


def _fissure(level, room: Room) -> None:
    fill(level, room.left + 1, room.top + 1, room.width() - 1, room.height() - 1, T.EMPTY)
    for y in range(room.top + 2, room.bottom - 1):
        for x in range(room.left + 2, room.right - 1):
            v = min(y - room.top, room.bottom - y)
            h = min(x - room.left, room.right - x)
            if min(v, h) > 2 or level.rng.Int(2) == 0:
                set_cell(level, x, y, T.CHASM)


PAINTERS = {
    RoomType.STANDARD: paint_standard,
    RoomType.ENTRANCE: paint_entrance,
    RoomType.EXIT: paint_exit,
    RoomType.TUNNEL: paint_tunnel,
}
