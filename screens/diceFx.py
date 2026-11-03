#Animacion de dados: cada tirada que el heroe ve se muestra en un recuadro (numeros al azar y luego el resultado).
#Minimo (1) = borde rojo + vibracion. Maximo = borde dorado + chispas + vibracion.
#Tres tamanos: tirada decisiva (salvaciones: superada / fallada) grande en el centro; d20 de ataque mediano; el
#resto (dano...) chico, abajo a la derecha. Se escala por un entero: el pixel art queda nitido.
#El azar de aqui es SOLO visual (random global); el resultado real ya lo decidio game/ con su rng.
import random
import pygame as py
from core import Font1x

BOX_W, BOX_H = 34, 20                #tamano base del recuadro (escala 1)
BOX_MARGIN : tuple = (4, 29)         #distancia al borde derecho / inferior de los recuadros chico y mediano
ROLL_TIME, SHOW_TIME = 0.35, 1.2     #segundos girando / mostrando el resultado
MIN_SHOW = 0.7                       #si hay otra tirada en cola, el resultado se ve al menos esto (que se lea)
TICK_TIME = 0.05                     #cada cuanto cambia el numero mientras gira
SHAKE_TIME, SHAKE_PX = 0.3, 2        #vibracion en 1 o maximo (en pixeles de escala 1)
FAST_QUEUE = 5                       #con mas tiradas en cola, todo va al doble de rapido
#tamanos: (escala, segundos girando, segundos mostrando, minimo mostrando si hay cola)
SMALL  : tuple = (1, ROLL_TIME, SHOW_TIME, MIN_SHOW)
D20    : tuple = (2, ROLL_TIME, SHOW_TIME, MIN_SHOW)
DECIDE : tuple = (3, 0.7, 1.8, 1.2)  #salvaciones: gira mas (suspenso) y el resultado queda mas tiempo
DECIDE_RISE = 12                     #el recuadro decisivo va centrado, esto por encima del centro (no tapa al heroe)
RED, GOLD, GREEN = (220, 40, 40), (255, 200, 40), (60, 200, 60)
BG = (20, 18, 16, 210)

def Tier(event : tuple) -> tuple:
    """Decisiva = tiene resultado (ok no es None: salvaciones, muerte, Medicina); d20 = rango de 20 valores."""
    _, _, lo, hi, ok = event
    return DECIDE if ok is not None else D20 if hi - lo == 19 else SMALL

class DiceFx():
    def __init__(self) -> None:
        self.queue : list = []    #(etiqueta, valor, min, max, ok): ok = salvacion superada/fallada, None = ataque
        self.current = None
        self.tier : tuple = SMALL
        self.t = 0.0              #tiempo dentro de la animacion actual
        self.shown = 0            #numero que se ve mientras gira
        self.tick = 0.0
        self.sparks : list = []   #[x, y, vx, vy, vida] en pixeles de escala 1, desde el centro del recuadro
        self.anchor : tuple = (0, 0, 1) #centro del ultimo recuadro dibujado y su escala (las chispas lo siguen)

    def Feed(self, events : list) -> None:
        self.queue.extend(events)
        events.clear()

    def Update(self, dt : float) -> None:
        speed = 2.0 if len(self.queue) >= FAST_QUEUE else 1.0
        for s in self.sparks:
            s[0] += s[2] * dt
            s[1] += s[3] * dt
            s[3] += 200 * dt #gravedad
            s[4] -= dt
        self.sparks = [s for s in self.sparks if s[4] > 0]
        if self.current is None:
            if not self.queue:
                return
            self.current, self.t, self.tick = self.queue.pop(0), 0.0, 0.0
            self.tier = Tier(self.current)
        _, roll, show, minShow = self.tier
        self.t += dt * speed
        label, value, lo, hi, ok = self.current
        if self.t < roll:
            self.tick -= dt * speed
            if self.tick <= 0:
                self.shown, self.tick = random.randint(lo, hi), TICK_TIME
        elif self.t - dt * speed < roll and ((value == hi and hi > lo) or ok): #justo al terminar de girar
            self._burst(28 if ok else 14) #maximo, o salvacion superada
        if self.t >= roll + show or (self.queue and self.t >= roll + minShow):
            self.current = None

    def Busy(self) -> bool:
        """True mientras falte mostrar algún resultado (el juego espera antes de aplicar el daño)."""
        return bool(self.queue) or (self.current is not None and self.t < self.tier[1])

    def _burst(self, n : int) -> None:
        for _ in range(n):
            self.sparks.append([random.uniform(-BOX_W / 2, BOX_W / 2), random.uniform(-BOX_H / 2, BOX_H / 2),
                                random.uniform(-70, 70), random.uniform(-110, -30), random.uniform(0.3, 0.6)])

    def Draw(self, canvas : py.Surface) -> None:
        W, H = canvas.get_size()
        if self.current is not None:
            label, value, lo, hi, ok = self.current
            s, roll = self.tier[0], self.tier[1]
            rolling = self.t < roll
            extreme = None if rolling or hi == lo else RED if value == lo else GOLD if value == hi else None
            border = extreme if ok is None or rolling else GREEN if ok else RED #salvacion: verde superada, rojo fallada
            dx = dy = 0
            if (extreme or (ok is not None and not rolling)) and self.t - roll < SHAKE_TIME:
                dx, dy = random.randint(-SHAKE_PX, SHAKE_PX) * s, random.randint(-SHAKE_PX, SHAKE_PX) * s
            tag = Font1x.Render(label, (136, 136, 136))
            w = max(BOX_W, tag.get_width() + 6) #"CON CD11" no entra en el ancho base
            box = py.Surface((w, BOX_H), py.SRCALPHA)
            box.fill(BG)
            py.draw.rect(box, border or (120, 110, 90), box.get_rect(), 1)
            num = Font1x.Render(str(self.shown if rolling else value), None if rolling else border)
            box.blit(num, ((w - num.get_width()) // 2, 2))
            box.blit(tag, ((w - tag.get_width()) // 2, BOX_H - 9))
            if s > 1:
                box = py.transform.scale(box, (w * s, BOX_H * s))
            if self.tier is DECIDE: #centrado, un poco arriba del heroe
                x, y = (W - w * s) // 2, H // 2 - BOX_H * s - DECIDE_RISE
            else: #abajo a la derecha, alineado a la derecha
                x, y = W - BOX_MARGIN[0] - w * s, H - BOX_MARGIN[1] - BOX_H * s
            canvas.blit(box, (x + dx, y + dy))
            self.anchor = (x + w * s / 2, y + BOX_H * s / 2, s)
        cx, cy, s = self.anchor
        for sx, sy, _, _, life in self.sparks:
            canvas.fill(GOLD if life > 0.2 else (255, 120, 20), (int(cx + sx * s), int(cy + sy * s), s, s))
