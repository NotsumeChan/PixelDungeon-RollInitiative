# PixelDungeonMazes&Monsters

Remake en Python de Pixel Dungeon (pisos 1–4, Alcantarillas, sin jefe) con multijugador P2P.
Referencia Java original: `~/Documents/programacion/personal/java/pixel-dungeon/src/com/watabou/`
(`pixeldungeon/` = juego, `utils/` = Random, Rect, Graph, PathFinder...).

## Arranque
```bash
source venv/bin/activate.fish   # el shell es fish; en bash: source venv/bin/activate
python main.py                  # ejecutar desde la raíz (las rutas de Assets/ son relativas)
pip freeze > requirements.txt   # tras instalar algo nuevo
```
Dependencias: `pygame-ce` (no `pygame`) + `moderngl`. Python 3.14.

## Estructura
- `main.py` entrada · `Local.config` título/FPS/resolución/versión
- `core/` motor: ventana OpenGL (`Screen`, con `HiRes` para el zoom nítido), `Button`, `Font1x` (font original), `Config`
- `screens/` pantallas (título, lista y creación de personajes, juego, about, mods); cada una es un loop propio.
  `gameUi` (UI inmediata del juego y ventanas, incl. configuración), `charSprite` (animaciones), `particles` (efectos)
- `Shaders/` GLSL como strings · `Assets/` sprites/tiles/música del original (los propios, `pddnd_*`, en `docs/assets.md`)
- `game/` lógica pura, **sin pygame**: niveles, turnos, héroe, mobs, ítems, estados, conjuros, trampas, guardado
- `net/` red LAN (stdlib, sin pygame): `lan` transporte UDP/TCP, `actions` input como datos, `session` host/cliente
- `roadmap/` plan y documentación por fase — leer `roadmap/00-plan.md` primero
- `docs/` un `.md` por archivo de código, misma ruta (`docs/game/level.md` <-> `game/level.py`)
- `data/` contenido en JSON (registros). Guardados y personajes: carpeta de datos del usuario (`game/characters.py`)

## Reglas
- Se dibuja en resolución virtual (~320x180, `Screen.w/h`), en pixeles del juego; OpenGL escala con `Zoom`. Mouse: `screen.MousePos()`.
- Estado compartido entre pantallas/sistemas: `from game.globals import Globals` (dict). Anotar claves nuevas en `docs/game/globals.md`.
- Usar los assets originales (`java/pixel-dungeon/assets/`); copiar a `Assets/` lo que falte. Assets nuevos sólo si
  el dueño lo pide: prefijo `pddnd_` y anotados en `docs/assets.md`.
- Audio (música y sonidos) queda para el final: no agregarlo hasta que se pida.
- Todo lo que cambia con el tiempo (movimiento, animaciones, scroll, timers) se multiplica por el delta
  (`screen.dt`, en segundos): el juego debe verse igual a 30, 60, 120 FPS o sin límite. Nunca "x por frame".
  Default 30 FPS (`Local.config` -> `maxFPS`).
- Contenido como datos en `data/*.json` (`game/registry.py`; campos en `docs/data.md`): clases, mobs, bestiario, ítems,
  estados y conjuros. Se cambia el JSON, no el código; no crear una clase de Python
  por cada mob o héroe. Es la base del futuro sistema de mods (`roadmap/futuro-mods-personajes-red.md`).
- El proyecto deriva de Pixel Dungeon (GPL v3): si se distribuye, también es GPL v3.
- IDs de contenido siempre de texto `namespace:nombre`. El juego base usa `pddnd:` (nunca `pd:`, por temas de
  representación legal). Nada de IDs numéricos.
- Textos de contenido (nombres, descripciones): `str` o dict por idioma `{"es": ..., "en": ...}`, se muestran con
  `game.lang.t()`. No reemplazar un idioma por otro: agregar el que falte.
- Efectos de ítems referenciados por nombre (`EFFECTS` en `game/items.py`): un mod de datos no puede apuntar a código.
- Reglas de D&D 5e (dados, CA, competencias): `game/dice.py` + `roadmap/dnd5e.md`. Toda tirada usa un rng con seed.
- Versión: `core/Version.py`; historial sólo en `roadmap/versiones.md` (no en el About). Subirla al cerrar cada lote. 1.0 = P2P listo.
- Opciones de desarrollador en `Local.config` sección `[dev]` (`dice_log`; `hotseat` = 2 héroes en la misma PC, Tab cambia).
- Reglas de la partida (opciones de juego que se guardan con la partida, estilo mundo de Don't Starve Together):
  `RULES` en `game/dungeon.py` + fila en `RULE_ROWS` (`screens/gameUi.py`). Por defecto, lo que dice 5e.
- El azar puramente visual (animaciones, chispas) puede usar `random` global; nunca para resultados del juego.
- `game/` nunca importa pygame; el render lee el estado de `game/`, no al revés.
- Toda aleatoriedad pasa por un `random.Random(seed)` propio. Nunca el `random` global ni
  `Math.random()` del Java (aparece en `RegularLevel.split`, hay que reemplazarlo).
- Nada de `set` cuyo orden afecte la generación: usar listas, para que la misma seed dé el
  mismo nivel en todos los peers.
- Al portar desde Java, cuidado con la división entera (`49 / rect.square()` es int en Java).
- No hay tests automáticos: el dueño prueba a mano. No ejecutar tests, el juego ni scripts de verificación.
- Al terminar algo de una fase, actualizar su doc en `roadmap/`.
- Al crear o tocar un archivo de código, crear/actualizar su doc en `docs/` (misma ruta, `.md`): qué hace, de qué archivo Java viene, decisiones y trampas. No crear docs de archivos que no se han tocado.

## Roadmap
- `roadmap/00-plan.md` objetivo, arquitectura, fases, decisiones del multijugador
- `roadmap/fase4-items-ui.md` ítems, inventario, UI
- `roadmap/versiones.md` versiones y plan hasta la 1.0 · `roadmap/mejoras.md` lotes de mejoras y pendientes
- `roadmap/fase5-red.md` plan de la 0.9 (multijugador LAN): pasos y riesgos
- `roadmap/dnd5e.md` capa de reglas D&D 5e (decisiones, mapeo de equipo, pendientes)
- `roadmap/futuro-mods-personajes-red.md` mods, personajes, red local, crossplay (plan; registros JSON ya hechos)
- `docs/data.md` campos de los registros JSON · `docs/assets.md` assets propios
- `roadmap/fase0-base.md` · `roadmap/fase1-generacion.md` · `roadmap/fase2-render.md` · `roadmap/fase3-turnos.md`
