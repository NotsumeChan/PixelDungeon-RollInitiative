"""Level.java (create) + RegularLevel.java + SewerLevel.java en una sola clase:
el alcance son sólo las Alcantarillas (pisos 1-4)."""
import math

from . import fov
from . import patch
from . import terrain as T
from .items import Heap, HEAP
from .geom import WIDTH, HEIGHT, LENGTH, NEIGHBOURS4, NEIGHBOURS8, NEIGHBOURS9, Rect
from .painters import PAINTERS, fill, fill_room
from .rng import Rng
from .room import Room, RoomType, Door, DoorType, SPECIALS, build_distance_map, build_path
from .terrain import Feeling

MIN_ROOM_SIZE = 7
MAX_ROOM_SIZE = 9

_SECRET_TRAPS = [T.SECRET_TOXIC_TRAP, T.SECRET_FIRE_TRAP, T.SECRET_PARALYTIC_TRAP, T.SECRET_POISON_TRAP,
                 T.SECRET_ALARM_TRAP, T.SECRET_LIGHTNING_TRAP, T.SECRET_GRIPPING_TRAP, T.SECRET_SUMMONING_TRAP]


def boss_level(depth: int) -> bool:
    return depth in (5, 10, 15, 20, 25)


def level_seed(seed: int, depth: int) -> int:
    # Estable entre procesos/peers (no usar hash()).
    return seed * 1000 + depth


