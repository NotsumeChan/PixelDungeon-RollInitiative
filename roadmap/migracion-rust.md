# Migración a Rust

Decisión del dueño (2026-11-03): el juego pasa a Rust. Motivo: todo el dibujo por GPU, nada de SDL, 200+ FPS en la
peor máquina, publicarlo en Linux, Windows y Android. La prueba de Android en Python falló donde importa: moderngl
depende de glcontext, que no tiene backend para Android (`glcontext/x11.cpp: 'X11/Xlib.h' file not found`).

La versión en Python queda congelada en la rama `python-prototipe`: es la **referencia** del port (como el Java de
Pixel Dungeon lo fue para Python). Sólo recibe arreglos si hacen falta para comparar.

## Qué se conserva tal cual
- `data/*.json` (clases, mobs, bestiario, ítems, estados, conjuros) y el sistema de registros con `"base"`.
- `Assets/` (los del original, GPL v3) y la regla: nada de sprites generados; sin sprite = el nombre.
- `roadmap/` (decisiones del dueño: 5e manda sobre PD, multijugador, reglas de la partida) y `docs/` como formato
  (un `.md` por archivo de código, misma ruta).
- Las reglas de `CLAUDE.md` que no son de Python: lógica sin dependencias de render, contenido como datos, IDs
  `namespace:nombre` (`pddnd:`), textos `{"es", "en"}`, todo por delta de tiempo, azar sólo por rng con seed.
- Licencia GPL v3 (agregar `LICENSE` antes de publicar).

## Stack
| Pieza | Crate | Por qué |
|---|---|---|
| Ventana e input | `winit` | Linux, Windows y Android (`android-activity`), sin SDL |
| GPU | `wgpu` | Vulkan / DX12 / Metal / GLES (fallback en Android y GPUs viejas) |
| Imágenes | `png` | sólo se cargan PNG de `Assets/` |
| Datos y guardados | `serde`, `serde_json` | los mismos JSON |
| Red | `std::net` + hilos | UDP broadcast + TCP, JSON por línea, como `net/` del prototipo; sin tokio para 2-4 jugadores |
| Android | `android-activity`, `jni` (MulticastLock), `xbuild` o `cargo-ndk` + Gradle | APK desde la misma base |

Audio, al final como siempre (cuando se pida: `kira` o `rodio`).

## Estructura (workspace de 2 crates)
- `game/` (lib, sin wgpu ni winit): port de `game/` de Python. Corre headless: base del host y de las pruebas.
- `app/` (bin + cdylib para Android): ventana, render, UI, pantallas, red. Lee el estado de `game`, nunca al revés.

`net` empieza como módulo de `app`; se separa en crate sólo si un servidor dedicado lo pide.

## Render (la meta: GPU para todo)
- Se dibuja en un render target de resolución virtual (~320x180, pixeles del juego) y se escala con factor entero
  y `NEAREST` a la ventana (lo que hacía `Screen` + `HiRes`).
- **Un atlas** por hoja (tiles, personajes, ítems, UI, fuente): los sprites de un frame van en **un buffer de
  instancias** (posición, recorte, color, espejo, alfa) y se dibujan en 1-2 draw calls.
- El mapa (tiles + agua) es un buffer estático: se sube al entrar al piso y se toca sólo cuando cambia una celda.
- Niebla: textura de 1 texel por celda (visible / vista / nunca vista), filtrada en el shader. Se actualiza al
  cambiar la vista.
- Texto: fuente bitmap del original (`Assets/Fonts/font1x.png`) como atlas; la UI también son quads en el mismo lote.
- `PresentMode::Immediate` / `Mailbox` sin vsync y tope de FPS opcional (`Local.config`), para medir los 200+.

## Determinismo (para el multijugador y para comparar con Python)
- `Rng` compatible con `random.Random` de CPython (MT19937, mismo seeding con entero, `random()` de 53 bits,
  `_randbelow` para `shuffle` / `choice`). Así, **misma seed = mismo nivel en Python y en Rust**: se comparan pisos
  enteros. Cuando el port esté completo se puede cambiar por otro rng (rompe guardados viejos).
- Nada de `HashMap` cuyo orden afecte resultados (en Rust el orden es aleatorio por proceso): `Vec`, `BTreeMap` o
  `IndexMap`. Es la regla del `set` de Python, más estricta.
