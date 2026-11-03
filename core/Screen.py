from time import perf_counter
import sys
import pygame   as pg
import moderngl as mg
from   time     import time
from   array    import array
from   Shaders  import Fragment as f
from   Shaders  import Vertex   as v
from  .Config   import PickResolution, Title, Fullscreen, IntegerScale

Window : pg.Surface = pg.display.set_mode(PickResolution, pg.OPENGL | pg.DOUBLEBUF | pg.RESIZABLE) #Ventana
pg.display.set_caption(Title) #Titulo de la ventana

# Zoom entero: se dibuja a baja resolucion (~320x180, como el original) y OpenGL lo estira con NEAREST.
# Hd -> 4 (320x180), Fhd -> 6 (320x180), Basic -> 2 (320x240)
Zoom : int = max(1, min(PickResolution[0] // 320, PickResolution[1] // 180))

ctx = mg.create_context() #Contexto de OpenGL
QuadBuffer = ctx.buffer(data=array('f', [
    # position (x, y), uv coords (x, y)
    -1.0, 1.0, 0.0, 0.0,  # topleft
    1.0, 1.0, 1.0, 0.0,   # topright
    -1.0, -1.0, 0.0, 1.0, # bottomleft
    1.0, -1.0, 1.0, 1.0,  # bottomright
]))

Program = ctx.program(vertex_shader=v.value, fragment_shader=f.value) #Programa de shaders
RenderObj = ctx.vertex_array(Program, [(QuadBuffer, '2f 2f', 'vert', 'texcoord')]) #Objeto a renderizar

IsFullscreen : bool = False
_LastSize : list = [] #ultimo tamano informado por eventos de la ventana

def WindowSize() -> tuple[int, int]:
    """Tamano real de la ventana. Con OpenGL algunos sistemas no actualizan get_window_size al maximizar:
    se prefiere el tamano que trae el ultimo evento de redimension."""
    return _LastSize[0] if _LastSize else pg.display.get_window_size()

def ToggleFullscreen() -> None:
    """F11. Pantalla completa "de escritorio" (sin cambiar la resolucion del monitor) y sin perder el contexto OpenGL."""
    global IsFullscreen
    try:
        win = pg.Window.from_display_module()
        if IsFullscreen:
            win.set_windowed()
        else:
            win.set_fullscreen(desktop=True)
    except Exception: #pygame sin la API Window: el toggle clasico
        pg.display.toggle_fullscreen()
    IsFullscreen = not IsFullscreen

def Viewport(vw : int, vh : int) -> tuple[int, int, int, int]:
    """Rectangulo de la ventana donde va el canvas (vw x vh): escala entera si entra (pixeles nitidos),
    centrado, con bandas negras. (x, y, ancho, alto) con y desde arriba."""
    ww, wh = WindowSize()
    scale = min(ww / vw, wh / vh)
    if IntegerScale and scale >= 1: #pixeles perfectos (con bandas); si no, llena manteniendo la proporcion
        scale = int(scale)
    w, h = max(1, int(vw * scale)), max(1, int(vh * scale))
    return (ww - w) // 2, (wh - h) // 2, w, h

if Fullscreen:
    ToggleFullscreen()

MAX_DT : float = 0.1

class Screen():
    UserFps = None #FPS elegidos en la ventana de configuracion (None = los de cada pantalla); 0 = sin limite
    OnQuit = None  #se llama antes de cerrar la ventana (la pantalla de juego guarda la partida)

    def __init__(self, w: int, h: int) -> None:
        self.running : bool = True
        self.zoom : int = Zoom
        self.w : int = w // Zoom #resolucion virtual: se dibuja en pixeles del juego
        self.h : int = h // Zoom
        self.canvas = pg.Surface((self.w, self.h))
        self.clock = pg.time.Clock()
        self.fps : float = 60
        self.dt : float = 0.0 #segundos que duro el ultimo frame
        self.last : float = 0.0 #perf_counter del frame anterior
        self.vp : tuple = (0, 0, w, h) #donde se dibujo el canvas en la ventana (para el mouse)
        self.hires = None #superficie a resolucion completa (HiRes); si existe, se sube ella en vez del canvas
        self.hitex = None

        self.tex = ctx.texture((self.w, self.h), 4)
        self.tex.filter = (mg.NEAREST, mg.NEAREST)
        self.tex.swizzle = 'BGRA'

    def EventHandler(self, event, ClicklabelOBJ : dict) -> None:
        mouse = self.MousePos()
        for key, button in ClicklabelOBJ.items():
            button.EventHandler(event, mouse)
        if event.type == pg.KEYDOWN and event.key == pg.K_F11:
            ToggleFullscreen()
        if event.type in (pg.VIDEORESIZE, pg.WINDOWSIZECHANGED, pg.WINDOWRESIZED):
            w, h = (event.w, event.h) if event.type == pg.VIDEORESIZE else (event.x, event.y)
            _LastSize[:] = [(w, h)]
        if event.type == pg.QUIT:
            if Screen.OnQuit:
                Screen.OnQuit()
            self.running = False
            pg.quit()
            sys.exit()

    def MousePos(self) -> tuple[int, int]:
        x, y = pg.mouse.get_pos()
        vx, vy, vw, vh = self.vp
        return int((x - vx) * self.w // vw), int((y - vy) * self.h // vh)

    def HiRes(self) -> pg.Surface:
        """Superficie de w*zoom x h*zoom que se sube a la ventana en lugar del canvas. La usa la pantalla de juego:
        el mundo se escala ahi con su propio factor entero (zoom de camara) sin perder pixeles, y la UI encima.
        ponytail: sube ~3.7 MB por frame en HD; en Android convendria componer en OpenGL (dos texturas)."""
        if self.hires is None:
            self.hires = pg.Surface((self.w * self.zoom, self.h * self.zoom))
            self.hitex = ctx.texture(self.hires.get_size(), 4)
            self.hitex.filter = (mg.NEAREST, mg.NEAREST)
            self.hitex.swizzle = 'BGRA'
        return self.hires

    def ResetScreen(self) -> None:
        self.canvas.fill((0,0,0))

    def Update(self) -> None:
        src, tex = (self.hires, self.hitex) if self.hires is not None else (self.canvas, self.tex)
        tex.write(src.get_view('1'))
        ww, wh = WindowSize()
        ctx.viewport = (0, 0, ww, wh)
        ctx.clear(0.0, 0.0, 0.0) #bandas negras alrededor
        self.vp = Viewport(self.w, self.h)
        x, y, w, h = self.vp
        ctx.viewport = (x, wh - y - h, w, h) #OpenGL cuenta y desde abajo

        tex.use(0)
        Program['tex'] = 0
        RenderObj.render(mode=mg.TRIANGLE_STRIP)

        pg.display.flip()
        self.clock.tick(self.fps if Screen.UserFps is None else Screen.UserFps) #limita los FPS cada frame
        #dt medido con perf_counter: clock.tick devuelve milisegundos ENTEROS (33 en vez de 33.3, 0 sin limite), y ese
        #redondeo hacia que las animaciones cambiaran de velocidad con los FPS. Tope MAX_DT: al volver a una pantalla
        #(o tras un tiron) no se salta medio segundo de golpe.
        now = perf_counter()
        self.dt = min(now - self.last, MAX_DT) if self.last else 0.0
        self.last = now

    def FrameRate(self, fps) -> None:
        self.fps = fps

    def deltaT(self, old) -> float:
        return time() - old