class SewerLevel:
    def __init__(self, seed: int, depth: int) -> None:
        self.depth = depth
        self.boss = boss_level(depth)
        self.boss_next = boss_level(depth + 1)
        self.rng = Rng(level_seed(seed, depth))
        self.feeling = Feeling.NONE
        self.entrance = self.exit = -1
        self.secret_doors = 0
        self.tombs: list[tuple[int, bool]] = []  # tumbas del cementerio, se llenan de ítems más adelante
        self.rooms: list[Room] = []
        self.room_entrance: Room | None = None
        self.room_exit: Room | None = None
        self.specials: list[RoomType] = []
        self.create()

    # --- Level.create ---

    def create(self) -> None:
        # ponytail: Java agrega aquí ítems obligatorios (comida, pociones...) que consumen rng; llegan con los ítems.
        if self.depth > 1:
            c = self.rng.Int(10)
            if c == 0 and not self.boss_next:
                self.feeling = Feeling.CHASM
            elif c == 1:
                self.feeling = Feeling.WATER
            elif c == 2:
                self.feeling = Feeling.GRASS

        while True:
            self.map = [T.CHASM if self.feeling is Feeling.CHASM else T.WALL] * LENGTH
            if self.build():
                break
        self.decorate()
        self.build_flag_maps()
        self.clean_walls()
        self.visible = [False] * LENGTH   # lo que se ve ahora (Dungeon.visible)
        self.visited = [False] * LENGTH   # lo que se vio alguna vez
        self.mapped = [False] * LENGTH    # revelado por magia (Scroll of Magic Mapping)
        self.heaps: dict[int, Heap] = {}  # celda -> montón de ítems en el suelo
        # createMobs / createItems: fase 3

    # --- RegularLevel.build ---

    def build(self) -> bool:
        rng = self.rng
        self.tombs = []
        if not self.init_rooms():
            return False

        rooms = self.rooms
        min_distance = int(math.sqrt(len(rooms)))
        retry = 0
        while True:
            while True:
                entrance = rng.element(rooms)
                if entrance.width() >= 4 and entrance.height() >= 4:
                    break
            while True:
                exit_ = rng.element(rooms)
                if exit_ is not entrance and exit_.width() >= 4 and exit_.height() >= 4:
                    break
            build_distance_map(rooms, exit_)
            distance = entrance.distance
            if retry > 10:
                return False
            retry += 1
            if distance >= min_distance:
                break

        entrance.type = RoomType.ENTRANCE
        exit_.type = RoomType.EXIT
        self.room_entrance, self.room_exit = entrance, exit_

        # Dos caminos entrada -> salida; el segundo se desvía porque el primero se encarece.
        connected = [entrance]
        for second in (False, True):
            build_distance_map(rooms, exit_)
            path = build_path(entrance, exit_)
            if path is None:
                return False
            room = entrance
            for nxt in path:
                room.connect(nxt)
                room = nxt
                if room not in connected:
                    connected.append(room)
            if not second:
                for r in path:
                    r.price = entrance.distance

        n_connected = int(len(rooms) * rng.Float(0.5, 0.7))
        while len(connected) < n_connected:
            cr = rng.element(connected)
            other = rng.element(cr.neigbours)
            if other is not None and other not in connected:
                cr.connect(other)
                connected.append(other)

        # Shop: no aparece en pisos 1-4.
        # ponytail: paso A sin salas especiales; paso B -> copia viva de SPECIALS desde el estado de partida
        self.specials = []
        self.assign_room_type()
        self.paint()
        self.paint_water()
        self.paint_grass()
        self.place_traps()
        return True

    def init_rooms(self) -> bool:
        self.rooms = []
        self.split(Rect(0, 0, WIDTH - 1, HEIGHT - 1))
        if len(self.rooms) < 8:
            return False
        ra = self.rooms
        for i in range(len(ra) - 1):
            for j in range(i + 1, len(ra)):
                ra[i].add_neigbour(ra[j])
        return True

    def split(self, rect: Rect) -> None:
        """Partición recursiva del mapa en salas de 7..9 (aprox)."""
        rng = self.rng
        w, h = rect.width(), rect.height()
        if w > MAX_ROOM_SIZE and h < MIN_ROOM_SIZE:
            vw = rng.Int(rect.left + 3, rect.right - 3)
            self.split(Rect(rect.left, rect.top, vw, rect.bottom))
            self.split(Rect(vw, rect.top, rect.right, rect.bottom))
        elif h > MAX_ROOM_SIZE and w < MIN_ROOM_SIZE:
            vh = rng.Int(rect.top + 3, rect.bottom - 3)
            self.split(Rect(rect.left, rect.top, rect.right, vh))
            self.split(Rect(rect.left, vh, rect.right, rect.bottom))
        # Java: Math.random() (sin seed) y división ENTERA 49 / square -> rng y //
        elif ((rng.random() <= MIN_ROOM_SIZE * MIN_ROOM_SIZE // rect.square() and w <= MAX_ROOM_SIZE and h <= MAX_ROOM_SIZE)
              or w < MIN_ROOM_SIZE or h < MIN_ROOM_SIZE):
            self.rooms.append(Room(rect))
        elif rng.Float() < (w - 2) / (w + h - 4):
            vw = rng.Int(rect.left + 3, rect.right - 3)
            self.split(Rect(rect.left, rect.top, vw, rect.bottom))
            self.split(Rect(vw, rect.top, rect.right, rect.bottom))
        else:
            vh = rng.Int(rect.top + 3, rect.bottom - 3)
            self.split(Rect(rect.left, rect.top, rect.right, vh))
            self.split(Rect(rect.left, vh, rect.right, rect.bottom))

    def assign_room_type(self) -> None:
        rng = self.rng
        for r in self.rooms:
            if r.type is RoomType.NULL and len(r.connected) == 1:
                # ponytail: rama de salas especiales (PIT, LABORATORY en piso 2, SPECIALS) va en el paso B;
                # con specials vacío Java tampoco entra (y no consume rng).
                if rng.Int(2) == 0:
                    candidates = [n for n in r.neigbours
                                  if n not in r.connected and n.type not in SPECIALS and n.type is not RoomType.PIT]
                    if len(candidates) > 1:
                        r.connect(rng.element(candidates))

        count = 0
        for r in self.rooms:
            if r.type is RoomType.NULL:
                connections = len(r.connected)
                if connections == 0:
                    pass
                elif rng.Int(connections * connections) == 0:
                    r.type = RoomType.STANDARD
                    count += 1
                else:
                    r.type = RoomType.TUNNEL
        while count < 4:
            r = self.random_room(RoomType.TUNNEL, 1)
            if r is not None:
                r.type = RoomType.STANDARD
                count += 1

    # --- pintado ---

    def tunnel_tile(self) -> int:
        return T.EMPTY_SP if self.feeling is Feeling.CHASM else T.EMPTY

    def paint(self) -> None:
        for r in self.rooms:
            if r.type is not RoomType.NULL:
                self.place_doors(r)
                PAINTERS[r.type](self, r)
            elif self.feeling is Feeling.CHASM and self.rng.Int(2) == 0:
                fill_room(self, r, 0, T.WALL)
        for r in self.rooms:
            self.paint_doors(r)

    def place_doors(self, r: Room) -> None:
        for n, door in r.connected.items():
            if door is None:
                i = r.intersect(n)
                if i.width() == 0:
                    door = Door(i.left, self.rng.Int(i.top + 1, i.bottom))
                else:
                    door = Door(self.rng.Int(i.left + 1, i.right), i.top)
                r.connected[n] = door
                n.connected[r] = door

    def paint_doors(self, r: Room) -> None:
        for n, d in r.connected.items():
            if self.join_rooms(r, n):
                continue
            pos = d.x + d.y * WIDTH
            if d.type is DoorType.EMPTY:
                self.map[pos] = T.EMPTY
            elif d.type is DoorType.TUNNEL:
                self.map[pos] = self.tunnel_tile()
            elif d.type is DoorType.REGULAR:
                if self.depth <= 1:
                    self.map[pos] = T.DOOR
                else:
                    secret = (self.rng.Int(12 - self.depth) if self.depth < 6 else self.rng.Int(6)) == 0
                    self.map[pos] = T.SECRET_DOOR if secret else T.DOOR
                    if secret:
                        self.secret_doors += 1
            elif d.type is DoorType.UNLOCKED:
                self.map[pos] = T.DOOR
            elif d.type is DoorType.HIDDEN:
                self.map[pos] = T.SECRET_DOOR
                self.secret_doors += 1
            elif d.type is DoorType.BARRICADE:
                self.map[pos] = T.BOOKSHELF if self.rng.Int(3) == 0 else T.BARRICADE
            elif d.type is DoorType.LOCKED:
                self.map[pos] = T.LOCKED_DOOR

    def join_rooms(self, r: Room, n: Room) -> bool:
        """Dos STANDARD contiguas y chicas se fusionan abriendo la pared compartida."""
        if r.type is not RoomType.STANDARD or n.type is not RoomType.STANDARD:
            return False
        w = r.intersect(n)
        if w.left == w.right:
            if w.bottom - w.top < 3 or w.height() == max(r.height(), n.height()) or r.width() + n.width() > MAX_ROOM_SIZE:
                return False
            w.top += 1
            w.right += 1
            fill(self, w.left, w.top, 1, w.height(), T.EMPTY)
        else:
            if w.right - w.left < 3 or w.width() == max(r.width(), n.width()) or r.height() + n.height() > MAX_ROOM_SIZE:
                return False
            w.left += 1
            w.bottom += 1
            fill(self, w.left, w.top, w.width(), 1, T.EMPTY)
        return True

    def paint_water(self) -> None:
        lake = patch.generate(self.rng, 0.60 if self.feeling is Feeling.WATER else 0.45, 5)
        for i in range(LENGTH):
            if self.map[i] == T.EMPTY and lake[i]:
                self.map[i] = T.WATER

    def paint_grass(self) -> None:
        grass = patch.generate(self.rng, 0.60 if self.feeling is Feeling.GRASS else 0.40, 4)
        if self.feeling is Feeling.GRASS:
            for r in self.rooms:
                if r.type not in (RoomType.NULL, RoomType.PASSAGE, RoomType.TUNNEL):
                    for x, y in ((r.left + 1, r.top + 1), (r.right - 1, r.top + 1),
                                 (r.left + 1, r.bottom - 1), (r.right - 1, r.bottom - 1)):
                        grass[x + y * WIDTH] = True
        for i in range(WIDTH + 1, LENGTH - WIDTH - 1):
            if self.map[i] == T.EMPTY and grass[i]:
                count = 1 + sum(grass[i + n] for n in NEIGHBOURS8)
                self.map[i] = T.HIGH_GRASS if self.rng.Float() < count / 12 else T.GRASS

    def place_traps(self) -> None:
        n_traps = 0 if self.depth <= 1 else self.rng.Int(1, len(self.rooms) + self.depth)
        for _ in range(n_traps):
            pos = self.rng.Int(LENGTH)
            if self.map[pos] == T.EMPTY:
                self.map[pos] = _SECRET_TRAPS[self.rng.chances([1] * 8)]

    # --- SewerLevel.decorate ---

    def decorate(self) -> None:
        rng, m = self.rng, self.map
        for i in range(WIDTH):
            if m[i] == T.WALL and m[i + WIDTH] == T.WATER and rng.Int(4) == 0:
                m[i] = T.WALL_DECO
        for i in range(WIDTH, LENGTH - WIDTH):
            if m[i] == T.WALL and m[i - WIDTH] == T.WALL and m[i + WIDTH] == T.WATER and rng.Int(2) == 0:
                m[i] = T.WALL_DECO
        for i in range(WIDTH + 1, LENGTH - WIDTH - 1):
            if m[i] == T.EMPTY:
                count = (m[i + 1] == T.WALL) + (m[i - 1] == T.WALL) + (m[i + WIDTH] == T.WALL) + (m[i - WIDTH] == T.WALL)
                if rng.Int(16) < count * count:
                    m[i] = T.EMPTY_DECO
        while True:
            pos = self.room_entrance.random(rng)
            if pos != self.entrance:
                m[pos] = T.SIGN
                break

    # --- Level.buildFlagMaps / cleanWalls ---

    def build_flag_maps(self) -> None:
        f = [T.flags[t] for t in self.map]
        self.passable = [bool(x & T.PASSABLE) for x in f]
        self.los_blocking = [bool(x & T.LOS_BLOCKING) for x in f]
        self.flamable = [bool(x & T.FLAMABLE) for x in f]
        self.secret = [bool(x & T.SECRET) for x in f]
        self.solid = [bool(x & T.SOLID) for x in f]
        self.avoid = [bool(x & T.AVOID) for x in f]
        self.water = [bool(x & T.LIQUID) for x in f]
        self.pit = [bool(x & T.PIT) for x in f]

        # El borde del mapa nunca es transitable.
        for i in range(LENGTH):
            x, y = i % WIDTH, i // WIDTH
            if x in (0, WIDTH - 1) or y in (0, HEIGHT - 1):
                self.passable[i] = self.avoid[i] = False

        m = self.map
        for i in range(WIDTH, LENGTH - WIDTH):
            if self.water[i]:
                # índice 48..63 según qué vecinos (N, E, S, O) son "unstitchable"
                m[i] = T.WATER_TILES + sum(1 << j for j, n in enumerate(NEIGHBOURS4)
                                           if T.flags[m[i + n]] & T.UNSTITCHABLE)
            if self.pit[i] and not self.pit[i - WIDTH]:
                c = m[i - WIDTH]
                if c in (T.EMPTY_SP, T.STATUE_SP):
                    m[i] = T.CHASM_FLOOR_SP
                elif self.water[i - WIDTH]:
                    m[i] = T.CHASM_WATER
                elif T.flags[c] & T.UNSTITCHABLE:
                    m[i] = T.CHASM_WALL
                else:
                    m[i] = T.CHASM_FLOOR

    def clean_walls(self) -> None:
        """discoverable[i]: la celda toca algo que no es pared y no es todo abismo (para buscar secretos)."""
        m = self.map
        self.discoverable = [False] * LENGTH
        for i in range(LENGTH):
            near = [i + n for n in NEIGHBOURS9 if 0 <= i + n < LENGTH]
            self.discoverable[i] = (any(m[n] not in (T.WALL, T.WALL_DECO) for n in near)
                                    and any(not self.pit[n] for n in near))

    # --- visión (Level.updateFieldOfView + Dungeon.observe) ---

    def observe(self, *positions: int, view_distance: int = fov.MAX_DISTANCE) -> list[list[bool]]:
        """Recalcula visible desde cada posición (la unión de las vistas) y lo suma a visited.
        Devuelve la vista de cada posición (Dungeon.observe la guarda en cada héroe).
        ponytail: sin buffs (ceguera, MindVision, Awareness...); llegan con los buffs."""
        views = [fov.cast_shadow(p % WIDTH, p // WIDTH, self.los_blocking, view_distance) for p in positions]
        self.visible = [any(v) for v in zip(*views)] if views else [False] * LENGTH
        self.visited = [a or b for a, b in zip(self.visited, self.visible)]
        return views

    # --- cambios de terreno en juego ---

    def set(self, pos: int, terrain: int) -> None:
        """Level.set: cambia una celda y sus flags (no recalcula el cosido del agua)."""
        self.map[pos] = terrain
        f = T.flags[terrain]
        self.passable[pos] = bool(f & T.PASSABLE)
        self.los_blocking[pos] = bool(f & T.LOS_BLOCKING)
        self.flamable[pos] = bool(f & T.FLAMABLE)
        self.secret[pos] = bool(f & T.SECRET)
        self.solid[pos] = bool(f & T.SOLID)
        self.avoid[pos] = bool(f & T.AVOID)
        self.pit[pos] = bool(f & T.PIT)
        self.water[pos] = bool(f & T.LIQUID)

    def enter(self, pos: int) -> None:
        """Alguien pisa pos. Door.enter: la puerta se abre."""
        if self.map[pos] == T.DOOR:
            self.set(pos, T.OPEN_DOOR)

    def leave(self, pos: int) -> None:
        """Alguien sale de pos. Door.leave: la puerta se cierra (hay una sola criatura por celda)."""
        if self.map[pos] == T.OPEN_DOOR:
            self.set(pos, T.DOOR)

    def discover(self, pos: int) -> None:
        """Un secreto encontrado (puerta / trampa) pasa a su versión visible."""
        self.set(pos, T.discover(self.map[pos]))

    # --- utilidades ---

    def random_room(self, type_: RoomType, tries: int, rng: Rng | None = None) -> Room | None:
        rng = rng or self.rng
        for _ in range(tries):
            room = rng.element(self.rooms)
            if room.type is type_:
                return room
        return None

    def random_respawn_cell(self, rng: Rng, occupied) -> int:
        """Celda libre en una sala STANDARD, fuera de la vista del héroe; -1 tras 10 intentos.
        occupied(celda) -> criatura o None."""
        for _ in range(10):
            room = self.random_room(RoomType.STANDARD, 10, rng)
            if room is None:
                continue
            cell = room.random(rng)
            if not self.visible[cell] and occupied(cell) is None and self.passable[cell]:
                return cell
        return -1

    def random_drop_cell(self, rng: Rng) -> int:
        """RegularLevel.randomDropCell: celda transitable de una sala STANDARD."""
        while True:
            room = self.random_room(RoomType.STANDARD, 1, rng)
            if room is not None:
                pos = room.random(rng)
                if self.passable[pos]:
                    return pos

    def drop(self, item, cell: int, type_: str = HEAP) -> Heap:
        """Level.drop: deja un ítem en el suelo (se apila en el montón que ya haya)."""
        heap = self.heaps.get(cell)
        if heap is None:
            heap = self.heaps[cell] = Heap(cell, type_)
        heap.add(item)
        return heap

    def random_destination(self, rng: Rng) -> int:
        """Celda transitable de cualquier sala: destino de los mobs que deambulan."""
        while True:
            cell = rng.element(self.rooms).random(rng)
            if self.passable[cell]:
                return cell


# --- volcado ASCII para revisar a mano: python -m game.level <seed> <depth> ---

_ASCII = {T.WALL: "#", T.WALL_DECO: "%", T.EMPTY: ".", T.EMPTY_DECO: ".", T.EMPTY_SP: ",",
          T.GRASS: '"', T.HIGH_GRASS: "&", T.DOOR: "+", T.SECRET_DOOR: "S", T.ENTRANCE: "<", T.EXIT: ">",
          T.SIGN: "?", T.EMBERS: ":", T.INACTIVE_TRAP: "_", T.BOOKSHELF: "B", T.PEDESTAL: "P",
          T.FIRE_TRAP: "^", T.BARRICADE: "=", T.LOCKED_DOOR: "L"}


def to_ascii(level: SewerLevel) -> str:
    def ch(t: int) -> str:
        if T.WATER_TILES <= t <= T.WATER:
            return "~"
        if T.flags[t] & T.PIT:
            return " "
        if T.flags[t] & T.SECRET:
            return "S" if t == T.SECRET_DOOR else "^"
        return _ASCII.get(t, "*")
    return "\n".join("".join(ch(level.map[x + y * WIDTH]) for x in range(WIDTH)) for y in range(HEIGHT))


if __name__ == "__main__":
    import sys
    seed = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    depth = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    lvl = SewerLevel(seed, depth)
    print(f"seed={seed} depth={depth} feeling={lvl.feeling.name} salas={len(lvl.rooms)} "
          f"puertas secretas={lvl.secret_doors}")
    print(to_ascii(lvl))
    print("# pared  % pared deco  . suelo  , suelo especial  \" pasto  & pasto alto  ~ agua  (espacio) abismo")
    print("+ puerta  S puerta secreta  ^ trampa  < entrada  > salida  ? cartel  : brasas  B estante  P pedestal")
