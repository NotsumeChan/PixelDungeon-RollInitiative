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

**Estado R4** ✏️ escrito (2026-11-04), pendiente de prueba del dueño:
- Personajes animados (`app/sprites.rs`): quieto, correr, atacar, operar y morir, girando hacia donde van; el héroe
  usa la fila de su armadura; cadáveres que se desvanecen; textos flotantes (daño, esquiva, caer, subir de nivel...);
  casillas revisadas al buscar; hojas del pasto; Zzz y ! sobre los mobs.
- Dados animados (`app/dice_fx.rs`): d20 de ataque, daño y salvaciones (grande en el centro); el daño se aplica
  cuando el dado muestra el número (`pending` ya no se resuelve en el acto).
- UI (`app/ui.rs`) con los assets originales: panel de estado (avatar -> ficha, PV, XP, nivel, piso, menú, ocupado),
  carrusel de iniciativa, barra con esperar / buscar / examinar, log, tooltips, ventanas de examinar (terreno o mob),
  ficha 5e y menú (Esc).
Para probar: que el daño caiga cuando se ve el dado, el carrusel al pasar los turnos, examinar terreno y mobs, la
ficha, los tooltips con el mouse, y en el teléfono que los botones se puedan tocar.
Quedan para R5 (dependen de ítems y estados): mochila, accesos rápidos, íconos de estados, partículas de trampas y
conjuros, configuración. El indicador de ocupado titila en vez de girar (el lote no rota sprites).

**Estado R5a** ✏️ escrito (2026-11-04), pendiente de prueba del dueño: ítems (`game/items.rs`: generador, montones,
apariencias sin identificar, nombres e info con los números de 5e), montones en el piso, cofres / tumbas / restos,
loot de los mobs, mochila de 19, oro, accesos rápidos (1 / 2, click derecho para elegir), comer, beber, leer (con
objetivo: identificar, mejorar), equipar / quitar / soltar con maldiciones, hambre, y en la UI la mochila, la ventana
del ítem y el ícono de hambre. Verificado: misma seed = mismas apariencias, mobs y montones que el prototipo (5 seeds x
2 clases). **Versión 0.4.0** (ver `roadmap/versiones.md`).
Para probar: recoger y abrir cofres, identificar una poción bebiéndola, leer identificar / mejorar eligiendo el
objetivo, equipar algo maldito, los accesos rápidos, pasar hambre.

**Estado R5b** ✏️ escrito (2026-11-04), pendiente de prueba del dueño: estados como datos (`game/buffs.rs`:
veneno, sangrado, quemadura, parálisis con salvación cada turno, lisiado), salvaciones de 5e con el dado grande,
las 8 trampas con sus salvaciones (`game/traps.rs`), reglas de la partida (`dot_death_saves`, `save_half`, por
ahora fijas), íconos de estados en el panel. Para probar: pisar trampas (ocultas y vistas), paralizarse, apagar el
fuego en el agua, que un mob paralizado no se mueva. Faltan las nubes de gas (Blob) y las partículas.

Arreglos tras la prueba del dueño (2026-11-04): los sprites recorren cada paso (no cortan paredes); atacar fuera de
combate inicia el combate con iniciativa y sorpresa (un mob que muere de un golpe igual tira iniciativa); la mochila,
menos brillante.

**R5e (en parte)** ✏️: configuración en `Local.config` desde el menú: tope de FPS, pantalla completa (F11) y controles
configurables. Falta el idioma (estructura de traducción).

**Estado R5c** ✏️ escrito (2026-11-04), pendiente de prueba del dueño: rasgos de clase (Segundo Aliento, Ataque
Furtivo, Enemigo predilecto) y conjuros del Mago (Descarga de fuego, Rayo de escarcha, Proyectil mágico, Dormir) y del
Explorador (Marca del cazador, Curar heridas) como datos, con espacios por nivel, trucos que escalan, línea de tiro y
cobertura (`game/spells.rs`, `game/ballistica.rs`), libro de conjuros (F) y el rasgo desde la ficha. Para probar: cada
conjuro y rasgo, que el daño espere al dado, Dormir sobre varios mobs, la Marca del cazador sumando 1d6, Ataque
Furtivo con daga contra un mob dormido. Faltan los proyectiles y partículas de los conjuros.

Arreglos tras la prueba del dueño (2026-11-04, 2a tanda): el acceso rápido de un arma arrojadiza ya no la suelta
(abre su ventana hasta que exista lanzar); pantalla de título (Jugar con elección de clase, Acerca de, configuración)
y "Menú principal" en el menú de la partida.

