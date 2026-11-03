from collections import deque
from enum import Enum, IntEnum

from .geom import Rect, WIDTH
from .rng import Rng


class RoomType(Enum):
    NULL = 0
    STANDARD = 1
    ENTRANCE = 2
    EXIT = 3
    BOSS_EXIT = 4
    TUNNEL = 5
    PASSAGE = 6
    SHOP = 7
    BLACKSMITH = 8
    TREASURY = 9
    ARMORY = 10
    LIBRARY = 11
    LABORATORY = 12
    VAULT = 13
    TRAPS = 14
    STORAGE = 15
    MAGIC_WELL = 16
    GARDEN = 17
    CRYPT = 18
    STATUE = 19
    POOL = 20
    RAT_KING = 21
    WEAK_FLOOR = 22
    PIT = 23
    ALTAR = 24


# Orden original de Room.SPECIALS. En Java es global y se reordena entre pisos (useType/shuffleTypes);
# aquí la copia viva irá en el estado de partida (paso B).
SPECIALS = [RoomType.ARMORY, RoomType.WEAK_FLOOR, RoomType.MAGIC_WELL, RoomType.CRYPT, RoomType.POOL,
            RoomType.GARDEN, RoomType.LIBRARY, RoomType.TREASURY, RoomType.TRAPS, RoomType.STORAGE,
            RoomType.STATUE, RoomType.LABORATORY, RoomType.VAULT, RoomType.ALTAR]


class DoorType(IntEnum):
    # El orden importa: Door.set sólo "sube" de tipo.
    EMPTY = 0
    TUNNEL = 1
    REGULAR = 2
    UNLOCKED = 3
    HIDDEN = 4
    BARRICADE = 5
    LOCKED = 6


class Door:
    __slots__ = ("x", "y", "type")

    def __init__(self, x: int, y: int) -> None:
        self.x, self.y, self.type = x, y, DoorType.EMPTY

    def set(self, t: DoorType) -> None:
        if t > self.type:
            self.type = t


class Room(Rect):
    """Room.java. Hereda de Rect; las paredes son el borde del rect, el interior va de left+1 a right-1."""
    __slots__ = ("type", "neigbours", "connected", "distance", "price")

    def __init__(self, r: Rect) -> None:
        super().__init__(r.left, r.top, r.right, r.bottom)
        self.type = RoomType.NULL
        self.neigbours: list[Room] = []
        self.connected: dict[Room, Door | None] = {}  # dict conserva orden de inserción -> determinista
        self.distance = 0
        self.price = 1

    __hash__ = object.__hash__  # identidad, como en Java

    def random(self, rng: Rng, m: int = 0) -> int:
        """Celda al azar del interior, a m celdas de la pared."""
        x = rng.Int(self.left + 1 + m, self.right - m)
        y = rng.Int(self.top + 1 + m, self.bottom - m)
        return x + y * WIDTH

    def add_neigbour(self, other: "Room") -> None:
        """Vecinas = comparten una pared de al menos 3 celdas."""
        i = self.intersect(other)
        if (i.width() == 0 and i.height() >= 3) or (i.height() == 0 and i.width() >= 3):
            self.neigbours.append(other)
            other.neigbours.append(self)

    def connect(self, other: "Room") -> None:
        if other not in self.connected:
            self.connected[other] = None
            other.connected[self] = None

    def inside(self, p: int) -> bool:
        x, y = p % WIDTH, p // WIDTH
        return self.left < x < self.right and self.top < y < self.bottom

    def center(self, rng: Rng) -> tuple[int, int]:
        # // de Python == / entera de Java aquí (coordenadas siempre >= 0)
        return ((self.left + self.right) // 2 + (rng.Int(2) if self.width() & 1 else 0),
                (self.top + self.bottom) // 2 + (rng.Int(2) if self.height() & 1 else 0))


# --- Graph.java (sólo se usa con salas, por eso vive aquí) ---

def build_distance_map(rooms: list[Room], focus: Room) -> None:
    """BFS ponderado por `price` desde focus sobre neigbours (no sobre connected)."""
    for r in rooms:
        r.distance = 2**31 - 1
    focus.distance = 0
    queue = deque([focus])
    while queue:
        node = queue.popleft()
        for edge in node.neigbours:
            if edge.distance > node.distance + node.price:
                edge.distance = node.distance + node.price
                queue.append(edge)


def build_path(start: Room, end: Room) -> list[Room] | None:
    """Baja por el mapa de distancias desde start hasta end. None si no hay camino."""
    path = []
    room = start
    while room is not end:
        nxt, best = None, room.distance
        for edge in room.neigbours:
            if edge.distance < best:
                best, nxt = edge.distance, edge
        if nxt is None:
            return None
        path.append(nxt)
        room = nxt
    return path
