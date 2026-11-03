#Lista de personajes (boton Play): elegir uno para jugar, crear uno nuevo o borrar.
import pygame as py
from core import Screen, PickResolution, FrameRate, Font1x
from game.lang import t
from game import dice, characters, save
from game.globals import Globals
from game.hero import HERO_CLASSES
from . import gameUi as ui
from .gameScreen import Game, NewGame, Avatar
from .createScreen import Create

#screen define
RosterScreen : Screen = Screen(PickResolution[0], PickResolution[1])

YELLOW, GREY = (0xFF, 0xFF, 0x44), (136, 136, 136)
SEL_BG, ROW_BG = (0x99, 0x88, 0x33, 0xAA), (0x53, 0x56, 0x4D, 0x99)
ROW_H, ROWS_VISIBLE, LIST_Y = 30, 4, 15

def Run(char : dict):
    """Partida de ese personaje si sigue vivo: la de memoria o, la primera vez, la guardada en disco (game/save.py)."""
    runs = Globals.setdefault("runs", {})
    if char["id"] not in runs:
        runs[char["id"]] = save.load_run(char["id"]) #None si no hay: no se vuelve a leer el disco en cada frame
    d = runs[char["id"]]
    return d if d is not None and any(h.is_alive() for h in d.heroes) else None

def Roster(Delta) -> None:
    RosterScreen.FrameRate(FrameRate)
    chars = characters.load()
    state = {"sel": 0 if chars else None, "scroll": 0, "confirm": False, "next": None}

    def Select(i):
        state["sel"], state["confirm"] = i, False

    def Play():
        if state["sel"] is None:
            return
        char = chars[state["sel"]]
        Globals["character"] = char
        run = Run(char)
        if run is not None:
            Globals["dungeon"] = run
        else:
            NewGame()
        state["next"] = Game
        RosterScreen.running = False

    def New():
        state["next"] = Create
        RosterScreen.running = False

    def Delete():
        if state["sel"] is None:
            return
        if not state["confirm"]: #primer click: pedir confirmacion
            state["confirm"] = True
            return
        char = chars.pop(state["sel"])
        characters.save(chars)
        Globals.get("runs", {}).pop(char["id"], None)
        save.delete_run(char["id"])
        state["sel"] = min(state["sel"], len(chars) - 1) if chars else None
        state["confirm"] = False

    while RosterScreen.running:
        for event in py.event.get():
            RosterScreen.EventHandler(event, {})
            if event.type == py.KEYDOWN and event.key == py.K_ESCAPE:
                RosterScreen.running = False
            elif event.type == py.MOUSEWHEEL:
                state["scroll"] = max(0, min(state["scroll"] - event.y, max(0, len(chars) - ROWS_VISIBLE)))
            elif event.type == py.MOUSEBUTTONDOWN and event.button == 1:
                ui.Click(RosterScreen.MousePos())
        if not RosterScreen.running:
            break

        RosterScreen.ResetScreen()
        canvas = RosterScreen.canvas
        W, H = canvas.get_size()
        ui.Hot.clear()
        canvas.blit(Font1x.Render("Personajes", YELLOW), (8, 3))
        if not chars:
            canvas.blit(Font1x.Render("Todavia no hay personajes. Crea uno con NUEVO.", GREY), (8, LIST_Y + 10))
        for row, i in enumerate(range(state["scroll"], min(len(chars), state["scroll"] + ROWS_VISIBLE))):
            c = chars[i]
            cls = HERO_CLASSES.get(c["class"])
            r = py.Rect(8, LIST_Y + row * (ROW_H + 2), W - 16, ROW_H)
            bg = py.Surface(r.size, py.SRCALPHA)
            bg.fill(SEL_BG if i == state["sel"] else ROW_BG)
            canvas.blit(bg, r.topleft)
            if cls is None: #clase de un mod que no esta cargado
                canvas.blit(Font1x.Render(f"{c['name']} - clase desconocida: {c['class']}", GREY), (r.x + 4, r.y + 4))
                continue
            av = Avatar(cls)
            canvas.blit(av, (r.x + 4, r.y + (ROW_H - av.get_height()) // 2))
            run = Run(c)
            canvas.blit(Font1x.Render(c["name"], YELLOW), (r.x + 20, r.y + 4))
            status = f"en partida - piso {run.depth}, nivel {run.hero.lvl}" if run else f"{t(cls['dnd_class'])} d{cls['hit_die']}"
            canvas.blit(Font1x.Render(status, GREY), (r.x + 100, r.y + 4))
            stats = "  ".join(f"{dice.ABILITY_NAMES[a]} {c['abilities'][a]}" for a in dice.ABILITIES)
            canvas.blit(Font1x.Render(stats), (r.x + 20, r.y + 17))
            ui.Hotspot(r, lambda i=i: Select(i))

        has = state["sel"] is not None #sin personajes (o sin elegir) JUGAR y BORRAR quedan grises
        buttons = [("JUGAR", Play, has), ("NUEVO", New, True), ("CONFIRMAR" if state["confirm"] else "BORRAR", Delete, has)]
        bw = (W - 16 - 2 * (len(buttons) - 1)) // len(buttons)
        for i, (label, fn, enabled) in enumerate(buttons):
            ui.DrawButton(canvas, py.Rect(8 + i * (bw + 2), H - 20, bw, 16), label, fn, enabled)
        RosterScreen.Update()
    RosterScreen.running = True
    if state["next"]: #la pantalla siguiente se abre al salir de este loop (no anidar)
        state["next"](Delta)