Menús como el prototipo (3a tanda): título con Jugar / Mods / Acerca de, menú de pausa con Reglas de la partida y
Menú principal | Salir, configuración con idioma (sólo español por ahora), FPS, pantalla completa, escala de la
interfaz y controles aparte; nombres nítidos en los ítems sin sprite. Pendiente que pidió el dueño: Jugar -> lista de
personajes y creación con dados (necesita el guardado de personajes, R5).

**Guardado y personajes (R5)** ✏️ (2026-11-04): partidas por personaje en JSON (`game/save.rs`, regenera los niveles
con la seed y guarda lo que cambió, combate en curso incluido), lista de personajes (`game/characters.rs` y la
ventana de la app: JUGAR / NUEVO / BORRAR), autoguardado al cambiar de piso, al menú principal, al salir y en Android
al pasar al fondo; si el héroe muere se borra. Misma carpeta de datos que el prototipo. Para probar: guardar en el piso
2, salir, volver a entrar y seguir; borrar con confirmación; morir y ver que la partida no queda. 

**Creación de personaje (R5)** ✏️ (2026-11-04), pendiente de prueba del dueño: NUEVO abre la creación con el layout
de `createScreen.py`: nombre, clase, 7 tiradas 4d6 (sin el menor) que se asignan tocando la tirada y luego la
característica, o escribiendo el número (1-20); TIRAR / LIMPIAR / EMPEZAR, Esc vuelve a la lista. En Android aparece el
teclado en pantalla al tocar el nombre o una característica. Sin arrastrar (el prototipo arrastraba): click-click hace
lo mismo. **Versión 0.5.0.** Para probar: tirar, asignar, mover una tirada de una característica a otra, escribir un
número, LIMPIAR, empezar sin nombre (usa el de la clase), en el teléfono el teclado y la creación en vertical.

**Estado R6** ✏️ escrito (2026-11-04), pendiente de prueba del dueño (PC + teléfono): red LAN en `app/src/net.rs`
(`docs/app/net.md`), sólo std. Lista de personajes -> CREAR LAN (juegas con el elegido y la partida queda abierta; el
log muestra la IP) o UNIRSE (lista de partidas de la red; tocar una entra con el personaje elegido). Host autoritativo
que manda el estado completo; el cliente manda acciones validadas. Descubrimiento: el cliente pregunta por broadcast y
el host contesta directo, así el teléfono encuentra la PC sin MulticastLock. Hash del contenido y versión: si no
coinciden, no se entra. Tests de red: 4, por loopback incluida una partida real (`cargo test -p app --lib net`).
Para probar (demo): en la PC abrir los puertos si ufw está activo (`sudo ufw allow 47810/udp` y
`sudo ufw allow 47811/tcp`), instalar el APK nuevo en el teléfono (`android/build.sh` + `adb install -r ...`), PC:
CREAR LAN; teléfono (misma Wi-Fi): UNIRSE, tocar la partida. Mirar: que aparezca la partida, entrar, moverse y pelear
los dos (en combate cada uno en su turno), bajar de piso por separado, que el teléfono se vaya y vuelva (vuelve a su
héroe), y que el host cierre (el teléfono vuelve a Unirse con el aviso).
Arreglos tras la primera prueba del dueño (PC + teléfono, 2026-11-04): sala de espera al crear (EMPEZAR / VOLVER;
el cliente ve quién está), lista de quién falta elegir acción y "Esperando a ...", teclado en pantalla para el nombre
en Android, botón del rasgo de clase en la barra, avisos del log sólo para quien hizo la acción, margen a los costados
en Android apaisado, ventanas que entran en vertical, VOLVER en la lista de personajes, Salir en el título, mostrar FPS
como opción y el botón Atrás de Android como Esc. Para probar: todo eso en el teléfono, en vertical y apaisado.
Pedido del dueño, después: **chat de texto** en la partida LAN (mensaje `chat` cliente -> host -> todos; en el
teléfono, con el teclado en pantalla).

Segunda tanda (2026-11-04): el teléfono ve animaciones, números de daño y dados (el host le manda los efectos y
dados de su piso); al cargar una partida para jugar solo, los héroes que quedaron de una sesión LAN se sacan (eran
las "copias" que aparecían sin hotseat).

Pendientes de R6: personaje portátil (paso 5: el cliente entra con nivel 1 y equipo inicial y al irse no se guarda su
progreso), aplicar el estado sin rearmar la partida
(hoy regenera los pisos en cada estado: puede trabar un poco en el teléfono), el teléfono como host (para recibir
los "discover" por broadcast puede necesitar MulticastLock: por ahora la partida se crea en la PC), escribir la IP a
mano si el router bloquea los broadcasts.

