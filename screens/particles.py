#Efectos visuales de PD: Speck (gases, grito, destello), FlameParticle, PoisonParticle, SparkParticle, Lightning, Wound.
#Coordenadas en pixeles del mundo; todo avanza con dt. El azar es solo visual (random global).
import math
import random
import pygame as py
from game import terrain as T
from game.geom import WIDTH, NEIGHBOURS9
from .gameUi import ItemIcon

TILE : int = 16
_Specks  : py.Surface = py.image.load("Assets/specks.png").convert_alpha()
_Effects : py.Surface = py.image.load("Assets/effects.png").convert_alpha()
SPECK : int = 7                                         #Speck.SIZE
STEAM, LIGHT, SCREAM = 13, 2, 5                         #frames de specks.png
ARC_IMG, WOUND_IMG = (16, 0, 16, 8), (16, 8, 16, 8)     #Effects.java: LIGHTNING, WOUND
#Speck: tipo -> (frame, color o None, grados por segundo)
SPECKS : dict = {"toxic": (STEAM, (0x50, 0xFF, 0x60), 30), "paralysis": (STEAM, (0xFF, 0xFF, 0x66), -30),
                 "scream": (SCREAM, None, 0), "light": (LIGHT, None, 90)}
GAS_TIME, GAS_INTERVAL = 2.0, 0.6  #sin Blob: la nube de una trampa suelta humo por celda cada 0.6 s durante 2 s
ARC_TIME, WOUND_TIME, APPEAR_TIME = 0.3, 0.8, 0.4
FLAME_INTERVAL : float = 0.06       #CharSprite (BURNING): pour(FlameParticle, 0.06)
FLAME, POISON_A, POISON_B, SPARK = (0xEE, 0x77, 0x22), (0x00, 0xFF, 0x00), (0x88, 0x44, 0xFF), (0xFF, 0xFF, 0xEE)
#MagicMissile: viaja a 200 px/s soltando particulas; tipo de conjuro -> particula de la estela
BOLT_SPEED, BOLT_INTERVAL = 200.0, 0.015
MISSILE_SPEED : float = 240.0 #MissileSprite.SPEED
BOLT_TRAIL : dict = {"fire": "flame", "frost": "cold", "missile": "white", "sleep": "pink"}
GLOW : dict = {"cold": (0x88, 0xCC, 0xFF), "white": (0xFF, 0xFF, 0xFF), "pink": (0xFF, 0x88, 0xCC)}

_Cache : dict = {}

