#UI de la pantalla de juego con los assets originales: StatusPane, Toolbar, WndBag, WndItem.
#UI "inmediata": cada frame se dibuja todo y se registran zonas clickeables (Hot); el click busca de arriba hacia abajo.
import pygame as py
from core import Font1x
from game.items import (Item, TARGETS, SLOT_WEAPON, SLOT_ARMOR, SLOT_RING, item_name, item_image, item_info,
                        ITEMS)
from game.hero import BACKPACK_SIZE, HUNGRY, STARVING
from game import dice, spells
from game import lang
from game.lang import t
import importlib
ScreenMod = importlib.import_module("core.Screen") #el modulo (core.Screen como atributo es la clase)
ConfigMod = importlib.import_module("core.Config")
from game.items import weapon_dice
from . import charSprite as cs

_Chrome  : py.Surface = py.image.load("Assets/chrome.png").convert_alpha()
_Items   : py.Surface = py.image.load("Assets/Items/items.png").convert_alpha()
_Status  : py.Surface = py.image.load("Assets/status_pane.png").convert_alpha()
_Toolbar : py.Surface = py.image.load("Assets/toolbar.png").convert_alpha()
_HpBar   : py.Surface = py.image.load("Assets/hp_bar.png").convert_alpha()
_ExpBar  : py.Surface = py.image.load("Assets/exp_bar.png").convert_alpha()
_Icons   : py.Surface = py.image.load("Assets/icons.png").convert_alpha()
_Buffs   : py.Surface = py.image.load("Assets/buffs.png").convert_alpha()

#Icons.java: recortes de icons.png (x, y, ancho, alto)
ICON_BUSY  : tuple = (8, 0, 8, 8)
ICON_SLEEP : tuple = (13, 45, 9, 8)
ICON_ALERT : tuple = (22, 45, 8, 8)
BUSY_SPEED : float = 720 #grados por segundo (BusyIndicator.angularSpeed)
#BuffIndicator.java: iconos de 7x7 en buffs.png; hambre y famelico son los indices 5 y 6
BUFF_SIZE, BUFF_HUNGER, BUFF_STARVATION = 7, 5, 6
#WndGame.java
MENU_W, MENU_BTN_H, MENU_GAP = 120, 20, 2
INFO_W : int = 120 #WndInfoCell / WndInfoMob
SETTINGS_W : int = 170 #WndSettings
SPELLS_W : int = 190
GREY : tuple = (0x88, 0x88, 0x88)
FPS_OPTIONS : list = [30, 60, 120, 0] #0 = sin limite (clock.tick(0))
#escala de la interfaz en partida: factor ENTERO de pixeles de pantalla por pixel de UI (nitido), del zoom base hacia
#abajo (75% y 50% en HD). ponytail: no hay >100%: la UI esta pensada para 320x180 y las ventanas no entrarian
UI_SCALES : list = [s / ScreenMod.Zoom for s in range(max(1, ScreenMod.Zoom - 2), ScreenMod.Zoom + 1)]
SELECTED : tuple = (0xFF, 0xFF, 0x44)
LINE_H : int = 10