**Hotseat (cierre de R5, 0.8)** ✏️ (2026-11-04), pendiente de prueba del dueño: `[dev] hotseat = true` en
`Local.config` suma un segundo héroe igual al personaje ("<nombre> 2"). Tab cambia de héroe; la UI pasa sola al que
le toca en combate (o al que sigue en pie si el local cae) y el log avisa "Turno de ..."; con los héroes en pisos
distintos se cambia con Tab. Con 2+ héroes se ve el nombre sobre cada uno. **Versión 0.8.0** (paridad con el
prototipo, con la excepción de partículas y proyectil en vuelo). Para probar: crear un personaje nuevo con hotseat
(una partida guardada de antes sigue con un héroe), combate con los dos y el cambio automático, Tab, un héroe en otro
piso, un héroe caído (el otro lo estabiliza), guardar y volver a entrar.

**Cierre de la 0.7** ✏️ (2026-11-04), pendiente de prueba del dueño: arrastrar en la creación de personaje como el
prototipo (tirada -> característica, característica -> característica las cambia, afuera la quita; con mouse o dedo),
idioma del contenido en la configuración (Español / English: nombres y descripciones de los JSON; los textos del
código siguen en español, como el prototipo) y `[dev] dice_log = true` en `Local.config` para el log de tiradas
detallado. **Versión 0.7.0**, con una excepción a la regla de versiones: faltan las partículas y el proyectil en
vuelo (pulido visual). Para probar: arrastrar en la PC y con el dedo, un click sigue eligiendo; cambiar a English y
ver ítems / mobs / clases; `dice_log`.
Nota: "arrastrar ítems en la mochila" no existía en el prototipo (su drag and drop de la 0.6 era sólo en la
creación); se saca de pendientes.

**Estado R5d** ✏️ escrito (2026-11-04), pendiente de prueba del dueño: armas a distancia. LANZAR (primera acción de las
arrojadizas y del acceso rápido; se elige la casilla como un conjuro), el arco con flechas (tocar un mob a tiro
dispara; sin flechas o fuera de alcance, cuerpo a cuerpo con 1d4), el bumerán vuelve a la mochila, los dardos ponen su
estado al acertar (`on_hit`), cobertura de un aliado en medio (-2), desventaja más allá del alcance normal o con un
enemigo al lado, Ataque Furtivo a distancia. Lanzar o disparar fuera de combate lo inicia. Para probar: lanzar dardos
(los fallados quedan en el suelo, los acertados se pierden), un dardo de curare (parálisis), el bumerán, el arco con y
sin flechas, el aviso de cobertura con un aliado en medio, y el acceso rápido de una arrojadiza. Falta el proyectil en
vuelo (pulido, con las partículas).
Arreglo: en el título, "Acerca de" y "Opciones" ya no se tocan (separación por el texto más largo).

**Configuración (pedido del dueño, 2026-11-04)** ✏️: la resolución de la ventana (`resolution` de `Local.config`, que
Rust ignoraba) es una opción de la configuración en escritorio: 640x480 / 1280x720 / 1920x1080, se aplica al momento.
El botón de configuración de arriba a la izquierda del título pasó al dashboard ("Opciones", con el mismo ícono al
doble). Para probar: cambiar la resolución en ventana y en pantalla completa, y el título en vertical (4 botones).

## Para retomar (al cerrar la segunda sesión del 2026-11-04)

**Dónde está todo.** Rama `dev` (26 commits por encima de `main`, sin pushear: `git push -u origin dev`). `main`
sigue en R2 (`30fbe9f`). El merge `dev -> main` lo hace el dueño cuando valide las fases. **Versión 0.8.0**
(`Cargo.toml` del workspace y `android/AndroidManifest.xml`; regla e historial en `roadmap/versiones.md`; la 0.7 y la
0.8 se marcaron con una excepción: faltan partículas y proyectil en vuelo). El APK de la última tanda está instalado
en el teléfono del dueño y el binario de escritorio compilado (`cargo run --release`).

**Hecho en esta sesión** (todo ✏️ escrito y compilado, falta la prueba del dueño salvo lo que dice "probado"):
creación de personaje con dados (con arrastrar y teclado en pantalla en Android), resolución de la ventana en la
configuración, Opciones en el dashboard, armas a distancia (R5d), idioma del contenido, `[dev] dice_log`, hotseat,
y la red LAN (R6): CREAR LAN con sala de espera / UNIRSE, host autoritativo, efectos y dados en el cliente, lista de
quién falta elegir, avisos privados del log, Atrás de Android = Esc, Salir que cierra el proceso en Android, margen a
los costados en Android apaisado, ventanas que entran en vertical, mostrar FPS como opción. El dueño probó la red
con la PC y el teléfono: la partida aparece y se puede entrar (lo reportado se arregló en las dos tandas de arreglos
de "Estado R6").