def _center(cell : int) -> tuple:
    return ((cell % WIDTH) + 0.5) * TILE, ((cell // WIDTH) + 0.5) * TILE

def _inside(cell : int) -> tuple:
    return ((cell % WIDTH) + random.random()) * TILE, ((cell // WIDTH) + random.random()) * TILE

def _speck_img(frame : int, color) -> py.Surface:
    key = ("speck", frame, color)
    if key not in _Cache:
        img = _Specks.subsurface((frame * SPECK, 0, SPECK, SPECK)).copy()
        if color: #hardlight: el humo blanco toma el color
            img.fill((*color, 255), special_flags=py.BLEND_RGBA_MULT)
        _Cache[key] = img
    return _Cache[key]

def _speck_look(kind : str, p : float) -> tuple:
    """Speck.update: (alpha, escala) segun el avance p (0 -> 1)."""
    if kind in ("toxic", "paralysis"):
        return (p if p < 0.5 else 1 - p), 1 + p * 2
    if kind == "scream":
        return math.sqrt((p if p < 0.5 else 1 - p) * 2), p * 7
    v = p * 5 if p < 0.2 else (1 - p) * 1.25 #light
    return v, v

class Effects():
    def __init__(self) -> None:
        self.pixels : list = []   #[tipo, x, y, vx, vy, ay, vida restante, vida total]
        self.specks : list = []   #[tipo, x, y, angulo, vida restante, vida total]
        self.emitters : list = [] #[tipo de speck, celda, intervalo, timer, tiempo restante, en el centro]
        self.arcs : list = []     #[(x1, y1), (x2, y2), vida]
        self.wounds : list = []   #[celda, vida]
        self.pour : dict = {}     #(personaje, tipo) -> timer
        self.appear : dict = {}   #personaje -> segundos desde que aparecio
        self.bolts : list = []    #[x, y, destino x, destino y, particula, timer]
        self.missiles : list = [] #[x, y, destino x, destino y, imagen, angulo, giro (grados/s)]

    def Clear(self) -> None:
        self.__init__()

    #--- disparadores ---
    def Trap(self, trap : int, cell : int, level) -> None:
        """levels/traps/*.java. Sin Blob: los gases sueltan humo un rato en el area 3x3."""
        if trap in (T.TOXIC_TRAP, T.PARALYTIC_TRAP):
            kind = "toxic" if trap == T.TOXIC_TRAP else "paralysis"
            for n in NEIGHBOURS9:
                c = cell + n
                if 0 <= c < len(level.solid) and not level.solid[c]:
                    self.emitters.append([kind, c, GAS_INTERVAL, random.uniform(0, GAS_INTERVAL), GAS_TIME, False])
        elif trap == T.FIRE_TRAP:
            for _ in range(5):
                self.Pixel("flame", *_inside(cell))
        elif trap == T.POISON_TRAP:
            for _ in range(3):
                self.Pixel("poison", *_center(cell))
        elif trap == T.ALARM_TRAP: #3 gritos, uno cada 0.3 s
            self.emitters.append(["scream", cell, 0.3, 0.0, 0.9, True])
        elif trap == T.LIGHTNING_TRAP: #un arco vertical y otro horizontal que cruzan la celda
            x, y = _center(cell)
            self.arcs += [[(x, y - TILE), (x, y + TILE), ARC_TIME], [(x - TILE, y), (x + TILE, y), ARC_TIME]]
            for _ in range(random.randint(3, 4)):
                self.Pixel("spark", x, y)
        elif trap == T.GRIPPING_TRAP:
            self.wounds.append([cell, WOUND_TIME])

    def Missile(self, frm : int, to : int, image, spin : float) -> None:
        """MissileSprite: el arma vuela a 240 px/s. Sin giro (dardos, jabalinas, flechas) vuela apuntando: el sprite
        original mira al noreste, de ahi el 135."""
        x0, y0 = _center(frm)
        x1, y1 = _center(to)
        angle = 135 - math.degrees(math.atan2(x1 - x0, y1 - y0)) if spin == 0 else 0.0
        self.missiles.append([x0, y0, x1, y1, image, angle, spin])

    def Bolt(self, frm : int, to : int, fx : str) -> None:
        """MagicMissile: proyectil de un conjuro. "mark" no viaja: destella sobre el objetivo."""
        if fx == "mark":
            self.emitters.append(["light", to, 0.15, 0.0, 0.45, True])
        elif fx in BOLT_TRAIL:
            self.bolts.append([*_center(frm), *_center(to), BOLT_TRAIL[fx], 0.0])

    def Appear(self, ch) -> None:
        """WandOfBlink.appear: el sprite aparece en 0.4 s y suelta 3 destellos."""
        self.appear[ch] = 0.0
        self.emitters.append(["light", ch.pos, 0.2, 0.0, 0.6, True])

    def Pour(self, key, rect : py.Rect, dt : float) -> None:
        """Llamas sobre un personaje en llamas, una cada FLAME_INTERVAL."""
        t = self.pour.get(key, 0.0) - dt
        while t <= 0:
            self.Pixel("flame", rect.x + random.random() * rect.w, rect.y + random.random() * rect.h)
            t += FLAME_INTERVAL
        self.pour[key] = t

    def Pixel(self, kind : str, x : float, y : float) -> None:
        if kind == "flame": #FlameParticle: sube acelerando y se achica
            self.pixels.append([kind, x, y, 0.0, 0.0, -80.0, 0.6, 0.6])
        elif kind == "poison": #PoisonParticle.SPLASH
            a, s = random.uniform(0, math.pi), random.uniform(10, 20)
            self.pixels.append([kind, x, y, math.cos(a) * s, math.sin(a) * s, 30.0, 0.6, 0.6])
        elif kind == "spark": #SparkParticle: salta hacia arriba y cae
            a, s, life = -random.uniform(0, math.pi), random.uniform(20, 40), random.uniform(0.5, 1.0)
            self.pixels.append([kind, x, y, math.cos(a) * s, math.sin(a) * s, 50.0, life, life])
        elif kind in GLOW: #estela de un proyectil: se dispersa un poco y se apaga
            self.pixels.append([kind, x, y, random.uniform(-8, 8), random.uniform(-8, 8), 0.0, 0.4, 0.4])

    def Speck(self, kind : str, x : float, y : float) -> None:
        life = random.uniform(1, 3) if kind in ("toxic", "paralysis") else 0.9 if kind == "scream" else 1.0
        self.specks.append([kind, x, y, random.uniform(0, 360), life, life])

    #--- tiempo ---
    def Update(self, dt : float, visible : list) -> None:
        for b in self.bolts:
            dx, dy = b[2] - b[0], b[3] - b[1]
            dist, step = math.hypot(dx, dy), BOLT_SPEED * dt
            b[5] -= dt
            while b[5] <= 0:
                self.Pixel(b[4], b[0], b[1])
                b[5] += BOLT_INTERVAL
            if dist <= step:
                b[0], b[1] = b[2], b[3]
            else:
                b[0] += dx / dist * step
                b[1] += dy / dist * step
        self.bolts = [b for b in self.bolts if (b[0], b[1]) != (b[2], b[3])]
        for m in self.missiles:
            dx, dy = m[2] - m[0], m[3] - m[1]
            dist, step = math.hypot(dx, dy), MISSILE_SPEED * dt
            m[5] += m[6] * dt
            if dist <= step:
                m[0], m[1] = m[2], m[3]
            else:
                m[0] += dx / dist * step
                m[1] += dy / dist * step
        self.missiles = [m for m in self.missiles if (m[0], m[1]) != (m[2], m[3])]
        for p in self.pixels:
            p[1] += p[3] * dt
            p[2] += p[4] * dt
            p[4] += p[5] * dt
            p[6] -= dt
        self.pixels = [p for p in self.pixels if p[6] > 0]
        for s in self.specks:
            s[3] += SPECKS[s[0]][2] * dt
            s[4] -= dt
        self.specks = [s for s in self.specks if s[4] > 0]
        for e in self.emitters: #BlobEmitter: solo suelta humo en celdas que se ven
            e[3] -= dt
            while e[3] <= 0 and e[4] > 0:
                if visible[e[1]]:
                    self.Speck(e[0], *(_center(e[1]) if e[5] else _inside(e[1])))
                e[3] += e[2]
            e[4] -= dt
        self.emitters = [e for e in self.emitters if e[4] > 0]
        for a in self.arcs:
            a[2] -= dt
        self.arcs = [a for a in self.arcs if a[2] > 0]
        for w in self.wounds:
            w[1] -= dt
        self.wounds = [w for w in self.wounds if w[1] > 0]
        for ch in list(self.appear):
            self.appear[ch] += dt
            if self.appear[ch] >= APPEAR_TIME:
                del self.appear[ch]

    def Alpha(self, ch) -> float:
        """Transparencia del sprite mientras aparece (1 = normal)."""
        return min(1.0, self.appear[ch] / APPEAR_TIME) if ch in self.appear else 1.0

    #--- dibujo ---
    def Draw(self, canvas : py.Surface, ox : int, oy : int) -> None:
        for cell, life in self.wounds: #Wound: se desvanece y se angosta
            p = life / WOUND_TIME
            img = py.transform.scale(_Effects.subsurface(WOUND_IMG), (int(WOUND_IMG[2] * (1 + p)), WOUND_IMG[3]))
            img.set_alpha(int(255 * p))
            x, y = _center(cell)
            canvas.blit(img, (ox + int(x) - img.get_width() // 2, oy + int(y) - img.get_height() // 2))
        for kind, x, y, angle, left, life in self.specks:
            am, scale = _speck_look(kind, 1 - left / life)
            if am <= 0 or scale <= 0:
                continue
            img = py.transform.rotozoom(_speck_img(SPECKS[kind][0], SPECKS[kind][1]), -angle, scale)
            img.set_alpha(int(255 * min(1.0, am)))
            canvas.blit(img, (ox + int(x) - img.get_width() // 2, oy + int(y) - img.get_height() // 2))
        for kind, x, y, _, _, _, left, life in self.pixels: #lightMode: se suman (BLEND_ADD)
            k = left / life
            if kind == "flame":
                am, size, color = ((1 - k) * 5 if k > 0.8 else 1), 4 * k, FLAME
            elif kind == "poison":
                am, size = 1, 4 - k * 3
                color = tuple(int(a + (b - a) * k) for a, b in zip(POISON_A, POISON_B))
            elif kind in GLOW:
                am, size, color = k, 2, GLOW[kind]
            else:
                am, size, color = 1, random.uniform(0, 5 * k), SPARK
            s = max(1, round(size))
            canvas.fill(tuple(int(c * am) for c in color), (ox + int(x - s / 2), oy + int(y - s / 2), s, s),
                        special_flags=py.BLEND_ADD)
        for x, y, _, _, image, angle, _ in self.missiles:
            img = py.transform.rotate(ItemIcon(image), -angle)
            canvas.blit(img, (ox + int(x) - img.get_width() // 2, oy + int(y) - img.get_height() // 2))
        for start, end, life in self.arcs: #Lightning: el punto medio tiembla cada frame
            mid = ((start[0] + end[0]) / 2 + random.uniform(-4, 4), (start[1] + end[1]) / 2 + random.uniform(-4, 4))
            for a, b in ((start, mid), (mid, end)):
                dx, dy = b[0] - a[0], b[1] - a[1]
                img = py.transform.scale(_Effects.subsurface(ARC_IMG), (max(1, int(math.hypot(dx, dy))), ARC_IMG[3]))
                img = py.transform.rotate(img, -math.degrees(math.atan2(dy, dx)))
                img.set_alpha(int(255 * life / ARC_TIME))
                cx, cy = (a[0] + b[0]) / 2, (a[1] + b[1]) / 2
                canvas.blit(img, (ox + int(cx) - img.get_width() // 2, oy + int(cy) - img.get_height() // 2))
