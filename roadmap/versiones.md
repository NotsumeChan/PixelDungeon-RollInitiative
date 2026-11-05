# Versiones

**Rust** (rama `main` / `dev`): la versión vive en el `Cargo.toml` del workspace (`game::VERSION` la lee de ahí) y en
`android/AndroidManifest.xml` (`versionName`, y `versionCode` sube de a uno o más, nunca baja). Regla: la versión de
Rust es la del prototipo cuyo contenido ya está portado completo. Al cerrar un paso del roadmap que la cambie, subirla
en los dos lugares y en la tabla de Rust.
**Parches** (pedido del dueño, 2026-11-05): cada cambio que se entrega, aunque sea chico (un arreglo, un paso como N2,
un ajuste de UI), sube el último número (0.9.0 -> 0.9.1 -> 0.9.2...) en los dos lugares, con `versionCode` + 1, una
fila en la tabla y su commit. **Un tema = una versión**: si el mismo tema necesita otro arreglo antes de pasar a otro,
va en esa misma versión (no se sube de nuevo el número). El número del medio sube con los objetivos del "Plan hacia la 1.0".

| Versión (Rust) | Pasos | Contenido |
|---|---|---|
| 0.1.0 | R0, R1 | Workspace, ventana y GPU, APK firmado, CI; generación de las Alcantarillas idéntica al prototipo |
| 0.2.0 | R2 | Render del mapa: tiles, agua, niebla, cámara, zoom, orientación vertical |
| 0.3.0 | R3 | Héroe, turnos y rondas 5e por iniciativa, mobs con IA, combate 5e, salvaciones contra la muerte, pisos 1-4 |
| 0.4.0 | R4, R5a | Personajes animados, dados animados, UI original (estado, iniciativa, barra, examinar, ficha, menú); ítems, mochila, accesos rápidos, pociones y pergaminos, hambre |
| 0.5.0 | R5 | Creación de personaje con dados (4d6, 7 tiradas, asignar o escribir), lista de personajes y guardado; ya estaban el combate 5e y la ficha |
| 0.8.0 | R5 | Hotseat (`[dev] hotseat`: 2 héroes en la misma PC, Tab, cambio automático al que le toca, nombres sobre los héroes): paridad con `python-prototipe`. La misma excepción que la 0.7 (partículas y proyectil en vuelo) |
| 0.7.0 | R5, R5d | Arrastrar en la creación (cierra la 0.6), armas a distancia, idioma del contenido, log de tiradas de desarrollador (`[dev] dice_log`). Excepción a la regla: faltan las partículas y el proyectil en vuelo (pulido visual, pendiente) |
| 0.9.15 | N5 | Scripts Lua en los mods (`script.lua`, `docs/mods.md`): condiciones y efectos `["lua", función, ...]` en los diálogos, en sandbox; el mod de ejemplo apuesta con un dado en Lua |
| 0.9.14 | N4 | Mercader ambulante como mod incluido (`data/mods/mercader-ambulante`): 25 % en cualquier piso (`"*"` en `npcs.json`), deambula (`"wander"`) salvo con un héroe al lado, mercancía nueva en cada piso |
| 0.9.13 | arreglos | Android vertical: la UI se corre un 2 % del alto hacia el centro arriba y abajo (los bordes curvos se comían la barra y el log), sin franjas negras: el mapa sigue a pantalla completa |
| 0.9.12 | N5 | Mods: carpetas en `<datos>/mods/` que pisan o suman registros (`docs/mods.md`; entran al hash de la red); mod de ejemplo `mods/ejemplo-herrero` (herrero troll en el piso 3 con tienda, misión y un ítem propio) |
| 0.9.11 | idioma | El juego en inglés: los textos del código (UI, menús, mensajes, descripciones) con `tr!`/`trf!`; antes la opción English sólo cambiaba los datos. Lo que arma el host (log, dados) llega en el idioma del host |
| 0.9.10 | N4 | NPCs: inventario y equipo como datos (pelean con él y lo sueltan al morir), precios en `items.json`, tienda desde el inventario del NPC (comprar y vender, la actitud mueve los precios); el Mercader con su mercancía |
| 0.9.9 | N3 | NPCs: misiones como datos (`data/quests.json`: matar o tener), en la ficha con su progreso, de la partida; el polvo de cadáver del Fabricante de varitas y las ratas del Mercader |
| 0.9.8 | arreglos | Mensaje de muerte partido al ancho de la pantalla; en Android sin teclas (la nueva partida está en el menú) y "Atras" en vez de "Esc" en los avisos. (Una 0.9.8 y 0.9.9 anteriores, del margen de los bordes, se deshicieron a pedido del dueño: el `versionCode` sigue en 18) |
| 0.9.7 | arreglos | Título en vertical: la versión va abajo al centro con margen (en la esquina la cortaba la curva de la pantalla) |
| 0.9.6 | arreglos | Tocarse a uno mismo parado sobre un ítem lo recoge (o abre el cofre / tumba de abajo) en vez de buscar; sin nada en el suelo sigue buscando |
| 0.9.5 | arreglos | El log (y el chat) parte las líneas largas al ancho de la pantalla; en vertical los dados chicos van arriba del log y ya no lo tapan |
| 0.9.4 | arreglos | Diálogo: la actitud del NPC va debajo del nombre (se pisaban). Dado grande: con un texto largo (pericias, conjuros) la escala baja hasta que entra en la pantalla |
| 0.9.3 | clases | Las 12 clases básicas de 5e: Bárbaro (Furia, Defensa sin armadura), Bardo, Clérigo, Druida (conjuros de CAR / SAB), Monje (Artes marciales, Defensa sin armadura), Paladín (Imposición de manos), Hechicero y Brujo (magia de pacto); conjuros de salvación, rasgos con usos por descanso |
| 0.9.2 | clases | Apariencia aparte de la clase: los 4 héroes de PD son skins (`data/skins.json`) que se eligen en la creación, se guardan con el personaje y viajan por la LAN; las clases salen de `data/classes.json` (sin lista fija en el código) y se eligen con flechas |
| 0.9.1 | N2 | NPCs: pericias 5e como datos (`data/skills.json`, `skill_prof` por clase, en la ficha), tiradas sociales en los diálogos (Perspicacia, Engaño, Interpretación, Persuasión, Intimidación), actitud de cada NPC (mueve la CD), atacar a un NPC hace desconfiar a los que lo vieron; Medicina con competencia |
| 0.9.0 | R6 | Red LAN para 2+ jugadores (PC y Android): crear / unirse con sala de espera, host autoritativo, efectos y dados en el cliente, quién falta elegir, personaje portátil (nivel, XP, oro y equipo viajan con el personaje), chat de texto. Además: NPCs con diálogos (N1), botones más altos y dados automáticos en la creación. Pendientes de R6 (no frenan la 0.9): aplicar el estado sin rearmar la partida, el teléfono como host, IP a mano |