**Siguiente, en este orden:**
1. **Que el dueño pruebe la última tanda** (PC + teléfono, vertical y apaisado): sala de espera, lista de quién
   falta, efectos y dados en el teléfono, teclado en pantalla, botón del rasgo, Salir y volver a abrir en Android,
   que no aparezcan copias de personajes. Lo que falle, primero.
2. **R6, personaje portátil** (paso 5 de `roadmap/fase5-red.md`): al unirse, el cliente manda su personaje con nivel,
   XP y equipo (hoy entra con nivel 1 y el equipo inicial) y al irse el host le devuelve el estado y el cliente lo
   guarda. Con eso y la prueba del dueño, **0.9**.
3. **Chat de texto** en la partida LAN (pedido del dueño; paso 8 de `fase5-red.md`).
4. Resto de R6: aplicar el estado sin rearmar la partida (el cliente regenera los pisos en cada estado: puede trabar
   en el teléfono), el teléfono como host (MulticastLock para recibir los "discover"), escribir la IP a mano.
5. **Idioma de los textos del código** (hoy en español, como el prototipo; el contenido ya se elige es / en).
6. Pulido: partículas y proyectiles en vuelo, nubes de gas.
7. **Publicar desde `main`** (pedido del dueño): GitHub Actions que, al llegar algo a `main`, compile Linux, Windows y
   el APK y los publique como **Release** (tag `v<versión>` de `Cargo.toml`, los 3 archivos adjuntos). Hoy
   `.github/workflows/build.yml` ya compila los 3 en cada push, pero sólo deja *artifacts* (vencen a los 90 días, no
   son descargas públicas). Falta: un job o workflow para `push` a `main` que junte los 3 y use
   `softprops/action-gh-release` (o `gh release create`), con `permissions: contents: write`; no repetir un tag que
   ya existe (subir la versión antes del merge). El APK tiene que ir firmado con la clave del dueño: cargar los
   secretos `PDRI_KEYSTORE_B64` y `PDRI_KEYSTORE_PASS` (ver "Pendiente del dueño"); sin ellos el CI firma con una
   clave descartable y ese APK no sirve para actualizar el instalado. Al dueño le da igual si compila GitHub o si
   compila su PC y sube los archivos (`gh release create v0.x.0 target/release/pdri target/android/pdri.apk ...`;
   el `.exe` de Windows sale más fácil de GitHub). Preferir GitHub: no depende de la PC encendida.

**Pendientes conocidos (pulido):** partículas de trampas y conjuros y proyectiles en vuelo (`screens/particles.py`),
nubes de gas que se expanden (Blob), el indicador de ocupado gira en el original (acá titila: `Sprite` no rota),
mimic, habilidades especiales de los mobs (Swarm, Skeleton, Thief), FLEEING, descansos (hoy se recupera todo al
llegar a un piso nuevo). En vertical, el dueño dijo que "cuesta interactuar": se arregló lo que se salía de la
pantalla; si sigue, pedirle qué pantalla.

**Cómo se trabaja (acordado con el dueño):** el dueño prueba el juego a mano; no correr simulaciones, bots ni
comparaciones de partidas, sólo compilar (`cargo clippy --workspace --all-targets` y
`cargo ndk -t arm64-v8a check -p app --lib`) y los tests de red (`cargo test -p app --lib net`, los únicos
automáticos; usan los puertos del juego). Commits en `dev`, cortos, a nombre del dueño, sin coautoría, con `#R<n>`;
el dueño pushea. Subir la versión al cerrar lo que la cambie. `Local.config` cambia cuando el dueño prueba opciones:
no commitearlo. Lo que el dueño tiene que revisar queda anotado en el "Estado" de cada paso.

**Pendiente del dueño** (actualizado al cerrar la segunda sesión del 2026-11-04):
- `git push -u origin dev`, probar y, cuando cierre cada fase, `merge: dev -> main`.
- Red LAN: con ufw activo en la PC, abrir los puertos (`sudo ufw allow 47810/udp` y `sudo ufw allow 47811/tcp`).
  La partida se crea en la PC (CREAR LAN) y el teléfono entra con UNIRSE, en la misma Wi-Fi.
- Probar en la PC (`cargo run --release`) y en el teléfono (`android/build.sh` y
  `adb install -r target/android/pdri.apk`; si adb se corta: otro cable/puerto o `adb pair` / `adb connect`). Qué
  mirar: el "Estado" de cada paso, arriba.
- Respaldar `~/.pddnd-signing/` (sin la clave no se pueden publicar actualizaciones del APK) y, para publicar desde
  `main` (punto 7 de "Siguiente"), cargar los secretos `PDRI_KEYSTORE_B64` (`base64 -w0 release.jks`) y
  `PDRI_KEYSTORE_PASS` en GitHub.
- Decidir si `Local.config` sigue versionado (cambia al probar opciones) o va al `.gitignore`.
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
