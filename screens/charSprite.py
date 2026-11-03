#Animaciones de personajes: CharSprite.java + HeroSprite / MobSprite y el *Sprite de cada mob.
#Las de los mobs son datos (MOBS[id]["anims"], un mod trae las suyas); las del heroe son iguales en todas las clases.
#Animacion = (fps, frames, se repite). Todo avanza con dt.
import pygame as py
from game.geom import WIDTH
from game.hero import Hero

HERO_FRAME : tuple = (12, 15) #HeroSprite FRAME_WIDTH / FRAME_HEIGHT
HERO_ANIMS : dict = {         #HeroSprite.updateArmor
    "idle":    (1,  [0, 0, 0, 1, 0, 0, 1, 1], True),
    "run":     (20, [2, 3, 4, 5, 6, 7], True),
    "die":     (20, [8, 9, 10, 11, 12, 11], False),
    "attack":  (15, [13, 14, 15, 0], False),
    "operate": (8,  [16, 17, 16, 17], False),
    "read":    (20, [19, 20, 20, 20, 20, 20, 20, 20, 20, 19], False),
}
DEFAULT_ANIMS : dict = {"idle": (1, [0], True)} #mob sin "anims" (p. ej. de un mod): primer frame quieto
ONESHOT : tuple = ("attack", "operate", "read") #se reproducen enteras antes de volver a quieto / correr
FADE_TIME : float = 3.0 #MobSprite: tras morir, el cadaver se desvanece

_Sheets : dict = {}
_Frames : dict = {}

def Anims(ch) -> dict:
    return HERO_ANIMS if isinstance(ch, Hero) else ch.stats.get("anims", DEFAULT_ANIMS)

def FrameSize(ch) -> tuple:
    return HERO_FRAME if isinstance(ch, Hero) else tuple(ch.stats["frame"])

def Frame(ch, index : int, flip : bool = False) -> py.Surface:
    """Frame index de la hoja del personaje. Heroe: una fila por tier de armadura (HeroSprite.tiers); mob: los frames
    van de izquierda a derecha y bajan de fila (TextureFilm)."""
    w, h = FrameSize(ch)
    if isinstance(ch, Hero):
        file, row = ch.cls["sprite"], (ch.armor.data["tier"] if ch.armor else 0)
    else:
        file, row = ch.stats["sprite"], None
    key = (file, w, h, row, index, flip)
    if key not in _Frames:
        if file not in _Sheets:
            _Sheets[file] = py.image.load(f"Assets/Characters/{file}").convert_alpha()
        sheet = _Sheets[file]
        if row is None:
            cols = max(1, sheet.get_width() // w)
            x, y = (index % cols) * w, (index // cols) * h
        else:
            x, y = index * w, row * h
        img = sheet.subsurface((x, y, w, h))
        _Frames[key] = py.transform.flip(img, True, False) if flip else img
    return _Frames[key]

class CharAnim():
    """Animacion actual de un personaje (CharSprite.play). flip = mira a la izquierda (CharSprite.turnTo)."""
    def __init__(self, name : str = "idle", flip : bool = False) -> None:
        self.name, self.t, self.flip = name, 0.0, flip

    def play(self, name : str) -> None:
        self.name, self.t = name, 0.0

    def duration(self, anims : dict) -> float:
        fps, frames, loop = anims.get(self.name, anims["idle"])
        return len(frames) / fps

    def done(self, anims : dict) -> bool:
        if self.name not in anims: #ese personaje no tiene esa animacion (un mob que "lee")
            return True
        return not anims[self.name][2] and self.t >= self.duration(anims)

    def index(self, anims : dict) -> int:
        fps, frames, loop = anims.get(self.name, anims["idle"])
        i = int(self.t * fps)
        return frames[i % len(frames)] if loop else frames[min(i, len(frames) - 1)]

def Act(anims : dict, ch, kind : str, cell : int | None) -> None:
    """Ataque / operar / leer: se gira hacia la celda y reproduce la animacion (CharSprite.attack / operate)."""
    a = anims.setdefault(ch, CharAnim())
    if cell is not None and cell % WIDTH != ch.pos % WIDTH:
        a.flip = cell % WIDTH < ch.pos % WIDTH
    a.play(kind)

def Update(anims : dict, chars : list, sprites : dict, rest, dt : float) -> None:
    """Elige la animacion de cada personaje vivo: muere, termina la que estaba (ataque...), corre si su sprite
    todavia viaja hacia su celda, si no queda quieto. rest(ch) = posicion de reposo del sprite."""
    alive = set(chars)
    for ch in list(anims):
        if ch not in alive:
            del anims[ch]
    for ch in chars:
        a = anims.setdefault(ch, CharAnim())
        data = Anims(ch)
        if ch.is_alive() and ch.has_flag("paralysed"): #CharSprite.paralyse: la animacion se congela
            continue
        a.t += dt
        if not ch.is_alive() or ch.is_downed(): #caido a 0 PV: tendido (queda en el ultimo frame de morir)
            if a.name != "die":
                a.play("die")
            continue
        if a.name in ONESHOT and not a.done(data):
            continue
        pos, target = sprites.get(ch), rest(ch)
        if pos is not None and pos != target:
            if pos[0] != target[0]:
                a.flip = target[0] < pos[0]
            if a.name != "run":
                a.play("run")
        elif a.name != "idle":
            a.play("idle")

def Image(anims : dict, ch) -> py.Surface:
    a = anims.get(ch) or CharAnim()
    return Frame(ch, a.index(Anims(ch)), a.flip)

def CorpseAlpha(anim : CharAnim, ch) -> float:
    """1 mientras se ve la animacion de muerte; despues baja a 0 en FADE_TIME."""
    return max(0.0, 1 - max(0.0, anim.t - anim.duration(Anims(ch))) / FADE_TIME)
