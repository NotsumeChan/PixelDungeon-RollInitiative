# Estado: lo escrito y qué probar

Lo que ya está en `dev`, fase por fase, con su **"Para probar"**: lo que el dueño revisa a mano. ✅ = probado por el
dueño, ✏️ = escrito y compilado, falta su prueba. Lo que falta hacer está en [`pendientes.md`](pendientes.md); lo que
sigue, en [`README.md`](README.md).

## Rust (R0-R6)
| Fase | Contenido | Referencia en Python | Cierra cuando |
|---|---|---|---|
| **R0** | Workspace, ventana winit + wgpu limpiando pantalla, contador de FPS, CI (GitHub Actions: Linux, Windows, APK), **APK en el teléfono** | `core/Screen.py` | El dueño abre el APK y el binario de escritorio y ve los FPS |
| **R1** | `Rng`, geometría, terreno, salas, pintores, `SewerLevel` (pisos 1-4), registros JSON | `game/rng, geom, terrain, room, painters, level, registry` | Mismas seeds -> mismos mapas que Python (lo compara el dueño) |
| **R2** | Render del mapa: atlas, tiles, agua, niebla, cámara, zoom nítido, arrastrar el mapa | `screens/gameScreen` (fase 2) | El dueño recorre un piso con la cámara |
| **R3** | Actores: planificador por tiempo + rondas 5e (iniciativa, movimiento + acción), héroe, mobs con IA, FOV, pathfinder, dados, combate 5e, salvaciones contra la muerte | `game/actor, char, hero, mobs, fov, pathfinder, dice, dungeon` | Se juega un piso con combate |
| **R4** | Sprites animados, partículas, dados animados, UI (estado, carrusel de iniciativa, barra, mochila, ventanas, tooltips), sin sprite = nombre | `screens/gameUi, charSprite, particles, diceFx` | Igual que el prototipo 0.8, pantalla por pantalla |
| **R5** | Ítems, estados, conjuros, trampas, creación de personaje, lista de personajes, guardado JSON, reglas de la partida, hotseat | `game/items, buffs, spells, traps, save, characters`, pantallas | Paridad con `python-prototipe` = **0.8 en Rust** |
| **R6** | Red LAN (fase 5, pasos 4-7): descubrimiento, host autoritativo, personaje portátil, UI de partidas, hash de contenido, MulticastLock en Android | `net/`, `roadmap/fase5-red.md` (rama `python-prototipe`) | **0.9**: 2 jugadores en LAN, PC y Android |
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

Arreglos en la creación (pedido del dueño, 2026-11-05) ✏️: botones de abajo de la lista de personajes y de la
creación más altos (24 en vez de 16, para el dedo); NUEVO ya tira los dados y las tiradas más altas van a las
características de la clase (`ability_priority` en `classes.json`: Guerrero FUE-CON, Mago INT-CON, Pícaro DES-CON,
Explorador DES-SAB); TIRAR reparte igual y cambiar de clase reacomoda si no se tocó nada a mano. Para probar: tocar los
botones en el teléfono, NUEVO con cada clase, cambiar de clase antes y después de mover una tirada.

**Clases y apariencias** (pedido del dueño, 2026-11-05): las 12 clases básicas de 5e, y la apariencia aparte de la
clase. ✏️ 0.9.2: los 4 héroes de PD son skins (`data/skins.json`, ids = los de sus clases de antes: un personaje viejo
se ve igual), elegidas en la creación con un selector con flechas, guardadas en el personaje y el héroe y mandadas en
el `hello` de la LAN; las clases salen de `data/classes.json` (las de un mod aparecen solas). Para probar: crear un
personaje con otra skin que su clase (p. ej. Mago con la Cazadora) y verlo en el mapa, la lista y la LAN; un
personaje de antes sigue igual.
✏️ 0.9.3: las 8 clases que faltan, con lo de nivel 1 de 5e:
- Bárbaro d12: Furia (acción adicional, 10 turnos: +2 al daño cuerpo a cuerpo y la mitad del daño de los ataques;
  2 usos, +1 cada 3 niveles) y Defensa sin armadura (CON).