(Las 0.2 y 0.3 no se marcaron en su momento: el `Cargo.toml` pasó de 0.1.0 a 0.4.0.)

**Prototipo en Python** (rama `python-prototipe`): la versión vivía en `core/Version.py`. **1.0 = multijugador P2P
listo.**

| Versión | Contenido |
|---|---|
| 0.1.0 | Base que arranca (fase 0) + generación de niveles de las Alcantarillas (fase 1) |
| 0.2.0 | Render: tiles, agua, campo de visión, niebla de guerra, cámara (fase 2) |
| 0.3.0 | Héroe, turnos, combate, mobs con IA, pisos 1–4 (fase 3) |
| 0.4.0 | Ítems, inventario, hambre, UI original, pantalla About (fase 4) |
| 0.5.0 | Capa D&D 5e: creación de personaje con dados, combate 5e, ficha |
| 0.6.0 | Mejoras: animación de dados, lista de personajes, drag and drop, pantalla completa, mochila, legibilidad, requisitos de FUE |
| 0.6.1 | El daño espera al dado, log de tiradas como opción de desarrollador, maximizar ajusta el contenido, arrastrar el mapa, Mods en el título, 2 accesos rápidos |
| 0.6.2 | Lista de personajes: JUGAR y BORRAR en gris si no hay personajes |
| 0.7.0 | Rasgos de clase y conjuros (Segundo Aliento, Ataque Furtivo, Mago, Explorador), salvaciones, trampas, estados como datos, armas a distancia (arrojadizas, bumerán, arco), subir de piso, menú y configuración (idioma, FPS, pantalla completa), examinar, animaciones de personajes y efectos, CA de mobs más baja, arreglos de niebla y de dt |
| 0.8.0 | Contenido en JSON (`data/`, con `"base"`), guardado de partidas por personaje, decisiones del multijugador, tooltips, accesos rápidos en orden, zoom de cámara nítido (rueda / pellizco), escala de la interfaz, vista recalculada al abrir puertas |

## Plan hacia la 1.0
| Versión | Objetivo |
|---|---|
| 0.7 | ✅ Rasgos de clase 5e, salvaciones, estados. La segunda pasada de ítems pasó a pendientes (`roadmap/mejoras.md`) |
| 0.8 | ✅ Guardado de partidas por personaje, registros en JSON (base para mods) |
| 0.9 | ✅ Red: descubrimiento LAN + partida para 2 jugadores (plan: `roadmap/fase5-red.md`). Falta la prueba del dueño del personaje portátil y el chat |
| **1.0** | **Multijugador P2P estable** |
| 1.x | Mods (`.pdmod`, instancias, sincronización estilo tModLoader), mapas por zonas, Android, internet (relay y servidores propios, no sólo PC) |
