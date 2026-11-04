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
  `namespace:nombre` (`pdri:`), textos `{"es", "en"}`, todo por delta de tiempo, azar sólo por rng con seed.
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
| **R1** | `Rng`, geometría, terreno, salas, pintores, `SewerLevel` (pisos 1-4), registros JSON | `game/rng, geom, terrain, room, painters, level, registry` | Mismas seeds -> mismos mapas que Python (lo compara el dueño) |
| **R2** | Render del mapa: atlas, tiles, agua, niebla, cámara, zoom nítido, arrastrar el mapa | `screens/gameScreen` (fase 2) | El dueño recorre un piso con la cámara |
| **R3** | Actores: planificador por tiempo + rondas 5e (iniciativa, movimiento + acción), héroe, mobs con IA, FOV, pathfinder, dados, combate 5e, salvaciones contra la muerte | `game/actor, char, hero, mobs, fov, pathfinder, dice, dungeon` | Se juega un piso con combate |
| **R4** | Sprites animados, partículas, dados animados, UI (estado, carrusel de iniciativa, barra, mochila, ventanas, tooltips), sin sprite = nombre | `screens/gameUi, charSprite, particles, diceFx` | Igual que el prototipo 0.8, pantalla por pantalla |
| **R5** | Ítems, estados, conjuros, trampas, creación de personaje, lista de personajes, guardado JSON, reglas de la partida, hotseat | `game/items, buffs, spells, traps, save, characters`, pantallas | Paridad con `python-prototipe` = **0.8 en Rust** |
| **R6** | Red LAN (fase 5, pasos 4-7): descubrimiento, host autoritativo, personaje portátil, UI de partidas, hash de contenido, MulticastLock en Android | `net/`, `roadmap/fase5-red.md` | **0.9**: 2 jugadores en LAN, PC y Android |
| — | Estabilidad del multijugador | | **1.0** |

**Estado R0** ✅ (dueño, 2026-11-03): corre en el teléfono a 120 FPS (el tope del panel). Workspace (`game`, `app`),
winit 0.30 + wgpu 30, lote de sprites por instancias, fuente bitmap, APK firmado sin Gradle, CI.

**Estado R1** ✅ (2026-11-03): 224 pisos comparados con el prototipo (56 seeds x pisos 1-4), idénticos. `Rng`
compatible con CPython, `geom`, `terrain`, `room`, `patch`, `painters`, `level`, `registry`; volcado ASCII
(`docs/game/level.md`).

**Estado R2** ✏️ escrito, pendiente de prueba del dueño: FOV (`game/fov.rs`), visión y puertas en `Level`; capas de
mundo y UI con zoom entero cada una; agua animada, tiles y niebla con LINEAR en la GPU; cámara que sigue al observador,
arrastre, zoom con rueda / pellizco; observador con WASD o toque. Orientación vertical (pedido del dueño, sobre todo
para Android): la app gira con el teléfono y en vertical la UI usa la referencia 180x320.

**Estado R3** ✏️ escrito (2026-11-04), pendiente de prueba del dueño: `dice`, `pathfinder`, `actor` (planificador +
rondas 5e), `char`, `items` (arma y armadura iniciales), `hero` (clase de `data/classes.json`, ataque 5e, buscar,
escaleras arriba y abajo, estabilizar, salvaciones contra la muerte, regeneración, XP y niveles), `mobs` (IA SLEEPING /
WANDERING / HUNTING, bestiario, respawner, XP repartida) y `dungeon` (pisos visitados, varios héroes, `pending`). La
app juega con el héroe: click / toque con pathfinder, WASD, espacio = esperar, Q o tocarse = buscar, R / C = partida /
clase; personajes con el primer frame de su hoja; HUD con PV, CA, XP, ronda y turno, y el log.
Verificado headless (simulación descartable, no queda en el repo): 40 partidas sin pánicos ni bucles, combates, bajada
hasta el piso 4 y muertes por salvaciones; los pisos siguen idénticos a `main` (16 seeds x piso comparados).
Para probar: que los mobs despierten y persigan, el orden de iniciativa y el movimiento por turno en combate, caer a 0
PV y las salvaciones, subir de nivel, subir y bajar de piso (los pisos guardan sus mobs), el balance (el héroe muere
seguido si no descansa).
Pendientes de R3 que dependen de R4/R5: el dado animado (`pending` se resuelve en el acto), botones táctiles para
esperar / buscar, apariencias de pociones al empezar (gasta rng en el prototipo), loot, trampas, hambre.

**Siguiente: R4** (sprites animados, partículas, dados animados, UI) y después la parte de R5 que llega a la 0.7
del prototipo (rasgos de clase, conjuros, salvaciones, estados, trampas, armas a distancia).

**Pendiente del dueño** (al cerrar la sesión del 2026-11-03):
- `git push origin main` (R1, R2 y la orientación vertical están sólo en local).
- Probar R2 en el teléfono y en la PC (`cargo run --release`): niebla en bordes de paredes, agua, zoom nítido.
- Instalar el APK con la orientación vertical: quedó armado en `target/android/pdri.apk` pero el teléfono se
  desconectó (`adb install -r target/android/pdri.apk`). Si adb sigue cortándose: otro cable/puerto o depuración
  inalámbrica (`adb pair` / `adb connect`).
- Respaldar `~/.pddnd-signing/` (sin la clave no se pueden publicar actualizaciones del APK) y, si se quiere firmar
  en CI, cargar los secretos `PDRI_KEYSTORE_B64` y `PDRI_KEYSTORE_PASS` en GitHub.
- Opcional: borrar `~/pddnd-android/` (prueba de buildozer; el SDK/NDK de `~/.buildozer` sí lo usa
  `android/build.sh`) y `~/pddnd-backup-20261103-1116/`.

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
- **Tamaño del port** (~7.300 líneas de Python): las fases lo parten; cada una se compara a mano contra Python.

## Decisiones del dueño (2026-11-03)
1. **Pruebas**: el juego lo prueba el dueño a mano; sin tests de juego. Tests automáticos sólo para la red (R6).
2. **Guardados**: los del prototipo eran de prueba; Rust usa un formato nuevo, sin migración.
3. **Firma del APK**: desde R0, con clave propia fuera del repo (`~/.pddnd-signing/`, ver README).
4. README con meta, decisiones, cómo compilar y contribuir, y créditos; licencia GPL v3 (`LICENSE`).