- Bardo d8 (CAR): Burla cruel, Curar heridas, Dormir.
- Clérigo d8 (SAB): Llama sagrada, Saeta guía, Curar heridas.
- Druida d8 (SAB): Producir llama, Rociada venenosa, Curar heridas.
- Monje d8: Artes marciales (FUE o DES con armas simples o sin armas; puño 1d4) y Defensa sin armadura (SAB).
- Paladín d10: Imposición de manos (5 PV por uso, usos = nivel).
- Hechicero d6 (CAR): Descarga de fuego, Rociada venenosa, Proyectil mágico, Dormir.
- Brujo d8 (CAR, magia de pacto): Descarga sobrenatural y Maleficio.

Cada una con sus salvaciones, competencias de armas, armaduras y pericias, equipo inicial y orden de
características. Los rasgos se ven y se usan desde el libro (F / el botón de la barra).
Ponytail, para después: Inspiración bárdica, Sentido divino, Canalizar divinidad, golpe extra del Monje, Forma
salvaje, Castigo divino, invocaciones del Brujo, metamagia; la Furia no exige FUE ni corta los conjuros.
Para probar: crear una de cada una, su CA sin armadura (Bárbaro y Monje), la Furia (daño +2 y la mitad del daño
recibido, el estado en la ficha), los conjuros de salvación (el dado verde o rojo del mob), Imposición de manos a un
aliado caído, que el Brujo tenga 1 espacio en nivel 1 y 2 en nivel 2.

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

**Personaje portátil (R6, paso 5)** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: UNIRSE manda el nivel, XP,
oro y equipo de la partida del personaje (`save::portable`); el host los valida (rangos básicos, ítems que existen,
equipo en su ranura; si no, "personaje invalido") y su héroe entra con eso, descansado y con los PV al máximo. Al
cambiar de piso, salir, ir al menú o cortarse la conexión, el cliente pasa su héroe a su partida de un jugador (o a
una nueva); si murió en la LAN, la partida se borra como al jugar solo. Tests de red: 6. Para probar: un personaje de
nivel 2+ con equipo entra así a la partida de la PC; subir de nivel o juntar algo en la LAN, salir y JUGAR solo con
ese personaje: lo tiene (en el piso donde había quedado su partida); un personaje nuevo que entra a la LAN y sale
queda "en partida".

**Chat de texto (R6, paso 8)** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: en una partida LAN, Enter (o
el botón "Chat" de la barra, en el teléfono con el teclado en pantalla) abre una línea arriba; Enter / LISTO manda y
Esc cancela. El mensaje sale en el log de todos como "<nombre>: <texto>" (hasta 40 caracteres). Para probar: escribir
desde la PC y desde el teléfono y verlo en los dos; Esc / Atrás cancela; que Enter no abra el chat jugando solo.

Pendientes de R6: aplicar el estado sin rearmar la partida
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

## NPCs y mods (N1-N5)

