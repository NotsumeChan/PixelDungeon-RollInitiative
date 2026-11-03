#Pantalla de juego: dibuja la partida de game/ (Dungeon) y traduce el input a acciones del heroe.
import random
import pygame as py
from core import Screen, PickResolution, FrameRate, Font1x, DevDiceLog, DevHotseat
from game.dungeon import Dungeon
from game import save
from game import terrain as T
from game.items import HEAP, CHEST, TOMB, SKELETON, item_name, item_image, item_info
from game.mobs import MOB_STATES
from game.lang import t
from game.geom import WIDTH, HEIGHT
from game.globals import Globals
from game.hero import Hero
from . import gameUi as ui
from .gameUi import UiState, ItemIcon, TOOLBAR_H
from .diceFx import DiceFx
from . import charSprite as cs
from .particles import Effects

#screen define
GameScreen : Screen = Screen(PickResolution[0], PickResolution[1])

#Assets: tiles0 = Alcantarillas. El indice del tile es el valor de terreno (DungeonTilemap.java)
TILE_SIZE : int = 16
_Sheet : py.Surface = py.image.load("Assets/Tiles/tiles0.png").convert_alpha()
Tiles : list[py.Surface] = [_Sheet.subsurface(((i % 16) * TILE_SIZE, (i // 16) * TILE_SIZE, TILE_SIZE, TILE_SIZE))
                            for i in range(64)]
Water : py.Surface = py.image.load("Assets/Tiles/water0.png").convert()
_SpriteCache : dict[tuple, py.Surface] = {}

#Velocidades: todo lo que avanza con el tiempo se multiplica por dt (independiente de los FPS)
MOVE_TIME   : float = 0.1 #segundos por casilla (CharSprite.MOVE_INTERVAL)
WATER_SPEED : float = 5   #px/s hacia abajo (GameScene: water.offset(0, -5 * elapsed))

#Niebla (FogOfWar.java): un color por esquina de celda, interpolado
FOG_VISIBLE   : tuple = (0, 0, 0, 0)
FOG_VISITED   : tuple = (0x11, 0x11, 0x11, 0xcc)
FOG_MAPPED    : tuple = (0x44, 0x22, 0x11, 0xcc)
FOG_INVISIBLE : tuple = (0, 0, 0, 0xff)
_FogTiles : dict = {} #(4 colores de esquina) -> tile de niebla de 16x16 ya interpolado

MOVE_KEYS : dict = {py.K_w: (0, -1), py.K_UP: (0, -1), py.K_s: (0, 1), py.K_DOWN: (0, 1),
                    py.K_a: (-1, 0), py.K_LEFT: (-1, 0), py.K_d: (1, 0), py.K_RIGHT: (1, 0)}
LOG_LINES : int = 4
#zoom de la camara (rueda / pellizco), como el original: pixeles de pantalla por pixel del mundo, siempre ENTERO
#(nitido). Con el zoom base 4 (HD): escalas 2..6 = 0.5, 0.75, 1, 1.25, 1.5 respecto del zoom base
ZOOMS : list = [s / GameScreen.zoom for s in range(max(1, GameScreen.zoom - 2), GameScreen.zoom + 3)]
PINCH_STEP : float = 0.04     #cuanto hay que pellizcar (MULTIGESTURE.pinched acumulado) para cambiar un paso
_World : dict = {}            #superficie del mundo por tamano (el mundo se dibuja aparte y se escala entero)
_Ui : dict = {}               #capa transparente de la UI por tamano
DRAG_MIN_PX : int = 4 #arrastrar el mapa mas que esto = mover la camara (no es un click)
#EmoIcon.java: (icono, escala maxima, velocidad del pulso por segundo)
EMO_SLEEP : tuple = (ui.ICON_SLEEP, 1.2, 0.5)
EMO_ALERT : tuple = (ui.ICON_ALERT, 1.3, 2.0)
#FloatingText.java: segundos de vida, pixeles que sube, colores (CharSprite.DEFAULT/POSITIVE/NEGATIVE/WARNING/NEUTRAL)
FLOAT_TIME, FLOAT_RISE, FLOAT_STACK = 1.0, 16, 8
FLOAT_COLORS : dict = {"default": (255, 255, 255), "pos": (0, 255, 0), "neg": (255, 68, 68), "warn": (255, 136, 0),
                       "neutral": (255, 255, 0)}
CHECKED_COLOR : tuple = (0x55, 0xAA, 0xFF) #CheckedCell: se achica y se desvanece en 0.8 s
LEAF_COLORS : list = [(0x00, 0x44, 0x00), (0x88, 0xCC, 0x44)] #LeafParticle de las Alcantarillas
HEAP_INFO : dict = {CHEST: ({"es": "Cofre", "en": "Chest"},
                            {"es": "No sabrás qué tiene hasta abrirlo.", "en": "You won't know what's inside until you open it."}),
                    TOMB: ({"es": "Tumba", "en": "Tomb"},
                           {"es": "Aquí descansa alguien. Quizás se llevó algo de valor a la tumba.",
                            "en": "This ancient tomb may contain something useful, but its owner will most certainly object to checking."}),
                    SKELETON: ({"es": "Restos de un aventurero", "en": "Skeletal remains"},
                               {"es": "Lo único que queda de un aventurero sin suerte. Quizás tenga algo de valor.",
                                "en": "This is all that's left from one of your predecessors. Maybe it's worth checking for any valuables."})}
ALERT_TIME : float = 1.5 #segundos que se ve el "!" cuando un mob te nota (en Java, hasta su proximo turno)

def Zoomed(zoom : float, step : int) -> float:
    """Un paso de zoom (1 = acercar, -1 = alejar). Se guarda en Local.config."""
    i = min(range(len(ZOOMS)), key=lambda k: abs(ZOOMS[k] - zoom)) #el mas cercano (por si la config trae otro)
    new = ZOOMS[max(0, min(len(ZOOMS) - 1, i + step))]
    if new != zoom:
        ui.ConfigMod.CameraZoom = new
        ui.ConfigMod.Save("camera_zoom", new)
    return new

def WorldSurface(w : int, h : int) -> py.Surface:
    if (w, h) not in _World:
        _World[(w, h)] = py.Surface((w, h))
    surf = _World[(w, h)]
    surf.fill((0, 0, 0))
    return surf

def UiSurface(w : int, h : int) -> py.Surface:
    if (w, h) not in _Ui:
        _Ui[(w, h)] = py.Surface((w, h), py.SRCALPHA)
    surf = _Ui[(w, h)]
    surf.fill((0, 0, 0, 0))
    return surf

def UiScale() -> int:
    """Pixeles de pantalla por pixel de UI (entero): zoom base x escala de la interfaz de la configuracion."""
    return max(1, round(ui.ConfigMod.UiScale * GameScreen.zoom))

def UiPos(m : tuple) -> tuple:
    """Posicion del mouse (en el canvas virtual) en la capa de UI."""
    s = UiScale()
    return m[0] * GameScreen.zoom // s, m[1] * GameScreen.zoom // s

def SaveRun() -> None:
    """Guarda la partida del personaje en juego (game/save.py). Sin personaje (prueba rapida) no se guarda."""
    d, char = Globals.get("dungeon"), Globals.get("character")
    if d is not None and char and char.get("id"):
        save.save_run(d, char["id"])

def NewGame() -> None:
    """Partida nueva con el personaje de Globals["character"]. Queda en Globals["runs"][id del personaje]."""
    char = Globals.get("character") or {"id": None, "class": "pddnd:warrior", "abilities": None}
    #la seed de partida puede salir del random global
    d = Globals["dungeon"] = Dungeon(random.randrange(2**31), char["class"], char["abilities"])
    d.hero.name = char.get("name") or d.hero.name
    if DevHotseat: #modo de prueba: un segundo heroe igual en la misma PC (Tab cambia de heroe)
        d.add_hero(char["class"], char["abilities"]).name = f"{d.hero.name} 2"
    Globals.setdefault("runs", {})[char.get("id")] = d

def SwitchHero(d : Dungeon, hero : Hero) -> None:
    """Modo de prueba: la UI pasa a manejar a otro heroe."""
    if hero is not d.hero:
        d.set_local(hero)
        d.log(f"Turno de {hero.name}")

def AutoSwitch(d : Dungeon) -> None:
    """Modo de prueba: si el piso espera la accion de otro heroe (o el local murio o cayo), la UI pasa a ese heroe."""
    if not d.hero.is_alive() or d.hero.is_downed():
        up = [h for h in d.heroes if h.is_alive() and not h.is_downed()]
        if up:
            SwitchHero(d, up[0])
        return
    b = d.blocking()
    if b is not None and b is not d.hero and b.is_alive() and not b.is_downed() and b.action is None \
            and not b.animating and not d.hero.animating:
        SwitchHero(d, b)

def Pulse(t : float, maxSize : float, rate : float) -> float:
    """Escala que va y viene entre 1 y maxSize (EmoIcon.update)."""
    span = maxSize - 1
    p = (t * rate) % (2 * span)
    return 1 + (p if p < span else 2 * span - p)

def DrawEmos(canvas : py.Surface, d : Dungeon, sprites : dict, emo : dict, ox : int, oy : int) -> None:
    """Zzz sobre los mobs dormidos y ! cuando te notan (CharSprite.showSleep / showAlert)."""
    for m in d.mobs:
        if not d.level.visible[m.pos]:
            continue
        if m.state == "SLEEPING":
            region, maxSize, rate = EMO_SLEEP
        elif emo["alerts"].get(m, 0) > 0:
            region, maxSize, rate = EMO_ALERT
        else:
            continue
        phase = emo["phase"].setdefault(m, random.random() * 4) #azar visual: random global
        icon = ui.IconImg(region)
        s = Pulse(emo["time"] + phase, maxSize, rate)
        img = py.transform.scale(icon, (round(icon.get_width() * s), round(icon.get_height() * s)))
        x, y = sprites.get(m, SpritePos(m))
        canvas.blit(img, (ox + int(x) + CharImg(m).get_width() - img.get_width() // 2, oy + int(y) - img.get_height()))

def UpdateEmos(emo : dict, d : Dungeon, dt : float) -> None:
    """Un mob que se despierta o pasa a cazarte muestra el ! durante ALERT_TIME."""
    emo["time"] += dt
    for m in d.mobs:
        prev = emo["states"].get(m)
        emo["states"][m] = m.state
        if prev is not None and prev != m.state and m.state != "SLEEPING" and (prev == "SLEEPING" or m.state == "HUNTING"):
            emo["alerts"][m] = ALERT_TIME
    for m in list(emo["alerts"]):
        emo["alerts"][m] -= dt
        if emo["alerts"][m] <= 0:
            del emo["alerts"][m]

def UpdateFx(fx : dict, d : Dungeon, dt : float, anims : dict, sprites : dict, effects : Effects) -> None:
    """Consume d.fx (texto flotante, casillas revisadas, hojas, animaciones, muertes, trampas) y los anima con dt."""
    level = d.level
    for e in d.fx:
        if e[0] == "trap":
            if level.visible[e[2]]:
                effects.Trap(e[1], e[2], level)
        elif e[0] == "appear":
            effects.Appear(e[1])
        elif e[0] == "missile": #arma arrojadiza o flecha en vuelo
            if level.visible[e[1]] or level.visible[e[2]]:
                effects.Missile(*e[1:])
        elif e[0] == "bolt": #proyectil de un conjuro
            effects.Bolt(e[1], e[2], e[3])
        elif e[0] == "anim": #ataque / operar / leer
            cs.Act(anims, e[1], e[2], e[3])
        elif e[0] == "die": #el mob ya salio de la partida: queda su cadaver animado donde estaba su sprite
            ch = e[1]
            old = anims.get(ch)
            fx["corpses"].append([ch, list(sprites.get(ch, SpritePos(ch))), cs.CharAnim("die", old.flip if old else False)])
        elif e[0] == "status":
            _, ch, text, kind = e
            row = sum(1 for f in fx["floats"] if f[0] is ch)
            fx["floats"].append([ch, text, FLOAT_COLORS.get(kind, FLOAT_COLORS["default"]), 0.0, row])
        elif e[0] == "checked":
            fx["checked"].append([e[1], 0.8])
        elif e[0] == "leaves": #azar visual: random global
            x, y = (e[1] % WIDTH) * TILE_SIZE, (e[1] // WIDTH) * TILE_SIZE
            for _ in range(4):
                fx["leaves"].append([x + random.uniform(2, 14), y + random.uniform(4, 14), random.uniform(-8, 8),
                                     random.uniform(-25, -10), 1.0, random.choice(LEAF_COLORS)])
    d.fx.clear()
    for f in fx["floats"]:
        f[3] += dt
    fx["floats"] = [f for f in fx["floats"] if f[3] < FLOAT_TIME]
    for c in fx["checked"]:
        c[1] -= dt
    fx["checked"] = [c for c in fx["checked"] if c[1] > 0]
    for l in fx["leaves"]:
        l[0] += l[2] * dt
        l[1] += l[3] * dt
        l[3] += 40 * dt
        l[4] -= dt
    fx["leaves"] = [l for l in fx["leaves"] if l[4] > 0]
    for ch in d.chars(): #estados con efecto visual (en llamas)
        if level.visible[ch.pos] and any(b.data.get("fx") == "flame" for b in ch.buffs):
            x, y = sprites.get(ch, SpritePos(ch))
            effects.Pour((ch, "flame"), py.Rect(int(x), int(y), *cs.FrameSize(ch)), dt)
    effects.Update(dt, level.visible)
    for c in fx["corpses"]:
        c[2].t += dt
    fx["corpses"] = [c for c in fx["corpses"] if cs.CorpseAlpha(c[2], c[0]) > 0]

NAME_LOCAL, NAME_OTHER, NAME_DOWN = (0xFF, 0xFF, 0x44), (0xFF, 0xFF, 0xFF), (0x88, 0x88, 0x88)

def DrawChecked(canvas : py.Surface, fx : dict, ox : int, oy : int) -> None:
    for cell, alpha in fx["checked"]:
        s = max(1, int(TILE_SIZE * alpha))
        sq = py.Surface((s, s), py.SRCALPHA)
        sq.fill((*CHECKED_COLOR, int(255 * alpha)))
        cx, cy = (cell % WIDTH) * TILE_SIZE + TILE_SIZE // 2, (cell // WIDTH) * TILE_SIZE + TILE_SIZE // 2
        canvas.blit(sq, (ox + cx - s // 2, oy + cy - s // 2))

def DrawFloats(canvas : py.Surface, fx : dict, sprites : dict, ox : int, oy : int) -> None:
    for x, y, _, _, life, color in fx["leaves"]:
        canvas.set_at((ox + int(x), oy + int(y)), color)
    for ch, text, color, t, row in fx["floats"]:
        img = Font1x.Render(text, color).copy() #copia: set_alpha no debe tocar la cache del font
        img.set_alpha(int(255 * (1 - t / FLOAT_TIME)))
        x, y = sprites.get(ch, SpritePos(ch))
        canvas.blit(img, (ox + int(x) + CharImg(ch).get_width() // 2 - img.get_width() // 2,
                          oy + int(y) - img.get_height() - int(FLOAT_RISE * t / FLOAT_TIME) - FLOAT_STACK * row))

def DrawNames(canvas : py.Surface, d : Dungeon, sprites : dict, ox : int, oy : int) -> None:
    """Multijugador (2+ heroes): nombre sobre cada heroe del piso que se ve. El tuyo en amarillo, caidos en gris."""
    if len(d.heroes) < 2:
        return
    for h in d.here():
        if h.dead or not (h is d.hero or d.level.visible[h.pos]):
            continue
        color = NAME_DOWN if h.is_downed() else NAME_LOCAL if h is d.hero else NAME_OTHER
        img = Font1x.Render(h.name, color)
        x, y = sprites.get(h, SpritePos(h))
        canvas.blit(img, (ox + int(x) + CharImg(h).get_width() // 2 - img.get_width() // 2,
                          oy + int(y) - img.get_height() - 1))

def Examine(d : Dungeon, state : UiState, cell : int) -> None:
    """Toolbar.informer: que hay en la casilla (el heroe, un mob, un monton de items o el terreno)."""
    level, hero = d.level, d.hero
    if not (0 <= cell < WIDTH * HEIGHT) or not (level.visited[cell] or level.mapped[cell]):
        state.window = ("info", None, "?", t({"es": "No sabes qué hay ahí.", "en": "You don't know what is there."}), None)
        return
    if level.visible[cell]:
        if cell == hero.pos:
            state.window = ("hero",)
            return
        ch = d.find_char(cell)
        if ch is not None: #WndInfoMob, con los numeros de 5e
            text = f"{t(ch.stats.get('desc', ''))}\n\n{t(MOB_STATES.get(ch.state, ''))}. CA {ch.armor_class()}, PV {ch.HP}/{ch.HT}."
            if ch.buffs:
                text += "\nEstados: " + ", ".join(b.name for b in ch.buffs)
            state.window = ("info", CharImg(ch), ch.name.capitalize(), text, ch.HP / ch.HT)
            return
        heap = level.heaps.get(cell)
        if heap is not None and heap.items:
            if heap.type != HEAP:
                title, text = HEAP_INFO.get(heap.type, ("?", ""))
                state.window = ("info", None, t(title), t(text), None)
            else: #WndInfoItem: el item de arriba
                item = heap.items[0]
                state.window = ("info", ItemIcon(item_image(item, d)), item_name(item, d).capitalize(),
                                item_info(item, d, hero), None)
            return
    tile, name, desc = T.tile_info(level.map[cell], level.water[cell], level.pit[cell])
    icon = Water.subsurface((0, 0, TILE_SIZE, TILE_SIZE)) if tile == T.WATER else Tiles[tile]
    state.window = ("info", icon, name, desc or t({"es": "No hay nada aquí.", "en": "There is nothing here."}), None)

def CharImg(ch) -> py.Surface:
    """Primer frame del sprite (iconos y medidas). Lo que se ve en el mapa sale de charSprite.Image (animado)."""
    return cs.Frame(ch, 0)

def Avatar(cls : dict) -> py.Surface:
    """HeroSprite.avatar: recorte (1, 0) - (12, 15) de la hoja de la clase (cls = entrada de HERO_CLASSES)."""
    key = ("avatar", cls["sprite"])
    if key not in _SpriteCache:
        _SpriteCache[key] = py.image.load(f"Assets/Characters/{cls['sprite']}").convert_alpha().subsurface((1, 0, 11, 15))
    return _SpriteCache[key]

def SpritePos(ch) -> list[float]:
    """Esquina del sprite parado en su celda: centrado en X, apoyado abajo (CharSprite.worldToCamera)."""
    w, h = cs.FrameSize(ch)
    return [(ch.pos % WIDTH) * TILE_SIZE + (TILE_SIZE - w) / 2, (ch.pos // WIDTH + 1) * TILE_SIZE - h]

def FogTile(colors : tuple) -> py.Surface:
    """Tile de niebla: degradado bilineal entre los colores de sus 4 esquinas (arriba-izq, arriba-der, abajo-izq,
    abajo-der). Cacheado: hay pocas combinaciones."""
    if colors not in _FogTiles:
        small = py.Surface((2, 2), py.SRCALPHA)
        for (x, y), c in zip(((0, 0), (1, 0), (0, 1), (1, 1)), colors):
            small.set_at((x, y), c)
        _FogTiles[colors] = py.transform.smoothscale(small, (TILE_SIZE, TILE_SIZE))
    return _FogTiles[colors]

def BuildFog(level) -> py.Surface:
    """FogOfWar.java: un color por ESQUINA de celda (visible si las 4 celdas que la tocan lo son) interpolado.
    El original estira una textura de esquinas x16 con filtro LINEAR de OpenGL. Aqui se arma tile por tile: estirar
    el mapa entero con smoothscale corria la niebla hasta medio tile hacia los bordes (pygame reparte los pixeles
    con (n-1)/m, no con 1/16) y se veia detras de las paredes simples."""
    vis, vst, mpd = level.visible, level.visited, level.mapped
    corners = [[FOG_INVISIBLE] * (WIDTH + 1) for _ in range(HEIGHT + 1)] #el borde del mapa queda negro
    for i in range(1, HEIGHT):
        for j in range(1, WIDTH):
            pos = i * WIDTH + j
            corner = (pos, pos - WIDTH, pos - 1, pos - WIDTH - 1) #las 4 celdas que tocan esta esquina
            if all(vis[c] for c in corner):
                corners[i][j] = FOG_VISIBLE
            elif all(vst[c] for c in corner):
                corners[i][j] = FOG_VISITED
            elif all(mpd[c] for c in corner):
                corners[i][j] = FOG_MAPPED
    fog = py.Surface((WIDTH * TILE_SIZE, HEIGHT * TILE_SIZE), py.SRCALPHA)
    for y in range(HEIGHT):
        for x in range(WIDTH):
            key = (corners[y][x], corners[y][x + 1], corners[y + 1][x], corners[y + 1][x + 1])
            if key != (FOG_VISIBLE,) * 4: #MAX sobre transparente = copia exacta (sin oscurecer por el alpha)
                fog.blit(FogTile(key), (x * TILE_SIZE, y * TILE_SIZE), special_flags=py.BLEND_RGBA_MAX)
    return fog

def DrawLevel(canvas : py.Surface, d : Dungeon, ox : int, oy : int, waterOff : float) -> None:
    level = d.level
    canvas.set_clip(py.Rect(ox, oy, WIDTH * TILE_SIZE, HEIGHT * TILE_SIZE))
    #agua debajo de todo; los tiles de agua (48..63) son transparentes donde hay agua
    ww, wh = Water.get_size()
    for y in range(oy - wh + int(waterOff) % wh, oy + HEIGHT * TILE_SIZE, wh):
        for x in range(ox, ox + WIDTH * TILE_SIZE, ww):
            canvas.blit(Water, (x, y))
    for i, t in enumerate(level.map):
        canvas.blit(Tiles[t], (ox + (i % WIDTH) * TILE_SIZE, oy + (i // WIDTH) * TILE_SIZE))
    #montones de items: se recuerdan donde ya se vio (la niebla los oscurece)
    for pos, heap in level.heaps.items():
        if level.visited[pos] or level.mapped[pos]:
            canvas.blit(ItemIcon(heap.image(d)), (ox + (pos % WIDTH) * TILE_SIZE, oy + (pos // WIDTH) * TILE_SIZE))
    canvas.set_clip(None)

def AnimateSprites(sprites : dict, chars : list, dt : float) -> None:
    """Cada sprite viaja hacia su celda a 1 casilla cada MOVE_TIME. La logica ya movio al personaje."""
    step = TILE_SIZE / MOVE_TIME * dt
    alive = set(chars)
    for ch in list(sprites):
        if ch not in alive:
            del sprites[ch]
    for ch in chars:
        target = SpritePos(ch)
        pos = sprites.setdefault(ch, target)
        for axis in (0, 1):
            diff = target[axis] - pos[axis]
            pos[axis] = target[axis] if abs(diff) <= step else pos[axis] + step * (1 if diff > 0 else -1)

def DrawLog(canvas : py.Surface, d : Dungeon) -> None:
    W, H = canvas.get_size() #la capa de UI (cambia con la escala de la interfaz)
    bottom = H - TOOLBAR_H - 1
    for i, msg in enumerate(d.messages[-LOG_LINES:]):
        canvas.blit(Font1x.Render(msg), (2, bottom - (LOG_LINES - i) * 9))
    if not any(h.is_alive() for h in d.heroes):
        text = Font1x.Render("Has muerto - R nueva partida, Esc menu", (255, 80, 80))
        canvas.blit(text, ((W - text.get_width()) // 2, H // 2 - 20))
    elif d.hero.is_downed(): #salvaciones contra la muerte en curso
        s, f = d.hero.saves
        msg = "Inconsciente - estable" if d.hero.stable else f"Inconsciente - exitos {s}/3, fallos {f}/3"
        text = Font1x.Render(msg, (255, 136, 0))
        canvas.blit(text, ((W - text.get_width()) // 2, H // 2 - 20))

#function
def Game(Delta) -> None:
    GameScreen.FrameRate(FrameRate)
    if "dungeon" not in Globals:
        NewGame()
    sprites : dict = {}
    emo : dict = {"time": 0.0, "states": {}, "alerts": {}, "phase": {}}
    fx : dict = {"floats": [], "checked": [], "leaves": [], "corpses": []}
    zoom : float = Zoomed(ui.ConfigMod.CameraZoom, 0)
    pinch : float = 0.0
    anims : dict = {} #personaje -> charSprite.CharAnim
    effects = Effects() #particulas y efectos de trampas
    state = UiState()
    dicefx = DiceFx()
    fogLevel = fogVisible = spriteLevel = None
    fog = None
    waterOff = 0.0
    ox = oy = 0
    pan = [0, 0]          #desplazamiento de camara al arrastrar el mapa
    press = None          #(posicion al apretar, pan al apretar) mientras se arrastra el mapa
    panned = False

    def ToMenu():
        SaveRun()
        GameScreen.running = False

    Screen.OnQuit = SaveRun #cerrar la ventana (o Salir del menu) tambien guarda

    while GameScreen.running:
        dt = GameScreen.dt
        d : Dungeon = Globals["dungeon"]
        hero = d.hero
        #event handler
        for event in py.event.get():
            GameScreen.EventHandler(event, {})
            if event.type == py.KEYDOWN:
                if event.key == py.K_ESCAPE: #cierra la ventana abierta o abre el menu de la partida
                    if state.selecting:
                        state.selecting = None
                    else:
                        state.window = None if state.window else ("menu",)
                elif event.key == py.K_i:
                    state.window = None if state.window else ("bag", None)
                elif event.key == py.K_r:
                    NewGame()
                    state.window = None
                elif event.key == py.K_SPACE and hero.is_alive() and not state.window:
                    hero.action = ("rest",)
                elif event.key == py.K_q and hero.is_alive() and not state.window:
                    hero.action = ("search",)
                elif event.key == py.K_f and hero.is_alive() and not state.window: #rasgo de clase
                    if hero.has_trait("pddnd:second_wind"):
                        hero.action = ("second_wind",)
                    elif hero.cls.get("spellcasting"): #libro de conjuros
                        state.window = ("spells",)
                elif event.key in (py.K_1, py.K_2) and not state.window: #accesos rapidos
                    ui.UseQuick(d, state, 0 if event.key == py.K_1 else 1)
                elif event.key == py.K_c and len(d.heroes) > 1 and not state.window: #centrar camara (multijugador)
                    state.center = True
                elif event.key == py.K_TAB and DevHotseat and not state.window: #modo de prueba: otro heroe
                    alive = [h for h in d.heroes if h.is_alive()]
                    if hero in alive and len(alive) > 1:
                        SwitchHero(d, alive[(alive.index(hero) + 1) % len(alive)])
                        state.selecting = None
            elif event.type == py.MOUSEBUTTONDOWN and event.button in (1, 3):
                m = GameScreen.MousePos()
                #primero la UI; si no toco nada, es el mapa (se decide al soltar: click o arrastre)
                if not ui.Click(UiPos(m), alt=event.button == 3) and event.button == 1:
                    press, panned = (m, list(pan)), False
            elif event.type == py.MOUSEWHEEL and not state.window: #rueda: acercar / alejar la camara
                zoom = Zoomed(zoom, 1 if event.y > 0 else -1)
            elif event.type == getattr(py, "MULTIGESTURE", -1) and not state.window: #pellizco en pantalla tactil
                pinch += event.pinched
                if abs(pinch) >= PINCH_STEP:
                    zoom, pinch = Zoomed(zoom, 1 if pinch > 0 else -1), 0.0
            elif event.type == py.MOUSEBUTTONUP and event.button == 1 and press is not None:
                m = GameScreen.MousePos()
                if not panned and hero.is_alive():
                    cx, cy = (int(m[0] / zoom) - ox) // TILE_SIZE, (int(m[1] / zoom) - oy) // TILE_SIZE
                    if state.selecting == "examine": #CellSelector del boton de examinar
                        state.selecting = None
                        Examine(d, state, cx + cy * WIDTH)
                    elif state.selecting and state.selecting[0] == "throw": #donde lanzar un arma arrojadiza
                        hero.action = ("throw", state.selecting[1], cx + cy * WIDTH)
                        state.selecting = None
                    elif state.selecting and state.selecting[0] == "spell": #objetivo de un conjuro
                        hero.action = ("cast", state.selecting[1], cx + cy * WIDTH)
                        state.selecting = None
                    elif 0 <= cx < WIDTH and 0 <= cy < HEIGHT:
                        hero.handle(cx + cy * WIDTH, stairs=True) #sólo el click usa escaleras
                press = None

        #window breaker
        if not GameScreen.running:
            break

        d = Globals["dungeon"] #R pudo crear otra partida
        hero = d.hero

        #teclas mantenidas: un paso cada vez que el heroe queda libre (no con una ventana abierta)
        if hero.is_alive() and hero.action is None and not hero.animating and not state.window:
            keys = py.key.get_pressed()
            dx = dy = 0
            for key, (kx, ky) in MOVE_KEYS.items():
                if keys[key]:
                    dx, dy = dx or kx, dy or ky #diagonal = dos teclas a la vez
            if dx or dy:
                hero.handle(hero.pos + dx + dy * WIDTH)

        #arrastrar el mapa mueve la camara para inspeccionar
        if press is not None:
            m = GameScreen.MousePos()
            dx, dy = m[0] - press[0][0], m[1] - press[0][1]
            if panned or abs(dx) + abs(dy) >= DRAG_MIN_PX:
                panned = True
                pan = [press[1][0] + dx / zoom, press[1][1] + dy / zoom] #pan en pixeles del mundo

        if state.center: #boton / tecla C: la camara vuelve a tu heroe
            pan, press, state.center = [0, 0], None, False

        #animacion -> cuando el heroe llega a su celda, la logica sigue
        AnimateSprites(sprites, d.chars(), dt)
        if sprites.get(hero) != SpritePos(hero) and press is None:
            pan = [0, 0] #el heroe camina: la camara vuelve a seguirlo
        for h in d.heroes: #los de otro piso no se dibujan: no esperan animacion
            if h.depth != d.depth or sprites.get(h) == SpritePos(h):
                h.animating = False
        d.verbose_rolls = DevDiceLog
        dicefx.Feed(d.dice_events)
        if d.pending and not dicefx.Busy(): #el dado ya mostro el resultado: recien ahi se aplica el daño
            d.resolve_pending()
        d.process()
        if DevHotseat:
            AutoSwitch(d)
            hero = d.hero
        if d.level is not spriteLevel: #piso o partida nueva: los sprites arrancan en su celda, sin deslizarse
            if spriteLevel is not None: #cambio de piso (o partida nueva): autoguardado
                SaveRun()
            sprites.clear()
            spriteLevel = d.level
            emo.update(states={}, alerts={}, phase={})
            fx.update(floats=[], checked=[], leaves=[], corpses=[])
            anims.clear()
            effects.Clear()

        level = d.level
        if level is not fogLevel or level.visible is not fogVisible: #observe() crea una lista nueva
            fog, fogLevel, fogVisible = BuildFog(level), level, level.visible
        waterOff += WATER_SPEED * dt
        UpdateEmos(emo, d, dt)
        UpdateFx(fx, d, dt, anims, sprites, effects)
        cs.Update(anims, d.chars(), sprites, SpritePos, dt)
        dicefx.Feed(d.dice_events)
        dicefx.Update(dt)

        #camara centrada en el sprite del heroe (Camera.focusOn)
        heroImg = CharImg(hero)
        hx, hy = sprites.get(hero, SpritePos(hero))
        hires = GameScreen.HiRes()
        scale = max(1, round(zoom * GameScreen.zoom)) #pixeles de pantalla por pixel del mundo (entero)
        viewW, viewH = -(-hires.get_width() // scale), -(-hires.get_height() // scale) #lo que se ve del mundo
        ox = int(viewW / 2 - (hx + heroImg.get_width() / 2) + pan[0])
        oy = int(viewH / 2 - (hy + heroImg.get_height() / 2) + pan[1])

        GameScreen.ResetScreen()
        canvas = WorldSurface(viewW, viewH) #el mundo se dibuja aparte, en pixeles del juego
        DrawLevel(canvas, d, ox, oy, waterOff)
        DrawChecked(canvas, fx, ox, oy)
        for ch, (x, y), anim in fx["corpses"]: #MobSprite: animacion de muerte y luego se desvanece
            if level.visible[ch.pos]:
                img = cs.Frame(ch, anim.index(cs.Anims(ch)), anim.flip).copy()
                img.set_alpha(int(255 * cs.CorpseAlpha(anim, ch)))
                canvas.blit(img, (ox + int(x), oy + int(y)))
        for ch in d.chars():
            if ch is hero or level.visible[ch.pos]:
                x, y = sprites.get(ch, SpritePos(ch))
                img = cs.Image(anims, ch)
                if ch in effects.appear: #recien invocado: aparece de a poco
                    img = img.copy()
                    img.set_alpha(int(255 * effects.Alpha(ch)))
                canvas.blit(img, (ox + int(x), oy + int(y)))
        effects.Draw(canvas, ox, oy)
        canvas.blit(fog, (ox, oy)) #la esquina (j, i) cae en (16j, 16i): sin desplazamiento
        DrawEmos(canvas, d, sprites, emo, ox, oy)
        DrawNames(canvas, d, sprites, ox, oy)
        DrawFloats(canvas, fx, sprites, ox, oy)
        #mundo -> resolucion completa con factor entero (nitido, como Camera.zoom del original)
        hires.blit(py.transform.scale(canvas, (viewW * scale, viewH * scale)), (0, 0))
        uiScale = UiScale() #la UI va en su capa, del tamano que da la escala de la interfaz
        uiW, uiH = -(-hires.get_width() // uiScale), -(-hires.get_height() // uiScale)
        canvas = UiSurface(uiW, uiH)
        state.mouse = UiPos(GameScreen.MousePos())
        DrawLog(canvas, d)
        ui.DrawUi(canvas, d, state, Avatar(hero.cls), ToMenu, NewGame)
        dicefx.Draw(canvas)
        #UI encima, escalada por el zoom base (entero). Blit normal con alfa: sobre una capa transparente pygame copia
        #los pixeles tal cual (no premultiplica), y los invisibles de textos y sprites suelen ser blancos con alfa 0
        hires.blit(py.transform.scale(canvas, (uiW * uiScale, uiH * uiScale)), (0, 0))

        #screen updater
        GameScreen.Update()
    Screen.OnQuit = None
    GameScreen.running = True
