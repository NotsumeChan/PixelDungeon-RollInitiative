#Creacion de personaje estilo D&D 5e: nombre + clase + 6 caracteristicas (tiradas 4d6 o escritas a mano).
#Las tiradas se asignan arrastrandolas (drag and drop) o con click en la tirada y luego en la caracteristica.
import random
import pygame as py
from core import Screen, PickResolution, FrameRate, Font1x
from game import dice, characters
from game.globals import Globals
from game.hero import HERO_CLASSES
from game.lang import t
from game.rng import Rng
from . import gameUi as ui
from .gameScreen import Game, NewGame, Avatar

#screen define
CreateScreen : Screen = Screen(PickResolution[0], PickResolution[1])

YELLOW, GREY = (0xFF, 0xFF, 0x44), (136, 136, 136)
SEL_BG, BOX_BG = (0x99, 0x88, 0x33, 0xAA), (0x53, 0x56, 0x4D, 0x99)
NAME_MAX : int = 16
DRAG_MIN_PX : int = 3 #menos que esto entre apretar y soltar = click

class CreateState():
    def __init__(self) -> None:
        self.name : str = ""
        self.cls : str = next(iter(HERO_CLASSES))
        self.Clear()
        self.pool : list = []           #[(valor, [4 dados])]
        self.drag = None                #("chip", i) | ("stat", a) mientras se arrastra
        self.press = (0, 0)
        self.chip_rects : dict = {}     #i -> Rect (se rellenan al dibujar)
        self.stat_rects : dict = {}     #a -> Rect
        self.name_rect = py.Rect(0, 0, 0, 0)

    def Clear(self) -> None:
        self.scores = {a: None for a in dice.ABILITIES}
        self.source = {a: None for a in dice.ABILITIES} #indice de la tirada asignada a cada caracteristica
        self.chip = None    #tirada seleccionada (modo click)
        self.focus = None   #"name" o una caracteristica: recibe lo que se escribe
        self.typed = ""

    def Roll(self) -> None:
        self.pool = dice.roll_ability_pool(Rng(random.randrange(2**31)))
        for a in dice.ABILITIES: #lo asignado desde tiradas viejas se borra
            if self.source[a] is not None:
                self.scores[a] = self.source[a] = None
        self.chip = None

    def Assign(self, chip : int, a : str) -> None:
        """Pone la tirada `chip` en `a`; si esa tirada estaba en otra caracteristica, se mueve."""
        for other in dice.ABILITIES:
            if self.source[other] == chip:
                self.scores[other] = self.source[other] = None
        self.scores[a], self.source[a] = self.pool[chip][0], chip

    def Swap(self, a : str, b : str) -> None:
        self.scores[a], self.scores[b] = self.scores[b], self.scores[a]
        self.source[a], self.source[b] = self.source[b], self.source[a]

    def ClickChip(self, i : int) -> None:
        self.chip = None if self.chip == i else i
        self.focus = None

    def ClickStat(self, a : str) -> None:
        if self.chip is not None:
            self.Assign(self.chip, a)
            self.chip = None
        else: #modo escribir
            self.focus, self.typed = a, ""

    def Type(self, event) -> None:
        if self.focus == "name":
            if event.key == py.K_BACKSPACE:
                self.name = self.name[:-1]
            elif event.key in (py.K_RETURN, py.K_TAB):
                self.focus, self.typed = dice.ABILITIES[0], ""
            elif event.unicode and event.unicode.isprintable() and len(self.name) < NAME_MAX:
                self.name += event.unicode
            return
        if self.focus is None:
            return
        if event.key == py.K_BACKSPACE:
            self.typed = self.typed[:-1]
        elif event.unicode.isdigit() and len(self.typed) < 2:
            self.typed += event.unicode
        elif event.key in (py.K_RETURN, py.K_TAB): #siguiente caracteristica
            i = dice.ABILITIES.index(self.focus)
            self.focus, self.typed = dice.ABILITIES[(i + 1) % len(dice.ABILITIES)], ""
            return
        else:
            return
        value = int(self.typed) if self.typed else None
        self.scores[self.focus] = value if value is not None and dice.MIN_SCORE <= value <= dice.MAX_SCORE else None
        self.source[self.focus] = None

    #--- mouse: drag and drop o click ---
    def Under(self, pos) -> tuple:
        for i, r in self.chip_rects.items():
            if r.collidepoint(pos):
                return ("chip", i)
        for a, r in self.stat_rects.items():
            if r.collidepoint(pos):
                return ("stat", a)
        if self.name_rect.collidepoint(pos):
            return ("name", None)
        return (None, None)

    def MouseDown(self, pos) -> bool:
        """True si lo tomo esta pantalla (si no, va a los botones)."""
        kind, key = self.Under(pos)
        self.press = pos
        if kind == "chip" or (kind == "stat" and self.scores[key] is not None and self.chip is None):
            self.drag = (kind, key)
            return True
        if kind == "stat":
            self.ClickStat(key)
            return True
        if kind == "name":
            self.focus, self.chip = "name", None
            return True
        return False

    def MouseUp(self, pos) -> None:
        if self.drag is None:
            return
        kind, key = self.drag
        self.drag = None
        moved = abs(pos[0] - self.press[0]) + abs(pos[1] - self.press[1]) >= DRAG_MIN_PX
        tkind, tkey = self.Under(pos)
        if not moved: #fue un click
            if kind == "chip":
                self.ClickChip(key)
            else:
                self.ClickStat(key)
        elif kind == "chip" and tkind == "stat":
            self.Assign(key, tkey)
            self.chip = None
        elif kind == "stat" and tkind == "stat" and tkey != key:
            self.Swap(key, tkey)
        elif kind == "stat" and tkind != "stat": #arrastrada afuera: se quita (la tirada vuelve a la lista)
            self.scores[key] = self.source[key] = None

