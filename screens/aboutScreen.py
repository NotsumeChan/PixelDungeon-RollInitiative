#Pantalla About: que es el proyecto, creditos del original y aviso legal (version propia de AboutScene.java).
import pygame as py
from core import Screen, PickResolution, FrameRate, Version, Font1x

#screen define
AboutScreen : Screen = Screen(PickResolution[0], PickResolution[1])

TITLE : str = f"PixelDND v{Version}"
PARAGRAPHS : list[str] = [
    "Extensión del juego base Pixel Dungeon, orientada a una experiencia customizada sobre los sistemas "
    "de Dungeons & Dragons 5e.",
    "Basado en Pixel Dungeon de Watabou: código y gráficos originales bajo licencia GNU GPL v3. "
    "Música original: Cube_Code.",
    "Proyecto de fans sin fines de lucro. No está afiliado ni respaldado por Watabou ni por Wizards of the Coast. "
    "Dungeons & Dragons es una marca de Wizards of the Coast.",
]
MARGIN : int = 12

def About(Delta) -> None:
    AboutScreen.FrameRate(FrameRate)
    width = AboutScreen.w - MARGIN * 2
    lines = [(TITLE, (0xFF, 0xFF, 0x44))]
    for p in PARAGRAPHS:
        lines += [(l, None) for l in Font1x.Wrap(p, width)] + [("", None)]
    lines.append(("Esc o click para volver", (136, 136, 136)))

    while AboutScreen.running:
        for event in py.event.get():
            AboutScreen.EventHandler(event, {})
            if (event.type == py.KEYDOWN and event.key == py.K_ESCAPE) or \
               (event.type == py.MOUSEBUTTONUP and event.button == 1):
                AboutScreen.running = False
        if not AboutScreen.running:
            break
        AboutScreen.ResetScreen()
        y = (AboutScreen.h - len(lines) * 9) // 2
        for text, color in lines:
            if text:
                surf = Font1x.Render(text, color)
                AboutScreen.canvas.blit(surf, ((AboutScreen.w - surf.get_width()) // 2, y))
            y += 9
        AboutScreen.Update()
    AboutScreen.running = True
