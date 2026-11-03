#Pantalla de titulo: port de TitleScene.java (arcos de fondo, banner, letreros que laten, dashboard)
import math
import pygame as py
from core import Screen, Button, PickResolution, FrameRate, Version, Font1x
from .rosterScreen import Roster
from .modsScreen import Mods
from .aboutScreen import About
from . import gameUi as ui
from game import lang
from game.lang import t

#screen define
MainScreen : Screen = Screen(PickResolution[0], PickResolution[1])
W, H = MainScreen.w, MainScreen.h

#Assets originales
ArcsBg  : py.Surface = py.image.load("Assets/arcs1.png").convert_alpha()
ArcsFg  : py.Surface = py.image.load("Assets/arcs2.png").convert_alpha()
Banners : py.Surface = py.image.load("Assets/banners.png").convert_alpha()
Dashboard : py.Surface = py.image.load("Assets/dashboard.png").convert_alpha()

Title : py.Surface = Banners.subsurface((0, 0, 128, 70))         #BannerSprites.PIXEL_DUNGEON
Signs : py.Surface = Banners.subsurface((0, 161, 128, 57)).premul_alpha() #PIXEL_DUNGEON_SIGNS (se suma, blend aditivo)

#Layout apaisado de TitleScene
ITEM_SIZE : int = 48 #DashboardItem.SIZE
LayoutH : int = Title.get_height() + ITEM_SIZE
TitlePos : tuple = ((W - Title.get_width()) // 2, (H - LayoutH) // 2)
ItemsY : int = (H + LayoutH) // 2 - ITEM_SIZE

def DashboardItem(index : int, text : str, x : int, action) -> Button:
    icon = Dashboard.subsurface((index * 32, 0, 32, 32))
    return Button(py.Rect(x, ItemsY, ITEM_SIZE, ITEM_SIZE), icon, action, Font1x.Render(text))

Settings : dict = {"open": False} #ventana de configuracion (WndSettings) abierta sobre el titulo

def BuildButtons() -> dict[str, Button]:
    """Rankings y Badges del original se reemplazaron por Mods. Se rearma si cambia el idioma."""
    return {
        "play"  : DashboardItem(0, t({"es": "Jugar", "en": "Play"}),      W // 2 - ITEM_SIZE * 3 // 2, Roster),
        "mods"  : DashboardItem(2, "Mods",                                W // 2 - ITEM_SIZE // 2,     Mods),
        "about" : DashboardItem(1, t({"es": "Acerca de", "en": "About"}), W // 2 + ITEM_SIZE // 2,     About),
        #PrefsButton de TitleScene: icono Icons.PREFS arriba a la izquierda
        "prefs" : Button(py.Rect(1, 1, 16, 16), ui.IconImg((30, 0, 16, 16)), lambda _: Settings.update(open=True)),
    }

MainButtons : dict[str, Button] = BuildButtons()

VersionText : py.Surface = Font1x.Render(f"v {Version}", (136, 136, 136))

def DrawTiled(canvas : py.Surface, tex : py.Surface, offY : float) -> None:
    """SkinnedBlock de Archs: textura repetida, centrada en X y desplazada en Y."""
    tw, th = tex.get_size()
    offX = tw // 4 - (W % tw) // 2
    for y in range(-int(offY % th), H, th):
        for x in range(-(offX % tw), W, tw):
            canvas.blit(tex, (x, y))

def HomeScreen(Delta) -> None:
    MainScreen.FrameRate(FrameRate)
    offB = offF = elapsed = 0.0
    builtLang = lang.LANG
    while MainScreen.running:
        dt = MainScreen.dt
        #event handler (con la configuracion abierta, los clicks van a la ventana)
        for event in py.event.get():
            MainScreen.EventHandler(event, {} if Settings["open"] else MainButtons)
            if Settings["open"]:
                if event.type == py.MOUSEBUTTONDOWN and event.button == 1:
                    ui.Click(MainScreen.MousePos())
                elif event.type == py.KEYDOWN and event.key == py.K_ESCAPE:
                    Settings["open"] = False
        if builtLang != lang.LANG: #cambio el idioma: rearmar los textos
            MainButtons.update(BuildButtons())
            builtLang = lang.LANG

        #window breaker
        if not MainScreen.running:
            break

        #arcos: el de adelante va al doble de velocidad (Archs.SCROLL_SPEED = 20)
        offB += dt * 20
        offF += dt * 40
        elapsed += dt

        canvas = MainScreen.canvas
        DrawTiled(canvas, ArcsBg, offB)
        DrawTiled(canvas, ArcsFg, offF)
        canvas.blit(Title, TitlePos)

        #letreros: alpha = sin(-t), negativo = invisible
        k = int(255 * max(0.0, math.sin(-elapsed)))
        if k:
            glow = Signs.copy()
            glow.fill((k, k, k), special_flags=py.BLEND_RGB_MULT)
            canvas.blit(glow, TitlePos, special_flags=py.BLEND_RGB_ADD)

        for key, item in MainButtons.items():
            item.Draw(MainScreen)
        canvas.blit(VersionText, (W - VersionText.get_width(), H - VersionText.get_height()))
        if Settings["open"]:
            ui.Hot.clear()
            ui.DrawSettings(canvas, lambda: Settings.update(open=False))

        #screen updater
        MainScreen.Update()
    MainScreen.running = True