def Box(canvas : py.Surface, rect : py.Rect, selected : bool) -> None:
    bg = py.Surface(rect.size, py.SRCALPHA)
    bg.fill(SEL_BG if selected else BOX_BG)
    canvas.blit(bg, rect.topleft)

def Centered(canvas : py.Surface, text : str, cx : int, y : int, color=None) -> None:
    surf = Font1x.Render(text, color)
    canvas.blit(surf, (cx - surf.get_width() // 2, y))

def Draw(canvas : py.Surface, st : CreateState, start, mouse) -> None:
    W, H = canvas.get_size()
    ui.Hot.clear()
    canvas.blit(Font1x.Render("Crear personaje", YELLOW), (8, 3))

    #nombre
    st.name_rect = py.Rect(110, 1, W - 118, 11)
    Box(canvas, st.name_rect, st.focus == "name")
    shown = st.name + ("_" if st.focus == "name" else "")
    canvas.blit(Font1x.Render(f"Nombre: {shown}" if shown else "Nombre: (click para escribir)",
                              None if shown else GREY), (st.name_rect.x + 2, st.name_rect.y + 2))

    #clases
    for i, (key, c) in enumerate(HERO_CLASSES.items()):
        r = py.Rect(8 + i * 77, 15, 73, 34)
        Box(canvas, r, key == st.cls)
        av = Avatar(c)
        canvas.blit(av, (r.x + 4, r.y + (r.h - av.get_height()) // 2))
        canvas.blit(Font1x.Render(t(c["name"])), (r.x + 18, r.y + 6))
        canvas.blit(Font1x.Render(f"{t(c['dnd_class'])} d{c['hit_die']}", GREY), (r.x + 18, r.y + 18))
        ui.Hotspot(r, lambda key=key: setattr(st, "cls", key))

    #caracteristicas
    st.stat_rects = {}
    dragging_stat = st.drag[1] if st.drag and st.drag[0] == "stat" else None
    for i, a in enumerate(dice.ABILITIES):
        r = st.stat_rects[a] = py.Rect(8 + i * 51, 53, 47, 32)
        Box(canvas, r, a == st.focus or (st.drag is not None and r.collidepoint(mouse)))
        Centered(canvas, dice.ABILITY_NAMES[a], r.centerx, r.y + 3, YELLOW)
        v = st.scores[a]
        if a == st.focus:
            Centered(canvas, st.typed or "_", r.centerx, r.y + 13)
        elif v is not None and a != dragging_stat:
            Centered(canvas, str(v), r.centerx, r.y + 13)
        if v is not None and a != dragging_stat:
            Centered(canvas, f"{dice.modifier(v):+d}", r.centerx, r.y + 22, GREY)

    #tiradas
    st.chip_rects = {}
    used = {s for s in st.source.values() if s is not None}
    canvas.blit(Font1x.Render("Tiradas (4d6, se quita el menor) - arrastralas:"), (8, 90))
    for i, (value, rolled) in enumerate(st.pool):
        r = st.chip_rects[i] = py.Rect(8 + i * 30, 100, 26, 16)
        Box(canvas, r, i == st.chip)
        Centered(canvas, str(value), r.centerx, r.y + 4, GREY if i in used else None)
    if st.chip is not None:
        value, rolled = st.pool[st.chip]
        low = rolled.index(min(rolled))
        detail = " ".join(f"[{d}]" if j == low else str(d) for j, d in enumerate(rolled))
        canvas.blit(Font1x.Render(f"{detail} = {value}", GREY), (8 + 7 * 30 + 4, 104))

    help_ = ("Arrastra una tirada a una caracteristica (sobra 1), o entre caracteristicas para cambiarlas. "
             "Click en una caracteristica para escribir el numero (1-20).")
    for j, line in enumerate(Font1x.Wrap(help_, W - 16)):
        canvas.blit(Font1x.Render(line, GREY), (8, 121 + j * 10))

    #botones
    ready = dice.valid_scores(st.scores)
    buttons = [("TIRAR", st.Roll), ("LIMPIAR", st.Clear),
               ("EMPEZAR" if ready else "FALTAN STATS", start if ready else (lambda: None))]
    bw = (W - 16 - 2 * (len(buttons) - 1)) // len(buttons)
    for i, (label, fn) in enumerate(buttons):
        ui.DrawButton(canvas, py.Rect(8 + i * (bw + 2), H - 20, bw, 16), label, fn)

    #lo que se arrastra sigue al mouse
    if st.drag:
        kind, key = st.drag
        value = st.pool[key][0] if kind == "chip" else st.scores[key]
        r = py.Rect(0, 0, 26, 16)
        r.center = mouse
        Box(canvas, r, True)
        Centered(canvas, str(value), r.centerx, r.y + 4)

def Create(Delta) -> None:
    CreateScreen.FrameRate(FrameRate)
    st = CreateState()
    started = []

    def Start():
        name = st.name.strip() or t(HERO_CLASSES[st.cls]["name"])
        char = characters.new_character(name, st.cls, st.scores)
        chars = characters.load()
        chars.append(char)
        characters.save(chars)
        Globals["character"] = char
        NewGame()
        CreateScreen.running = False
        started.append(True) #se entra al juego al salir de este loop (no anidar pantallas)

    while CreateScreen.running:
        mouse = CreateScreen.MousePos()
        for event in py.event.get():
            CreateScreen.EventHandler(event, {})
            if event.type == py.KEYDOWN:
                if event.key == py.K_ESCAPE:
                    CreateScreen.running = False
                else:
                    st.Type(event)
            elif event.type == py.MOUSEBUTTONDOWN and event.button == 1:
                if not st.MouseDown(mouse):
                    ui.Click(mouse)
            elif event.type == py.MOUSEBUTTONUP and event.button == 1:
                st.MouseUp(mouse)
        if not CreateScreen.running:
            break
        CreateScreen.ResetScreen()
        Draw(CreateScreen.canvas, st, Start, mouse)
        CreateScreen.Update()
    CreateScreen.running = True
    if started:
        Game(Delta)
