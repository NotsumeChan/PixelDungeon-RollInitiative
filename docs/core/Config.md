# core/Config.py

Lee `Local.config` (desde la raíz del proyecto) y expone la configuración local.

| Variable | De `Local.config` | Default |
|---|---|---|
| `Title` | `gameTitle` | "Ventana de juego" |
| `FrameRate` | `maxFPS` | **30** |
| `PickResolution` | `resolution` (`Basic`, `Hd`, `Fhd`) | `Hd` (1280x720) |
| `Version` | `version` | "0.0.0" |

- `Resolution`: info de la pantalla del sistema (`pg.display.Info()`).
- `Keyboard` / `Input` / `Controller` / `Touch`: esbozos de mapeo de input, todavía sin uso.
- Al importar inicializa sólo el display de pygame. Antes hacía `pg.init()` + `pg.quit()`, lo que apagaba el mixer.

## Pendiente (roadmap)
Selector de FPS en opciones: 30 / 60 / 120 / sin límite. "Sin límite" = `clock.tick(0)`.

## Cambios 0.6.0
- `fullscreen` en `Local.config` (`Fullscreen`, por defecto false).
- `Version` ya no sale de `Local.config`: viene de `core/Version.py`.

## Cambios 0.6.1
| Clave | Sección | Para qué |
|---|---|---|
| `integer_scale` | `[general]` | `IntegerScale`: sólo escalas enteras al agrandar la ventana (por defecto false = llenar) |
| `dice_log` | `[dev]` | `DevDiceLog`: opción de desarrollador, el log muestra el cálculo de cada tirada |
| `hotseat` | `[dev]` | `DevHotseat`: modo de prueba de la 0.9, 2 héroes en la misma PC (Tab cambia de héroe) |

## Idioma y guardado
- `Config.optionxform = str`: conserva las mayúsculas de las claves (`gameTitle`, `maxFPS`) al guardar.
- `Language` (`[general] language`, por defecto `es`) se pasa a `game.lang` al importar.
- `Save(clave, valor, sección="general")`: guarda una opción en `Local.config` (reescribe el archivo). La usa la
  ventana de configuración.

## Zoom de cámara
`CameraZoom` (`[general] camera_zoom`, 1 por defecto): lo cambia y guarda la pantalla de juego.

## Escala de la interfaz
`UiScale` (`[general] ui_scale`, 1 por defecto): la cambia la ventana de configuración; la pantalla de juego la lee
cada frame.