#Chrome.java: (recorte en chrome.png, margen del nine-patch)
CHROME_WINDOW : tuple = ((0, 0, 22, 22), 7)
CHROME_BUTTON : tuple = ((58, 0, 6, 6), 2)
#Toolbar.java: recortes de toolbar.png
TOOL_WAIT, TOOL_SEARCH, TOOL_INFO = (0, 7, 20, 25), (20, 7, 20, 25), (40, 7, 21, 25)
TOOL_BAG, TOOL_QUICK = (60, 7, 23, 25), (83, 7, 22, 25)
TOOLBAR_H : int = 25
#Mochila: 1a fila = equipo (arma, armadura, 2 anillos) + oro aparte a la derecha; debajo, las 19 de mochila
SLOT_SIZE, SLOT_MARGIN, BAG_COLS, BAG_TITLE_H, BAG_GAP = 28, 1, 7, 12, 4
BAG_ROWS : int = 1 + -(-BACKPACK_SIZE // BAG_COLS) #fila de equipo + filas de mochila
SLOT_BG : tuple = (0x53, 0x56, 0x4D, 0x99)
ITEM_WND_W : int = 170

_Cache : dict = {}

#carrusel de iniciativa (combate 5e): lado de cada casilla, cuantos retratos, velocidad del deslizamiento (casillas/s)
CAROUSEL_SLOT, CAROUSEL_MAX, CAROUSEL_SPEED, CAROUSEL_Y = 16, 7, 8.0, 30
HERO_BOX, MOB_BOX = (0x22, 0x44, 0x66, 0xCC), (0x55, 0x22, 0x22, 0xCC)
_Carousel : dict = {"cur": None, "off": 0.0, "t": 0}
CENTER_BTN : tuple = (2, 34, 14, 14) #boton de centrar la camara (multijugador), bajo el avatar. ponytail: a ojo

class UiState():
    """window: None | ("bag", select) | ("item", item) | ("hero",) | ("menu",) | ("settings",) | ("rules",) |
    ("info", icono, titulo, texto, vida) | ("spells",).
    selecting = None | "examine" | ("spell", id) | ("throw", item): el proximo click en el mapa elige esa casilla.
    select = None | ("target", pergamino, filtro) al elegir objetivo | ("quick", i) al elegir el acceso rapido i."""
    def __init__(self) -> None:
        self.window = None
        self.mouse = None     #posicion del mouse en el canvas (la pone la pantalla cada frame): tooltips
        self.selecting = None #"examine": el proximo click en el mapa examina esa casilla (CellSelector)
        self.center = False   #boton de centrar (multijugador): la pantalla vuelve la camara a tu heroe

Hot : list = [] #(Rect, callback) registrados en este frame

def Hotspot(rect, fn, alt=None, tip : str | None = None) -> None:
    """alt = accion del click derecho (None = el mismo que el izquierdo). tip = texto al pasar el mouse."""
    Hot.append((py.Rect(rect), fn, alt, tip))

def Click(pos, alt : bool = False) -> bool:
    for rect, fn, altfn, _ in reversed(Hot):
        if rect.collidepoint(pos):
            (altfn if alt and altfn else fn)()
            return True
    return False

#--- dibujo basico ---

def TextIcon(name : str) -> py.Surface:
    """Sprite que falta (o que no carga): el nombre, una palabra por linea, achicado para entrar en 16x16."""
    lines = [Font1x.Render(w, (0xFF, 0xFF, 0xFF)) for w in str(name).split()] or [Font1x.Render("?", GREY)]
    w, h = max(l.get_width() for l in lines), sum(l.get_height() for l in lines)
    text = py.Surface((w, h), py.SRCALPHA)
    y = 0
    for l in lines:
        text.blit(l, ((w - l.get_width()) // 2, y))
        y += l.get_height()
    f = min(1.0, 16 / w, 16 / h)
    if f < 1:
        text = py.transform.scale(text, (max(1, int(w * f)), max(1, int(h * f))))
    out = py.Surface((16, 16), py.SRCALPHA)
    out.blit(text, text.get_rect(center=(8, 8)))
    return out

def ItemIcon(index) -> py.Surface:
    """items.png: grilla de 8 columnas de 16x16 (ItemSpriteSheet). index = entero (items.png del original),
    (archivo, indice[, nombre]) para hojas propias o de mods (celdas de 16x16), o ("text", nombre) sin sprite.
    Si la hoja no carga o el indice no existe, se dibuja el nombre (TextIcon)."""
    if not isinstance(index, int):
        index = tuple(index) #en los JSON es una lista
    key = ("icon", index)
    if key not in _Cache:
        if isinstance(index, int):
            _Cache[key] = _Items.subsurface(((index % 8) * 16, (index // 8) * 16, 16, 16))
        elif index[0] == "text":
            _Cache[key] = TextIcon(index[1])
        else:
            file, i, *name = index
            try:
                sheet = _Cache.get(("sheet", file))
                if sheet is None:
                    sheet = _Cache[("sheet", file)] = py.image.load(file).convert_alpha()
                cols = max(1, sheet.get_width() // 16)
                _Cache[key] = sheet.subsurface(((i % cols) * 16, (i // cols) * 16, 16, 16))
            except (OSError, ValueError, py.error): #archivo que falta, roto o indice fuera de la hoja
                _Cache[key] = TextIcon(name[0] if name else "?")
    return _Cache[key]

def IconImg(region : tuple) -> py.Surface:
    key = ("icons", region)
    if key not in _Cache:
        _Cache[key] = _Icons.subsurface(region)
    return _Cache[key]

def BuffIcon(index : int) -> py.Surface:
    key = ("buff", index)
    if key not in _Cache:
        cols = _Buffs.get_width() // BUFF_SIZE
        _Cache[key] = _Buffs.subsurface(((index % cols) * BUFF_SIZE, (index // cols) * BUFF_SIZE, BUFF_SIZE, BUFF_SIZE))
    return _Cache[key]

def NinePatch(src : py.Surface, region : tuple, m : int, w : int, h : int) -> py.Surface:
    """Estira region a w x h conservando las esquinas de m pixeles (NinePatch de noosa)."""
    key = ("9", id(src), region, m, w, h)
    if key not in _Cache:
        sx, sy, sw, sh = region
        out = py.Surface((w, h), py.SRCALPHA)
        cols = [(0, m, 0, m), (m, sw - 2 * m, m, w - 2 * m), (sw - m, m, w - m, m)]
        rows = [(0, m, 0, m), (m, sh - 2 * m, m, h - 2 * m), (sh - m, m, h - m, m)]
        for cx, cw, dx, dw in cols:
            for cy, ch, dy, dh in rows:
                if cw > 0 and ch > 0 and dw > 0 and dh > 0:
                    piece = src.subsurface((sx + cx, sy + cy, cw, ch))
                    out.blit(py.transform.scale(piece, (dw, dh)), (dx, dy))
        _Cache[key] = out
    return _Cache[key]

def DrawWindow(canvas : py.Surface, content : py.Rect) -> None:
    (region, m) = CHROME_WINDOW
    canvas.blit(NinePatch(_Chrome, region, m, content.w + 2 * m, content.h + 2 * m), (content.x - m, content.y - m))

def DrawButton(canvas : py.Surface, rect : py.Rect, label : str, fn, enabled : bool = True) -> None:
    """enabled=False: boton gris y bloqueado (no registra click)."""
    (region, m) = CHROME_BUTTON
    bg = NinePatch(_Chrome, region, m, rect.w, rect.h)
    if not enabled:
        bg = bg.copy()
        bg.set_alpha(110)
    canvas.blit(bg, rect.topleft)
    text = Font1x.Render(label, None if enabled else (110, 110, 110))
    canvas.blit(text, (rect.centerx - text.get_width() // 2, rect.centery - text.get_height() // 2))
    if enabled:
        Hotspot(rect, fn)

#--- StatusPane ---

def DrawStatus(canvas : py.Surface, d, avatar : py.Surface, ui : UiState) -> None:
    W = canvas.get_width()
    h = d.hero
    #NinePatch(Assets.STATUS, 80, 0, 48, 0) sobre 128 px: el centro mide 0, asi que se estira la columna 80
    pane = py.Surface((W, _Status.get_height()), py.SRCALPHA)
    pane.blit(_Status.subsurface((0, 0, 80, 64)), (0, 0))
    pane.blit(py.transform.scale(_Status.subsurface((80, 0, 1, 64)), (max(W - 128, 0), 64)), (80, 0))
    pane.blit(_Status.subsurface((80, 0, 48, 64)), (W - 48, 0))
    canvas.blit(pane, (0, 0))
    canvas.blit(avatar, (15 - avatar.get_width() // 2, 16 - avatar.get_height() // 2))
    Hotspot((0, 1, 30, 30), lambda: setattr(ui, "window", ("hero",)), #TouchArea del avatar -> ficha
            tip=t({"es": "Ficha del personaje", "en": "Character sheet"}))
    #barra de vida (30, 3) y de experiencia (arriba, a lo ancho)
    hpw = max(0, int(_HpBar.get_width() * h.HP / h.HT))
    if hpw:
        canvas.blit(_HpBar.subsurface((0, 0, hpw, _HpBar.get_height())), (30, 3))
    expw = int(W * h.exp / h.max_exp())
    if expw:
        canvas.blit(py.transform.scale(_ExpBar, (expw, _ExpBar.get_height())), (0, 0))
    lvl = Font1x.Render(str(h.lvl), (0xFF, 0xEB, 0xA4))
    canvas.blit(lvl, (28 - lvl.get_width() // 2, 25))
    depth = Font1x.Render(str(d.depth), (0xCA, 0xCF, 0xC2))
    canvas.blit(depth, (W - 24 - depth.get_width() - 18, 6))
    #BuffIndicator en (32, 11): hambre + estados del heroe (icon de BUFFS)
    icons = [BUFF_STARVATION] if h.hunger >= STARVING else [BUFF_HUNGER] if h.hunger >= HUNGRY else []
    icons += [b.data["icon"] for b in h.buffs if b.data.get("icon", -1) >= 0]
    for i, icon in enumerate(icons):
        canvas.blit(BuffIcon(icon), (32 + i * (BUFF_SIZE + 2), 11))
    s = d.scheduler
    DrawInitiative(canvas, d)
    if len(d.heroes) > 1: #multijugador: centrar la camara en tu heroe (tecla C)
        r = py.Rect(CENTER_BTN)
        canvas.blit(NinePatch(_Chrome, CHROME_BUTTON[0], CHROME_BUTTON[1], r.w, r.h), r)
        cx, cy = r.center
        white = (0xFF, 0xFF, 0xFF)
        py.draw.circle(canvas, white, (cx, cy), 3, 1) #mira
        py.draw.line(canvas, white, (cx, r.top + 2), (cx, r.bottom - 3))
        py.draw.line(canvas, white, (r.left + 2, cy), (r.right - 3, cy))
        Hotspot(r, lambda: setattr(ui, "center", True), tip=t({"es": "Centrar en tu heroe (C)", "en": "Center on your hero (C)"}))
    #boton de menu: recorte (114, 3, 12, 11) de status_pane.png -> WndGame
    canvas.blit(_Status.subsurface((114, 3, 12, 11)), (W - 16 + 2, 1 + 2))
    Hotspot((W - 16, 1, 16, 16), lambda: setattr(ui, "window", ("menu",)), tip=t({"es": "Menu (Esc)", "en": "Menu (Esc)"}))
    #BusyIndicator: gira mientras el heroe no espera input. En el original va en (1, 1); aca, debajo del menu
    if h.is_alive() and (h.action is not None or h.animating or d.pending or h.has_flag("paralysed")
                         or (s.order is not None and s.current() is not h)):
        img = py.transform.rotate(IconImg(ICON_BUSY), -(py.time.get_ticks() / 1000 * BUSY_SPEED) % 360)
        canvas.blit(img, (W - 8 - img.get_width() // 2, 22 - img.get_height() // 2))

def DrawInitiative(canvas : py.Surface, d) -> None:
    """Carrusel de iniciativa (combate 5e), arriba a la derecha bajo piso/menu: el primero (borde amarillo) es al que
    le toca, despues los siguientes en orden. Al pasar el turno se desliza. Lo que no se ve sale como "?".
    Debajo: ronda, y en tu turno los pies de movimiento que quedan y si te queda la accion."""
    s = d.scheduler
    if s.order is None:
        _Carousel["cur"] = None
        return
    order = [a for a in s.order if a in s.actors]
    if not order:
        return
    cur = s.current()
    i = order.index(cur) if cur in order else 0
    ring = (order[i:] + order[:i])[:CAROUSEL_MAX]
    now = py.time.get_ticks() #dt real: el deslizamiento dura lo mismo a cualquier FPS
    dt = min((now - _Carousel["t"]) / 1000, 0.1)
    _Carousel["t"] = now
    if cur is not _Carousel["cur"]:
        if _Carousel["cur"] is not None:
            _Carousel["off"] = 1.0 #arranca corrido una casilla y vuelve a su lugar
        _Carousel["cur"] = cur
    _Carousel["off"] = max(0.0, _Carousel["off"] - CAROUSEL_SPEED * dt)
    S, W = CAROUSEL_SLOT, canvas.get_width()
    x0 = W - 2 - len(ring) * S
    for k, a in enumerate(ring):
        r = py.Rect(x0 + int((k + _Carousel["off"]) * S), CAROUSEL_Y, S - 1, S - 1)
        box = py.Surface(r.size, py.SRCALPHA)
        box.fill(HERO_BOX if a in d.heroes else MOB_BOX)
        canvas.blit(box, r)
        seen = d.level.visible[a.pos]
        img = cs.Frame(a, 0) if seen else Font1x.Render("?", GREY)
        f = min(1.0, (S - 3) / max(img.get_size()))
        if f < 1:
            img = py.transform.scale(img, (max(1, int(img.get_width() * f)), max(1, int(img.get_height() * f))))
        canvas.blit(img, img.get_rect(center=r.center))
        if seen:
            ini = Font1x.Render(str(a.init), (0xFF, 0xFF, 0xFF))
            canvas.blit(ini, (r.right - ini.get_width(), r.bottom - ini.get_height()))
            Hotspot(r, lambda: None, tip=f"{a.name}: {t({'es': 'iniciativa', 'en': 'initiative'})} {a.init}")
        if k == 0:
            py.draw.rect(canvas, SELECTED, r, 1)
    h = d.hero
    if cur is h:
        txt = t({"es": "Ronda {r} - {ft} ft", "en": "Round {r} - {ft} ft"}).format(r=s.round, ft=h.moves * 5)
        txt += t({"es": " - accion", "en": " - action"}) if h.has_action else ""
    else:
        txt = t({"es": "Ronda {r}", "en": "Round {r}"}).format(r=s.round)
    lbl = Font1x.Render(txt, (0xFF, 0xEB, 0xA4))
    canvas.blit(lbl, (W - 2 - lbl.get_width(), CAROUSEL_Y + S + 1))

#--- Toolbar ---

def DrawToolbar(canvas : py.Surface, d, ui : UiState) -> None:
    W, H = canvas.get_size()
    y = H - TOOLBAR_H
    hero = d.hero
    def act(action):
        def fn():
            if hero.is_alive():
                hero.action = action
        return fn
    def toggleBag():
        ui.window = None if ui.window else ("bag", None)
    tools = [(TOOL_WAIT, 0, act(("rest",)), None, t({"es": "Esperar un turno (Espacio)", "en": "Wait a turn (Space)"})),
             (TOOL_SEARCH, 20, act(("search",)), None, t({"es": "Buscar (Q)", "en": "Search (Q)"})),
             (TOOL_INFO, 40, lambda: setattr(ui, "selecting", None if ui.selecting else "examine"), None,
              t({"es": "Examinar una casilla", "en": "Examine a cell"})),
             (TOOL_BAG, W - 44 - 23, toggleBag, None, t({"es": "Mochila (I)", "en": "Backpack (I)"}))]
    #2 ranuras de acceso rapido (Toolbar.btnQuick1/2), de izquierda a derecha = teclas 1 y 2.
    #click = usar (o elegir si esta vacia), click derecho = elegir
    for i, x in QUICK_X(W):
        item = hero.quick_item(i)
        what = item_name(item, d) if item else t({"es": "vacio, click derecho para elegir", "en": "empty, right click to choose"})
        tools.append((TOOL_QUICK, x, lambda i=i: UseQuick(d, ui, i), lambda i=i: setattr(ui, "window", ("bag", ("quick", i))),
                      f"{t({'es': 'Acceso rapido', 'en': 'Quick slot'})} {i + 1}: {what} ({i + 1})"))
    for region, x, fn, alt, tip in tools:
        canvas.blit(_Toolbar.subsurface(region), (x, y))
        Hotspot((x, y, region[2], region[3]), fn, alt, tip)
    for i, x in QUICK_X(W):
        item = hero.quick_item(i)
        if item is not None:
            canvas.blit(ItemIcon(item_image(item, d)), (x + 3, y + 5))
            if item.quantity > 1:
                canvas.blit(Font1x.Render(str(item.quantity)), (x + 2, y + 3))

def QUICK_X(W : int) -> tuple:
    """(ranura, x) de los accesos rapidos: el 1 a la izquierda, el 2 a la derecha."""
    return ((0, W - 44), (1, W - 22))

def DrawTooltip(canvas : py.Surface, ui : UiState) -> None:
    """Texto del boton bajo el mouse (el de mas arriba). Encima del mouse, sin salirse de la pantalla."""
    if ui.mouse is None:
        return
    tip = next((tip for rect, _, _, tip in reversed(Hot) if rect.collidepoint(ui.mouse)), None)
    if not tip:
        return
    img = Font1x.Render(tip)
    W, H = canvas.get_size()
    box = py.Rect(0, 0, img.get_width() + 6, img.get_height() + 4)
    box.midbottom = (ui.mouse[0], ui.mouse[1] - 4)
    box.clamp_ip(canvas.get_rect())
    bg = py.Surface(box.size, py.SRCALPHA)
    bg.fill((0, 0, 0, 200))
    bg.blit(img, (3, 2))
    canvas.blit(bg, box.topleft)

def UseQuick(d, ui : UiState, i : int) -> None:
    """Usa el item del acceso rapido i con su accion principal (la primera de Verbs)."""
    hero = d.hero
    item = hero.quick_item(i)
    if item is None:
        ui.window = ("bag", ("quick", i))
        return
    if not hero.is_alive():
        return
    label, verb = Verbs(item, hero)[0]
    need = item.data.get("target")
    if verb == "throw": #elegir donde
        ui.window, ui.selecting = None, ("throw", item)
    elif verb == "read" and need:
        ui.window = ("bag", ("target", item, need))
    else:
        hero.use(item, verb)

#--- ventanas ---

def DrawSlot(canvas : py.Surface, rect : py.Rect, item : Item | None, d, placeholder : int = -1,
             enabled : bool = True) -> None:
    bg = py.Surface(rect.size, py.SRCALPHA)
    bg.fill(SLOT_BG)
    canvas.blit(bg, rect.topleft)
    if item is None:
        if placeholder >= 0:
            icon = ItemIcon(placeholder).copy()
            icon.set_alpha(80)
            canvas.blit(icon, (rect.centerx - 8, rect.centery - 8))
        return
    icon = ItemIcon(item_image(item, d))
    if not enabled:
        icon = icon.copy()
        icon.set_alpha(70)
    canvas.blit(icon, (rect.centerx - 8, rect.centery - 8))
    if item.quantity > 1:
        canvas.blit(Font1x.Render(str(item.quantity)), (rect.x + 1, rect.y + 1))
    if item.is_equipment() and item.level_known and item.level:
        lv = Font1x.Render(f"{item.level:+d}", (80, 255, 80) if item.level > 0 else (255, 80, 80))
        canvas.blit(lv, (rect.right - lv.get_width(), rect.bottom - lv.get_height()))

def DrawBag(canvas : py.Surface, d, ui : UiState) -> None:
    W, H = canvas.get_size()
    hero = d.hero
    select = ui.window[1]
    cw = SLOT_SIZE * BAG_COLS + SLOT_MARGIN * (BAG_COLS - 1)
    ch = BAG_TITLE_H + SLOT_SIZE * BAG_ROWS + SLOT_MARGIN * (BAG_ROWS - 1) + BAG_GAP
    content = py.Rect((W - cw) // 2, (H - TOOLBAR_H - ch) // 2, cw, ch)
    Hotspot(canvas.get_rect(), lambda: setattr(ui, "window", None)) #click afuera = cerrar
    DrawWindow(canvas, content)
    Hotspot(content, lambda: None) #click en el fondo de la ventana no la cierra
    title = "Elige un objeto" if select and select[0] == "target" else \
            f"Acceso rapido {select[1] + 1}: elige un objeto" if select else "Mochila"
    canvas.blit(Font1x.Render(title, (0xFF, 0xFF, 0x44)), (content.x, content.y + 2))

    def cell(col, row):
        gap = BAG_GAP if row > 0 else 0 #separacion entre la fila de equipo y la mochila
        return py.Rect(content.x + col * (SLOT_SIZE + SLOT_MARGIN),
                       content.y + BAG_TITLE_H + row * (SLOT_SIZE + SLOT_MARGIN) + gap, SLOT_SIZE, SLOT_SIZE)

    def eligible(item):
        if select is None or select[0] == "quick":
            return True
        _, scroll, need = select
        return item is not scroll and item.id != scroll.id and TARGETS[need](item, d) #identify no se elige a si mismo

    def onItem(item):
        def fn():
            if select and select[0] == "quick": #guardar en el acceso rapido (apilables por id)
                hero.quickslots[select[1]] = item.id if item.stackable() else item
                ui.window = None
            elif select:
                if eligible(item):
                    hero.use(select[1], "read", item)
                    ui.window = None
            else:
                ui.window = ("item", item)
        return fn

    def slot(r, item, placeholder=-1):
        DrawSlot(canvas, r, item, d, placeholder, item is not None and eligible(item))
        if item is not None:
            Hotspot(r, onItem(item))

    #fila 0: equipo
    for col, (item, placeholder) in enumerate([(hero.weapon, SLOT_WEAPON), (hero.armor, SLOT_ARMOR),
                                                (None, SLOT_RING), (None, SLOT_RING)]):
        slot(cell(col, 0), item, placeholder)
    #oro: ranura aparte, al final de la fila de equipo
    r = cell(BAG_COLS - 1, 0)
    DrawSlot(canvas, r, Item("pddnd:gold", max(hero.gold, 1)) if hero.gold else None, d, ITEMS["pddnd:gold"]["image"])
    #mochila: exactamente BACKPACK_SIZE ranuras
    for i in range(BACKPACK_SIZE):
        slot(cell(i % BAG_COLS, 1 + i // BAG_COLS), hero.backpack[i] if i < len(hero.backpack) else None)

def Verbs(item : Item, hero) -> list[tuple[str, str]]:
    cat = item.category
    verbs = []
    if item.is_equipment():
        verbs.append(("QUITAR", "unequip") if item in (hero.weapon, hero.armor) else ("EQUIPAR", "equip"))
    elif cat == "food":
        verbs.append(("COMER", "eat"))
    elif cat == "potion":
        verbs.append(("BEBER", "drink"))
    elif cat == "scroll":
        verbs.append(("LEER", "read"))
    elif cat == "missile" and item.data.get("thrown"):
        verbs.append(("LANZAR", "throw"))
    verbs.append(("SOLTAR", "drop"))
    return verbs

def DrawItemWindow(canvas : py.Surface, d, ui : UiState) -> None:
    W, H = canvas.get_size()
    hero = d.hero
    item : Item = ui.window[1]
    if item not in hero.all_items(): #se gasto o se solto
        ui.window = None
        return
    nameLines = Font1x.Wrap(item_name(item, d), ITEM_WND_W - 20)
    infoLines = Font1x.Wrap(item_info(item, d, hero), ITEM_WND_W)
    btnH = 16
    ch = max(16, len(nameLines) * 9) + 4 + len(infoLines) * 9 + 4 + btnH
    content = py.Rect((W - ITEM_WND_W) // 2, (H - TOOLBAR_H - ch) // 2, ITEM_WND_W, ch)
    Hotspot(canvas.get_rect(), lambda: setattr(ui, "window", None))
    DrawWindow(canvas, content)
    Hotspot(content, lambda: None)
    canvas.blit(ItemIcon(item_image(item, d)), content.topleft)
    for i, line in enumerate(nameLines):
        canvas.blit(Font1x.Render(line, (0xFF, 0xFF, 0x44)), (content.x + 20, content.y + 4 + i * 9))
    y = content.y + max(16, len(nameLines) * 9) + 4
    for i, line in enumerate(infoLines):
        canvas.blit(Font1x.Render(line), (content.x, y + i * 9))
    verbs = Verbs(item, hero)
    bw = (ITEM_WND_W - (len(verbs) - 1) * 2) // len(verbs)
    for i, (label, verb) in enumerate(verbs):
        def fn(verb=verb):
            need = item.data.get("target")
            if verb == "throw": #primero elegir donde
                ui.window, ui.selecting = None, ("throw", item)
            elif verb == "read" and need:
                ui.window = ("bag", ("target", item, need)) #primero elegir el objetivo
            else:
                hero.use(item, verb)
                ui.window = None
        DrawButton(canvas, py.Rect(content.x + i * (bw + 2), content.bottom - btnH, bw, btnH), label, fn)

def DrawHeroWindow(canvas : py.Surface, d, ui : UiState) -> None:
    """Ficha 5e del personaje (WndHero del original, con los numeros de D&D)."""
    W, H = canvas.get_size()
    h = d.hero
    w = h.weapon
    lines = [(f"{h.name} ({t(h.cls['dnd_class'])}) nivel {h.lvl}", (0xFF, 0xFF, 0x44)),
             ("   ".join(f"{dice.ABILITY_NAMES[a]} {h.abilities[a]}({h.mod(a):+d})" for a in dice.ABILITIES[:3]), None),
             ("   ".join(f"{dice.ABILITY_NAMES[a]} {h.abilities[a]}({h.mod(a):+d})" for a in dice.ABILITIES[3:]), None),
             (f"PV {h.HP}/{h.HT}   CA {h.armor_class()}   Competencia +{h.proficiency()}", None),
             (f"Ataque {h.attack_bonus(None):+d}   Daño {weapon_dice(w) if w else '1'}{h._attack_mod():+d}"
              f"   Dado de golpe d{h.cls['hit_die']}", None),
             ("Salvaciones " + " ".join(f"{dice.ABILITY_NAMES[a]}{h.save_bonus(a):+d}" for a in dice.ABILITIES), None),
             (f"XP {h.exp}/{h.max_exp()}   Oro {h.gold}", None)]
    if h.has_trait("pddnd:second_wind"):
        lines.append((f"Segundo Aliento (F): {'disponible' if h.second_wind else 'usado'}", None))
    if h.has_trait("pddnd:sneak_attack"):
        lines.append((f"Ataque Furtivo: +{h.sneak_dice()}d6 con ventaja y arma sutil", None))
    if spells.spellcasting(h):
        lines.append((f"Conjuros (F): ataque {spells.spell_bonus(h):+d}, CD {spells.spell_dc(h)}", None))
    if h.buffs:
        lines.append(("Estados: " + ", ".join(b.name for b in h.buffs), (0xFF, 0x88, 0x88)))
    cw = max(Font1x.Render(t).get_width() for t, _ in lines)
    content = py.Rect((W - cw) // 2, (H - TOOLBAR_H - len(lines) * 10) // 2, cw, len(lines) * 10)
    Hotspot(canvas.get_rect(), lambda: setattr(ui, "window", None))
    DrawWindow(canvas, content)
    for i, (text, color) in enumerate(lines):
        canvas.blit(Font1x.Render(text, color), (content.x, content.y + i * 10))

def DrawMenuWindow(canvas : py.Surface, d, ui : UiState, onMenu, onNew) -> None:
    """WndGame: menu de la partida (boton de menu o Esc). Nueva partida solo si el heroe murio, como el original.
    Configuracion abre la ventana de opciones encima de la partida (WndSettings), sin salir de ella."""
    W, H = canvas.get_size()
    def close():
        ui.window = None
    def new():
        onNew()
        ui.window = None
    rows = [[(t({"es": "Nueva partida", "en": "Start new game"}), new)]] if not d.hero.is_alive() else []
    rows += [[(t({"es": "Configuracion", "en": "Settings"}), lambda: setattr(ui, "window", ("settings",)))],
             [(t({"es": "Reglas de la partida", "en": "Game rules"}), lambda: setattr(ui, "window", ("rules",)))],
             [(t({"es": "Menu principal", "en": "Main menu"}), onMenu),
              (t({"es": "Salir", "en": "Exit"}), lambda: py.event.post(py.event.Event(py.QUIT)))],
             [(t({"es": "Volver al juego", "en": "Return to game"}), close)]]
    h = len(rows) * (MENU_BTN_H + MENU_GAP) - MENU_GAP
    content = py.Rect((W - MENU_W) // 2, (H - TOOLBAR_H - h) // 2, MENU_W, h)
    Hotspot(canvas.get_rect(), close) #click afuera cierra
    Hotspot(content, lambda: None)
    DrawWindow(canvas, content)
    for r, row in enumerate(rows):
        bw = (MENU_W - MENU_GAP * (len(row) - 1)) // len(row)
        for i, (label, fn) in enumerate(row):
            rect = py.Rect(content.x + i * (bw + MENU_GAP), content.y + r * (MENU_BTN_H + MENU_GAP), bw, MENU_BTN_H)
            DrawButton(canvas, rect, label, fn)

def DrawInfoWindow(canvas : py.Surface, ui : UiState) -> None:
    """WndInfoCell / WndInfoMob / WndInfoItem: icono + titulo (+ barra de vida si es un mob) y descripcion."""
    _, icon, title, text, hp = ui.window
    W, H = canvas.get_size()
    lines = []
    for para in text.split("\n"):
        lines += Font1x.Wrap(para, INFO_W) or [""]
    head = max(icon.get_height() if icon else 0, LINE_H + (4 if hp is not None else 0))
    content = py.Rect((W - INFO_W) // 2, 0, INFO_W, head + 4 + len(lines) * LINE_H)
    content.y = (H - TOOLBAR_H - content.h) // 2
    Hotspot(canvas.get_rect(), lambda: setattr(ui, "window", None)) #click en cualquier lado cierra
    DrawWindow(canvas, content)
    x = content.x
    if icon:
        canvas.blit(icon, (x, content.y))
        x += icon.get_width() + 3
    canvas.blit(Font1x.Render(title, (0xFF, 0xFF, 0x44)), (x, content.y + 1))
    if hp is not None: #HealthBar
        bar = py.Rect(x, content.y + LINE_H + 1, content.right - x, 2)
        canvas.fill((0xCC, 0, 0), bar)
        canvas.fill((0, 0xEE, 0), (bar.x, bar.y, int(bar.w * max(0.0, hp)), bar.h))
    for i, line in enumerate(lines):
        canvas.blit(Font1x.Render(line), (content.x, content.y + head + 4 + i * LINE_H))

def SetLanguage(code : str) -> None:
    lang.set_lang(code)
    ConfigMod.Save("language", code)

def SetFullscreen(on : bool) -> None:
    if on != ScreenMod.IsFullscreen:
        ScreenMod.ToggleFullscreen()
    ConfigMod.Save("fullscreen", "true" if on else "false")

def CurrentFps() -> int:
    return ScreenMod.Screen.UserFps if ScreenMod.Screen.UserFps is not None else ConfigMod.FrameRate

def SetFps(fps : int) -> None:
    ScreenMod.Screen.UserFps = fps #vale para todas las pantallas desde ya
    ConfigMod.Save("maxFPS", fps)

def SetUiScale(scale : float) -> None:
    ConfigMod.UiScale = scale #la pantalla de juego lo lee cada frame
    ConfigMod.Save("ui_scale", scale)

def DrawSettings(canvas : py.Surface, onClose) -> None:
    """WndSettings: idioma, pantalla completa y FPS. Se usa en el titulo (icono de preferencias) y en el menu de la
    partida. Cada cambio se aplica al momento y se guarda en Local.config."""
    W, H = canvas.get_size()
    yes, no = t({"es": "Si", "en": "Yes"}), t({"es": "No", "en": "No"})
    rows = [(t({"es": "Idioma", "en": "Language"}),
             [(name, code == lang.LANG, lambda c=code: SetLanguage(c)) for code, name in lang.LANGS.items()]),
            (t({"es": "Pantalla completa (F11)", "en": "Fullscreen (F11)"}),
             [(yes, ScreenMod.IsFullscreen, lambda: SetFullscreen(True)),
              (no, not ScreenMod.IsFullscreen, lambda: SetFullscreen(False))]),
            ("FPS", [(str(f) if f else t({"es": "Sin limite", "en": "Unlimited"}), CurrentFps() == f,
                      lambda f=f: SetFps(f)) for f in FPS_OPTIONS]),
            (t({"es": "Escala de la interfaz (en partida)", "en": "Interface scale (in game)"}),
             [(f"{round(s * 100)}%", abs(ConfigMod.UiScale - s) < 0.01, lambda s=s: SetUiScale(s)) for s in UI_SCALES])]
    row_h = LINE_H + MENU_BTN_H + MENU_GAP * 2
    content = py.Rect((W - SETTINGS_W) // 2, 0, SETTINGS_W, len(rows) * row_h + MENU_BTN_H)
    content.y = (H - content.h) // 2
    Hotspot(canvas.get_rect(), onClose) #click afuera cierra
    Hotspot(content, lambda: None)
    DrawWindow(canvas, content)
    y = content.y
    for label, options in rows:
        canvas.blit(Font1x.Render(label, SELECTED), (content.x, y))
        y += LINE_H
        bw = (SETTINGS_W - MENU_GAP * (len(options) - 1)) // len(options)
        for i, (text, selected, fn) in enumerate(options):
            rect = py.Rect(content.x + i * (bw + MENU_GAP), y, bw, MENU_BTN_H)
            DrawButton(canvas, rect, text, fn)
            if selected:
                py.draw.rect(canvas, SELECTED, rect, 1)
        y += MENU_BTN_H + MENU_GAP * 2
    DrawButton(canvas, py.Rect(content.x, y, SETTINGS_W, MENU_BTN_H), t({"es": "Volver", "en": "Back"}), onClose)

#Reglas de la partida (Dungeon.rules): (clave, texto, [(opcion, valor)]). La primera opcion es la de 5e
RULE_ROWS : list = [
    ("dot_death_saves",
     {"es": "Caido: el daño por turno (veneno, sangrado, fuego) suma fallos de salvacion",
      "en": "Downed: damage over time (poison, bleeding, fire) adds death save failures"},
     [({"es": "Si (5e)", "en": "Yes (5e)"}, True), ({"es": "No", "en": "No"}, False)]),
    ("save_half",
     {"es": "Salvacion superada contra gas toxico o descarga electrica",
      "en": "Successful save against toxic gas or lightning"},
     [({"es": "Mitad (5e)", "en": "Half (5e)"}, True), ({"es": "Sin efecto", "en": "No effect"}, False)]),
]

def DrawRulesWindow(canvas : py.Surface, d, onClose) -> None:
    """Reglas de la partida, como la configuracion de mundo de Don't Starve Together: se guardan con la partida
    (game/save.py) y se aplican al momento. En red las decidira el host."""
    W, H = canvas.get_size()
    rows = [(Font1x.Wrap(t(label), SETTINGS_W), key, opts) for key, label, opts in RULE_ROWS]
    title = t({"es": "Reglas de la partida", "en": "Game rules"})
    h = LINE_H + MENU_GAP * 2 + sum(len(l) * LINE_H + MENU_BTN_H + MENU_GAP * 2 for l, _, _ in rows) + MENU_BTN_H
    content = py.Rect((W - SETTINGS_W) // 2, (H - h) // 2, SETTINGS_W, h)
    Hotspot(canvas.get_rect(), onClose) #click afuera cierra
    Hotspot(content, lambda: None)
    DrawWindow(canvas, content)
    head = Font1x.Render(title, SELECTED)
    canvas.blit(head, (content.x + (SETTINGS_W - head.get_width()) // 2, content.y))
    y = content.y + LINE_H + MENU_GAP * 2
    for lines, key, opts in rows:
        for line in lines:
            canvas.blit(Font1x.Render(line), (content.x, y))
            y += LINE_H
        bw = (SETTINGS_W - MENU_GAP * (len(opts) - 1)) // len(opts)
        for i, (text, value) in enumerate(opts):
            rect = py.Rect(content.x + i * (bw + MENU_GAP), y, bw, MENU_BTN_H)
            DrawButton(canvas, rect, t(text), lambda k=key, v=value: d.rules.__setitem__(k, v))
            if d.rules.get(key) == value:
                py.draw.rect(canvas, SELECTED, rect, 1)
        y += MENU_BTN_H + MENU_GAP * 2
    DrawButton(canvas, py.Rect(content.x, y, SETTINGS_W, MENU_BTN_H), t({"es": "Volver", "en": "Back"}), onClose)

def DrawSpellsWindow(canvas : py.Surface, d, ui : UiState) -> None:
    """Libro de conjuros (tecla F): bono, CD, espacios y usos gratis. Click en un conjuro = elegir el objetivo
    (o lanzarlo, si es sobre uno mismo). En gris los que no se pueden lanzar ahora."""
    W, H = canvas.get_size()
    h = d.hero
    sc = spells.spellcasting(h)
    slots = "  ".join(f"nv{i + 1} {n}/{m}" for i, (n, m) in enumerate(zip(h.slots, spells.max_slots(h))))
    head = [(t({"es": "Conjuros", "en": "Spells"}), SELECTED),
            (f"{t({'es': 'Ataque', 'en': 'Attack'})} {spells.spell_bonus(h):+d}   CD {spells.spell_dc(h)}"
             f"   ({dice.ABILITY_NAMES[sc['ability']]})", None),
            (f"{t({'es': 'Espacios', 'en': 'Slots'})}: {slots or '-'}", None)]
    for sid, n in h.free_casts.items():
        head.append((f"{t(spells.SPELLS[sid]['name'])}: {n} {t({'es': 'gratis', 'en': 'free'})}", None))
    rows = []
    for sid in sc["spells"]:
        s = spells.SPELLS[sid]
        lvl = t({"es": "truco", "en": "cantrip"}) if s["level"] == 0 else f"nv {s['level']}"
        rows.append((sid, s, f"{t(s['name'])} ({lvl})", Font1x.Wrap(t(s.get("desc", "")), SPELLS_W)[:2]))
    height = len(head) * LINE_H + 4 + sum(MENU_BTN_H + 2 + len(r[3]) * LINE_H + MENU_GAP for r in rows)
    content = py.Rect((W - SPELLS_W) // 2, 0, SPELLS_W, height)
    content.y = max(2, (H - TOOLBAR_H - height) // 2)
    Hotspot(canvas.get_rect(), lambda: setattr(ui, "window", None))
    Hotspot(content, lambda: None)
    DrawWindow(canvas, content)
    y = content.y
    for text, color in head:
        canvas.blit(Font1x.Render(text, color), (content.x, y))
        y += LINE_H
    y += 4
    for sid, s, label, desc in rows:
        def choose(sid=sid, s=s):
            ui.window = None
            if s["target"] == "self":
                h.action = ("cast", sid, h.pos)
            else:
                ui.selecting = ("spell", sid)
        DrawButton(canvas, py.Rect(content.x, y, SPELLS_W, MENU_BTN_H), label, choose, spells.can_cast(h, sid))
        y += MENU_BTN_H + 2
        for line in desc:
            canvas.blit(Font1x.Render(line, GREY), (content.x, y))
            y += LINE_H
        y += MENU_GAP

def DrawPrompt(canvas : py.Surface, text : str) -> None:
    """GameScene.prompt: aviso arriba mientras se elige una casilla."""
    img = Font1x.Render(text)
    bg = py.Surface((img.get_width() + 8, img.get_height() + 4), py.SRCALPHA)
    bg.fill((0, 0, 0, 170))
    bg.blit(img, (4, 2))
    canvas.blit(bg, ((canvas.get_width() - bg.get_width()) // 2, 34))

def DrawUi(canvas : py.Surface, d, ui : UiState, avatar : py.Surface, onMenu, onNew) -> None:
    Hot.clear()
    DrawStatus(canvas, d, avatar, ui)
    DrawToolbar(canvas, d, ui)
    if ui.window and ui.window[0] == "bag":
        DrawBag(canvas, d, ui)
    elif ui.window and ui.window[0] == "item":
        DrawItemWindow(canvas, d, ui)
    elif ui.window and ui.window[0] == "hero":
        DrawHeroWindow(canvas, d, ui)
    elif ui.window and ui.window[0] == "menu":
        DrawMenuWindow(canvas, d, ui, onMenu, onNew)
    elif ui.window and ui.window[0] == "info":
        DrawInfoWindow(canvas, ui)
    elif ui.window and ui.window[0] == "spells":
        DrawSpellsWindow(canvas, d, ui)
    elif ui.window and ui.window[0] == "settings":
        DrawSettings(canvas, lambda: setattr(ui, "window", None))
    elif ui.window and ui.window[0] == "rules":
        DrawRulesWindow(canvas, d, lambda: setattr(ui, "window", ("menu",)))
    if not ui.window:
        DrawTooltip(canvas, ui)
    if ui.selecting == "examine" and not ui.window:
        DrawPrompt(canvas, t({"es": "Toca una casilla para examinarla", "en": "Select a cell to examine"}))
    elif ui.selecting and ui.selecting[0] == "throw" and not ui.window:
        name = item_name(ui.selecting[1], d)
        DrawPrompt(canvas, t({"es": f"Elige donde lanzar: {name} (Esc cancela)", "en": f"Choose where to throw: {name} (Esc cancels)"}))
    elif ui.selecting and ui.selecting[0] == "spell" and not ui.window:
        name = t(spells.SPELLS[ui.selecting[1]]["name"])
        DrawPrompt(canvas, t({"es": f"Elige el objetivo: {name} (Esc cancela)", "en": f"Choose a target: {name} (Esc cancels)"}))