- Ojo al portar: la división entera de Rust trunca hacia 0 como Java; `//` de Python redondea hacia abajo (difieren con
  negativos). Donde el prototipo usa `//` con valores que pueden ser negativos, replicar `div_euclid` / `floor`.

## Fases (cada una cierra con algo que el dueño prueba a mano)
| Fase | Contenido | Referencia en Python | Cierra cuando |
|---|---|---|---|
| **R0** | Workspace, ventana winit + wgpu limpiando pantalla, contador de FPS, CI (GitHub Actions: Linux, Windows, APK), **APK en el teléfono** | `core/Screen.py` | El dueño abre el APK y el binario de escritorio y ve los FPS |
| **R1** | `Rng`, geometría, terreno, salas, pintores, `SewerLevel` (pisos 1-4), registros JSON | `game/rng, geom, terrain, room, painters, level, registry` | Mismas seeds -> mismos mapas que Python (volcado comparado) |
| **R2** | Render del mapa: atlas, tiles, agua, niebla, cámara, zoom nítido, arrastrar el mapa | `screens/gameScreen` (fase 2) | El dueño recorre un piso con la cámara |
| **R3** | Actores: planificador por tiempo + rondas 5e (iniciativa, movimiento + acción), héroe, mobs con IA, FOV, pathfinder, dados, combate 5e, salvaciones contra la muerte | `game/actor, char, hero, mobs, fov, pathfinder, dice, dungeon` | Se juega un piso con combate |
| **R4** | Sprites animados, partículas, dados animados, UI (estado, carrusel de iniciativa, barra, mochila, ventanas, tooltips), sin sprite = nombre | `screens/gameUi, charSprite, particles, diceFx` | Igual que el prototipo 0.8, pantalla por pantalla |
| **R5** | Ítems, estados, conjuros, trampas, creación de personaje, lista de personajes, guardado JSON, reglas de la partida, hotseat | `game/items, buffs, spells, traps, save, characters`, pantallas | Paridad con `python-prototipe` = **0.8 en Rust** |
| **R6** | Red LAN (fase 5, pasos 4-7): descubrimiento, host autoritativo, personaje portátil, UI de partidas, hash de contenido, MulticastLock en Android | `net/`, `roadmap/fase5-red.md` | **0.9**: 2 jugadores en LAN, PC y Android |
| — | Estabilidad del multijugador | | **1.0** |

R0 va primero a propósito: Android fue lo que rompió Python. Si `wgpu` + `winit` no levanta en el teléfono, se sabe
antes de portar una sola línea de lógica.

## Qué NO se porta
- `pygame`, `moderngl`, los shaders GLSL 330 (se reescriben en WGSL), `core/` tal cual (se rehace sobre winit/wgpu).
- Los `ponytail:` del prototipo se revisan uno por uno al portar su archivo: algunos se resuelven gratis en Rust.

## Ramas y versiones
- `python-prototipe`: el prototipo congelado (0.8 + pasos 3-4 de la fase 5 escritos).
- `main`: el juego en Rust. El primer commit de R0 reemplaza ahí el código de Python (el historial se conserva;
  el prototipo sigue completo en `python-prototipe`).
- `dev` para el trabajo diario y `merge: dev -> main` al cerrar cada fase. Commits cortos: `feat:`, `fix:`, `merge:`.
- Versión: el Rust sigue la numeración (llega a 0.8 al cerrar R5). Historial en `roadmap/versiones.md`.

## Riesgos
- **wgpu en Android viejo**: si no hay Vulkan usa GLES 3; probar en el teléfono del dueño en R0.
- **Tooling de Android** (NDK, firma, Gradle): por eso va en CI desde R0.
- **Tamaño del port** (~7.300 líneas de Python): las fases lo parten; R1 se compara automático contra Python.
- **Pruebas**: el prototipo no tenía tests automáticos por decisión del dueño. Para R1 propongo una sola prueba de
  paridad (volcado de mapas por seed); el resto, a mano como siempre. Decide el dueño.

## Pendiente de decisión del dueño
1. ¿Prueba de paridad automática en R1 (y en R3 para el combate)? Propuesta: sí, sólo esas.
2. ¿Los guardados del prototipo deben poder cargarse en Rust? Propuesta: no (formato nuevo v3).
3. ¿El APK se firma con una clave propia ya en R0 o sólo debug hasta la 0.9?