Pedido del dueño (2026-11-05): NPCs como en D&D (hablar, misiones, comerciar, persuadir, mentir, intimidar,
coquetear o atacar, con tiradas 5e), todo como datos. Decisiones en [`decisiones.md`](decisiones.md#npcs-2026-11-05).

### Modelo (propuesta)
- **NPC = mob con diálogo**: en `data/mobs.json`, `"npc": "<id del diálogo>"` y `"disposition"` inicial. Así reusa
  sprite, animaciones, guardado, red y combate. IA nueva `State::Passive`: no caza, deambula cerca de su casilla
  o se queda quieto. Tocarlo = **Hablar** (`Action::Talk(mob)`); en combate no se habla.
- **Disposición** (actitud de 5e: hostil / indiferente / amistoso, como número -2..2): la mueven las tiradas
  sociales y las acciones. Cambia las CD y lo que ofrece el diálogo. Atacarlo lo vuelve hostil (pasa a
  `Hunting`) y baja la disposición de los NPC que lo vean.
- **Diálogos** (`data/dialogs.json`): árbol de nodos. Nodo = `text` + `options`. Opción = `text`, `if`
  (condiciones), `check` (tirada: `{"skill", "dc", "ok", "fail"}`), `do` (efectos), `go` (nodo siguiente o fin).
  - Condiciones: `has` [ítem, n], `gold` n, `flag` [clave, op, valor], `quest` [id, estado], `disposition` n,
    `class` id, `level` n.
  - Efectos: `give` / `take` [ítem, n], `gold` ±n, `flag` [clave, valor], `quest` [id, estado], `disposition` ±n,
    `hostile`, `shop` id, `log` texto (pistas), `map` (revela el piso), `end`.
  - Textos `{"es", "en"}` como el resto del contenido.
- **Pericias 5e** (`data/skills.json`: id -> característica y nombre; Persuasión, Engaño, Intimidación,
  Interpretación, Perspicacia, ...) y `skills` (competencias) en `classes.json`. Tirada = d20 + mod + competencia
  contra la CD, con el dado grande (como las salvaciones). Coquetear = Persuasión o Interpretación, según el
  diálogo.
- **Misiones** (`data/quests.json`): nombre, descripción, objetivos como condiciones (`kill` [mob, n] con el
  contador `kills:<mob>`, `has` ítem, `flag`) y recompensa (efectos). Ventana de misiones en la ficha.
- **Inventario y equipo de los NPC** (pedido del dueño, 2026-11-05): cada NPC lleva mochila, arma y armadura (como el
  héroe): en `mobs.json`, `inventory` ([ítem, cantidad] o sacado del generador) y `weapon` / `armor`. Pelea con su
  equipo, se guarda con el piso y, si muere, lo suelta. Un diálogo puede dar o pedirle cosas a su inventario.
- **Comercio**: la tienda **vende lo que el NPC tiene en su inventario** (así el mercader fijo y el ambulante son lo
  mismo); precio = `price` de `items.json` (nuevo campo), modificado por la disposición. Vender = la mitad y el ítem
  pasa a su inventario. Ventana de tienda con las dos mochilas.
- **Mercader ambulante** (hecho, 0.9.14: mod incluido `data/mods/mercader-ambulante`; se reabastece con mercancía nueva en cada piso donde aparece): un NPC con tienda que deambula (`State::Passive` que camina) y puede aparecer en
  cualquier piso; reabastece su inventario cada tanto.
- **Colocación**: `npcs` por piso (como `bestiary.json`), con un rng propio derivado de la seed del piso: no gasta el
  rng del nivel, así los pisos y los mobs siguen saliendo iguales.
- **Red**: `Talk` y `Choose(k)` son acciones como las demás (el host las valida). El diálogo abierto va en el estado
  del héroe: cada jugador ve el suyo y lo importante sale en el log de todos ("X convence a Y").
- **Guardado**: disposición, flags, contadores y misiones.

### Contenido base (del original, con sus sprites)
- **Fabricante de varitas** (`Wandmaker.java`, `wandmaker.png`): el NPC inicial. En el original aparece en las Prisiones
  y pide un objeto a cambio de una varita; acá, en las Alcantarillas, con su misión adaptada.
- **Mercader** (`shopkeeper.png`): en el original aparece en el piso 6; acá, uno en las Alcantarillas.
- Fuera por ahora (decisión del dueño): el Fantasma triste (`Ghost.java`).
- Para después (sprites que ya están en `Assets/Characters/`): blacksmith, imp, ratking, ghost.

### Estado
- **N1** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: NPCs tranquilos, Hablar, diálogos como datos con
  condiciones y efectos, flags de la partida, ventana de diálogo, guardado y red (`docs/game/npcs.md`). Contenido: el
  Mercader en el piso 1 (vende una ración por 10 de oro y el mapa del piso por 30, un rumor) y el Fabricante de
  varitas en el piso 2 (consejos, el ingrediente que busca, regala una poción de curación una vez por partida). Los
  dos se pueden atacar (opción [Atacar], o pegarles). Para probar: tocar al NPC (se acerca y habla), cada opción, que
  las de oro no aparezcan sin oro, el regalo una sola vez (también tras guardar y volver), Esc / Atrás cierra,
  atacarlo (empieza el combate con él sorprendido), que no corte la caminata ni entre al combate de otros mobs, y en
  la LAN que cada uno vea su propio diálogo.

- **N2** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: las 18 pericias de 5e como datos
  (`data/skills.json`) y sus competencias por clase (`skill_prof`: Guerrero Atletismo e Intimidación, Mago Arcanos y
  Perspicacia, Pícaro Engaño, Persuasión, Sigilo y Percepción, Explorador Percepción, Supervivencia y Perspicacia), en
  la ficha; opciones con tirada (`check`) que van a un nodo u otro según el resultado, con el dado grande y el
  resultado en el log de todos; actitud de cada NPC (de hostil a muy amistoso, se ve en la ventana, mueve la CD 2 por
  punto); atacar a un NPC hace desconfiar a los que lo vieron. Medicina ya suma la competencia. Contenido: el
  Fabricante (observarlo con Perspicacia, mentirle con Engaño, coquetearle con Interpretación) y el Mercader
  (regatear con Persuasión, intimidarlo; si es amistoso, un regalo). Cada intento, una vez por partida. Para probar:
  cada tirada con una clase con y sin competencia (el bono del botón cambia), que mentir mal lo vuelva desconfiado y
  suba la CD de lo demás, que regatear bien habilite "¿Algo especial para un amigo?", pegarle a uno delante del otro
  (en pisos distintos no se ven: probarlo con un mod o más adelante), y el dado y el log en la LAN.

- **N3** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: misiones como datos (`data/quests.json`) con objetivos
  de matar (contador por mob) o de tener un ítem, estado de la partida (compartido en la LAN), en la ficha con su
  progreso, y aceptar / revisar / entregar desde los diálogos. Contenido: el Fabricante de varitas (piso 2) pide polvo de
  cadáver, que aparece en unos restos de esqueleto de su piso al aceptar; lo paga con un pergamino de mejora y una
  poción de curación (y se vuelve más amistoso). El Mercader (piso 1) paga 40 de oro por 5 ratas. Para probar: aceptar
  cada una, ver el progreso en la ficha (las ratas cuentan desde que se acepta), abrir los restos y recoger el polvo,
  entregar; guardar y volver a entrar a mitad; en la LAN, que la misión sea del grupo.

- **N4** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: inventario y equipo de los NPC como datos (fijo y al
  azar del generador), con el que pelean y que sueltan al morir; precios en `items.json`; tienda desde el inventario
  (comprar de a una unidad, vender de a una; lo vendido queda en su inventario; la actitud mueve los precios); `give` y
  `take` usan su inventario. El Mercader vende raciones, pociones de curación, pergaminos de identificar, dardos,
  armadura de cuero y algo al azar (2 pociones, 2 pergaminos, 1 arma); pelea con espada corta y armadura de cuero. El
  Fabricante de varitas lleva un bastón. Para probar: "Veamos qué tienes" abre la tienda, comprar con y sin oro, vender
  algo y volver a comprarlo, regatear bien (más barato) o intimidarlo (más caro), la mochila llena, matarlo y ver que
  suelta su mercancía, y en la LAN que cada uno compre de la misma tienda.
  Mercader ambulante ✏️ (0.9.14, 0.9.17, 0.9.19): mod incluido `data/mods/mercader-ambulante`; aparece junto a la
  bajada con 25 % en el piso 1, 5 % menos por piso (0 desde el 6), mercancía nueva en cada piso; ajustes "Frecuencia
  (%)" y "Deambula" (si no, se queda donde aparece). Para probar: bajar varios pisos (en el 1-5 a veces está junto a
  la bajada), la tienda, la tirada de Perspicacia, la frecuencia en 0 (no aparece) y 200, y "Deambula" en No.

