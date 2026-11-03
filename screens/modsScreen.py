#Pantalla de Mods (reemplaza a Rankings y Badges del original). Por ahora: estado y aviso de seguridad.
#El sistema de mods esta planificado en roadmap/futuro-mods-personajes-red.md.
import pygame as py
from core import Screen, PickResolution, FrameRate, Font1x

#screen define
ModsScreen : Screen = Screen(PickResolution[0], PickResolution[1])

PARAGRAPHS : list[tuple[str, tuple | None]] = [
    ("Mods", (0xFF, 0xFF, 0x44)),
    ("No hay mods instalados. El sistema de mods todavia no esta disponible.", None),
    ("Los mods vienen de fuentes externas: ten cuidado de donde los descargas y revisalos antes de instalarlos.",
     (190, 190, 190)),
    ("Los mods que traen codigo pueden cambiar el juego de forma considerable y hacer cualquier cosa en tu "
     "equipo: instalalos solo si confias en su origen.", (255, 120, 80)),
]
MARGIN : int = 12

def Mods(Delta) -> None:
    ModsScreen.FrameRate(FrameRate)
    width = ModsScreen.w - MARGIN * 2
    lines = []
    for text, color in PARAGRAPHS:
        lines += [(l, color) for l in Font1x.Wrap(text, width)] + [("", None)]
    lines.append(("Esc o click para volver", (136, 136, 136)))

    while ModsScreen.running:
        for event in py.event.get():
            ModsScreen.EventHandler(event, {})
            if (event.type == py.KEYDOWN and event.key == py.K_ESCAPE) or \
               (event.type == py.MOUSEBUTTONUP and event.button == 1):
                ModsScreen.running = False
        if not ModsScreen.running:
            break
        ModsScreen.ResetScreen()
        y = (ModsScreen.h - len(lines) * 10) // 2
        for text, color in lines:
            if text:
                surf = Font1x.Render(text, color)
                ModsScreen.canvas.blit(surf, ((ModsScreen.w - surf.get_width()) // 2, y))
            y += 10
        ModsScreen.Update()
    ModsScreen.running = True
