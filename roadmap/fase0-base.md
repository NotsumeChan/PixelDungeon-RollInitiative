# Fase 0 — Base que arranca ✅

## Entorno
- `venv/` con Python 3.14: `pygame-ce 2.5.8`, `moderngl 5.13.0` (ver `requirements.txt`).
- Recrear: `python3 -m venv venv && venv/bin/pip install -r requirements.txt`.

## Bugs corregidos
| Archivo | Problema | Arreglo |
|---|---|---|
| `core/Config.py` | `Version = Resolutions[version]` -> KeyError | lee el string directo |
| `core/Config.py` | `pg.init()`/`pg.quit()` al importar apagaba pygame (mixer incluido) | sólo `pg.display.init()` |
| `core/Button.py` | `self.IsHover` sin `()` (siempre True); `SetPos*` llamaban `rect.center(...)`; `SetHoverImg` pisaba `img`; `Set*Img` no escalaban | reescrito; usa `event.button` |
| `core/AnimationPlayer.py` | parámetro duplicado (SyntaxError), `pygame` sin importar, `_splitFrames` sin `self`, `animation_speed` vs `animationSpeed` | reescrito |
| `core/Player.py` | `LoadIcon` siempre lanzaba; `Draw` usaba `self.color` inexistente | placeholder mínimo (el héroe real va en fase 3) |
| `core/Screen.py` | rama `VIDEORESIZE` llamaba `self.Resize` inexistente | eliminada (la ventana no es redimensionable) |
| `core/Assets.py` | `pg.ANIMATIONS` no existe | `dict = {}` |
| `screens/homeScreen.py` | anotación `pg.Surface` con pygame importado como `py` | `Screen` |

## Verificado
- `compileall` sin errores.
- `python main.py` abre la ventana y queda en el loop del home.

## Pendiente conocido (no bloquea)
- `core/Screen.py` crea la ventana al importarse (efecto colateral). Si molesta para tests, mover a una función.
- Cada pantalla tiene su propio loop bloqueante; volver de `Game` al home = salir del loop de `Game`.