- **N5** ✏️ escrito (0.9.12, 0.9.15, 0.9.16, 0.9.18), pendiente de prueba del dueño (`docs/mods.md`):
  - Carga de mods de `<datos>/mods/<mod>/` (registros que pisan o suman al base; entran en el hash de la LAN) y mods
    incluidos (`data/mods/`); `npcs.json` con `"*"`, probabilidad por profundidad, `near` y `scale`.
  - Scripts Lua (`script.lua`, `game/src/script.rs`, mlua): condiciones y efectos `["lua", ...]` de los diálogos, en
    sandbox (sin archivos, tope de instrucciones y memoria), `ctx` con héroe, NPC, piso, flags y ajustes.
  - Importar un zip con el selector de archivos del sistema (escritorio `rfd`, Android `MainActivity.java`: el APK
    ya tiene código Java, `android/build.sh` lo compila con javac + d8); aviso de responsabilidad una sola vez.
  - Lista de mods: ícono (`icon.png`), `mod.json` (nombre, descripción, autor, versión), ON / OFF (al reiniciar) y
    AJUSTES por mod (`settings` int / bool, en Local.config, en caliente).
  - Mod de ejemplo `mods/ejemplo-herrero`: herrero troll en el piso 3 (sprite `blacksmith.png`, cuadro 14x16
    estimado: revisar que se vea bien), tienda, misión de 3 gnolls, ítem propio con `base`, Persuasión y una
    apuesta en Lua con sus ajustes.
  - Para probar: comprimir `mods/ejemplo-herrero` en un zip, importarlo en el teléfono y en la PC, reiniciar, el
    herrero en el piso 3 (apuesta, ajustes), desactivar el mercader ambulante, y en la LAN los dos con los mismos mods.

