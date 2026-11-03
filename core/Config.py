import os
import pygame       as pg
import configparser as cp
from   core.constants   import Resolutions
from   core.Version     import VERSION

#Leer desde "Local.config" para obtener configuraciones locales
Config = cp.ConfigParser()
Config.optionxform = str #conserva las mayusculas (gameTitle, maxFPS) al guardar
Config.read("Local.config")

Title : str = "Ventana de juego"
PickResolution : tuple = Resolutions["Hd"]
FrameRate : int = 30
Version : str = VERSION #la version es del codigo, no de la config local
Fullscreen : bool = False
IntegerScale : bool = False
DevDiceLog : bool = False
DevHotseat : bool = False #modo de prueba de la 0.9: 2 heroes en la misma PC (Tab cambia de heroe)
Language : str = "es"
CameraZoom : float = 1.0 #zoom de la camara del juego (rueda del mouse / pellizco)
UiScale : float = 1.0    #escala de la interfaz en partida, respecto del zoom base (ventana de configuracion)
if "general" in Config:
    Title = Config["general"]["gameTitle"]
    FrameRate = int(Config["general"]["maxFPS"])
    PickResolution = Resolutions[Config["general"]["resolution"]]
    Fullscreen = Config["general"].getboolean("fullscreen", fallback=False)
    IntegerScale = Config["general"].getboolean("integer_scale", fallback=False)
    Language = Config["general"].get("language", fallback="es")
    CameraZoom = Config["general"].getfloat("camera_zoom", fallback=1.0)
    UiScale = Config["general"].getfloat("ui_scale", fallback=1.0)
if "dev" in Config:
    DevDiceLog = Config["dev"].getboolean("dice_log", fallback=False) #log con el calculo de cada tirada
    DevHotseat = Config["dev"].getboolean("hotseat", fallback=False)

from game import lang as _lang
_lang.set_lang(Language)

def Save(key : str, value, section : str = "general") -> None:
    """Guarda una opcion en Local.config (la usa la ventana de configuracion)."""
    if section not in Config:
        Config[section] = {}
    Config[section][key] = str(value)
    with open("Local.config", "w") as f:
        Config.write(f)

pg.display.init()
Resolution = pg.display.Info() #Resolucion de la pantalla actual

#Agregar Opciones Graficas

#Agregar Opciones de Audio

#Teclado/input
InputType = {"k":"keyboard", "c" : "controller", "t" : "touch"}

Touch : dict = {} #ni idea de como hacer esto

Keyboard : dict = {"Foward": "K_w", "Left" : "K_a", "Right" : "K_d",  "Backward" : "K_s"} #agregar las keys de pygame para validaciond e inputs

pg.joystick.init()
if pg.joystick.get_count() > 0:
    Controller : dict =  None #agregar los botones del mando de pygame para validacion de inputs
pg.joystick.quit()

Input = Keyboard #input actual