## Sesión 2026-11-06 (tarde): M1c, separación de PD, controles, combate, mods, L1 (0.9.34-0.9.45)
Todo ✏️. El "Para probar" detallado de cada versión está en [`README.md`](README.md#qué-probar-todo--lo-que-falle-va-primero)
y en [`versiones.md`](versiones.md); los docs de cada archivo, en `docs/`.
- **M1c** (0.9.34): bordes abiertos hacia la zona de al lado (solos si existe; `north` / `south` / `east` / `west`
  cambian o cierran; sin zona al lado, pared invisible) y portales (`portals`); `Exit`, `Level::edges`,
  `Dungeon::edge_to` / `leave`, acción de red `leave`. **M1d** a medias (0.9.42): nombre y coordenadas en la barra.
- **Separación de PD** (0.9.35-0.9.36): turnos fuera de combate propios y simultáneos; aliados que no cortan el
  camino (cambio de lugar o espera); pathfinder sin zigzag (desempate por la recta al destino).
- **Controles** (0.9.37, 0.9.38, 0.9.44): tocarse usa lo de abajo; acción rápida, Seguir, arrojadizas que apuntan
  solas; teclas E / X.
- **Combate** (0.9.39, 0.9.41): entran sólo los que pelean o se ven; termina en el momento; XP al final con resumen
  y guardada a mitad de pelea (`PROTOCOL` 4).
- **Mods** (0.9.45): lista a todo el ancho, texto recortado, menú por mod con BORRAR, scroll.
- **L1 / L2** (0.9.40, 0.9.43): textos sin D&D y avisos (README, Acerca de); `class_5e` (alias `dnd_class`);
  `roadmap/reglas5e.md`.
- **Planes nuevos** (sólo documentos): [P](pasos/7-progresion-5e.md), [C](pasos/8-personajes-mundos.md),
  [N6](pasos/9-packs-mods.md), [L](pasos/10-nombre-avisos.md); decisiones en `decisiones.md`.

