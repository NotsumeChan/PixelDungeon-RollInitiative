# Mejoras pedidas

Estado: ⏳ en curso / ✅ escrito (pendiente de revisión manual del dueño) / ✅ probado = el dueño lo probó y funciona.

## Lote 6 — versión 0.9.0 ⏳ (multijugador LAN) — plan en `roadmap/fase5-red.md`

| # | Paso | Estado | Dónde |
|---|---|---|---|
| 1 | Varios héroes en `game/` (vista, pisos, XP repartida + bonus al último golpe), modo de prueba en una PC (`[dev] hotseat`, Tab) | ✅ probado | `game/dungeon.py`, `game/hero.py`, `game/mobs.py`, `game/save.py` (formato v2), `screens/gameScreen.py` |
| 2 | Salvaciones contra la muerte 5e (caído, estable, daño masivo, estabilizar con Medicina, curar levanta). Volver a entrar con un personaje nuevo: con la red (pasos 5-6) | ✅ probado | `game/hero.py`, `game/char.py`, `game/spells.py`, `screens/gameScreen.py`, `screens/charSprite.py` |
| 2b | Reglas de la partida (estilo mundo de Don't Starve Together): daño por turno estando caído y "mitad si supera" de trampas. Mensajes: fallos por daño estando caído, "la mitad del efecto", recuadro del dado con de qué es la salvación | ✅ probado | `game/dungeon.py` (`RULES`), `screens/gameUi.py`, `game/hero.py`, `game/traps.py` |
| 2c | Dados de salvación decisivos grandes y al centro; d20 mediano; daño chico | ✅ probado | `screens/diceFx.py` |
| 2d | Dados más lentos (se veían 0,15 s con otro en cola; ahora 0,7 s mínimo) | ✅ probado | `screens/diceFx.py` |
| 2e | Sin fuego amigo: dardos, flechas y conjuros pasan por los aliados (antes el d20 se tiraba contra el aliado del medio); el enemigo detrás tiene media cobertura (-2). Click sobre un aliado ya no lo ataca | ✅ probado | `game/dungeon.py` (`shot`), `game/hero.py`, `game/spells.py` |
| 3 | Modelo de turnos: simultáneo fuera de combate, iniciativa en combate | ⏳ | |
| 4 | Red: descubrimiento LAN (UDP) + conexión (TCP, JSON por línea), host autoritativo | ⏳ | `net/` |
| 5 | Personaje portátil (estilo Terraria) | ⏳ | |
| 6 | UI: crear / unirse, sala de espera | ⏳ | |
| 7 | Hash de registros al unirse | ⏳ | |

## Lote 5 — versión 0.8.0 ✅ (guardado y registros en JSON) — cerrado

| # | Mejora | Estado | Dónde |
|---|---|---|---|
| 1 | Contenido en `data/*.json` (clases, mobs, bestiario, ítems, estados, conjuros) con `"base"` para custom | ✅ | `game/registry.py`, `data/` |
| 2 | Guardado de la partida por personaje en JSON: al salir, al cerrar y al cambiar de piso; se borra al morir | ✅ | `game/save.py`, `screens/gameScreen.py`, `screens/rosterScreen.py` |
| 3 | Decisiones del multijugador anotadas | ✅ | `roadmap/00-plan.md` |
| 4 | Tooltips con nombre y tecla en la barra, los accesos rápidos, el avatar y el menú | ✅ | `screens/gameUi.py` |
| 5 | Accesos rápidos en orden: 1 a la izquierda, 2 a la derecha | ✅ | `screens/gameUi.py` |
| 6 | Zoom de cámara con la rueda del mouse y el pellizco, guardado en la config. Nítido: escala entera a resolución completa, como el original (antes se pixelaba al alejar) | ✅ | `screens/gameScreen.py`, `core/Screen.py` |
| 7 | Guardado probado por el dueño: funciona | ✅ | |
| 8 | Escala de la interfaz en la configuración (50 / 75 / 100%), nítida | ✅ | `screens/gameUi.py`, `screens/gameScreen.py` |
| 9 | Bug: una puerta abierta por un mob (que moría ahí) dejaba la vista vieja y no se veía del otro lado: se recalcula la vista al abrir/cerrar puertas | ✅ | `game/char.py` |

## Pendientes (fuera de lote)
| # | Mejora | Dónde |
|---|---|---|
| — | Frases de la interfaz y del log en ambos idiomas (hoy sólo los nombres de contenido) | catálogo con `t()` |
| — | Segunda pasada de ítems: pociones y pergaminos con estados, varitas, anillos, semillas, bombas | `game/items.py` |
| — | Oleada de Acción (Guerrero), estilo de combate (Explorador), concentración y foco de conjuros | `roadmap/dnd5e.md` |

## Lote 4 — versión 0.7.0 ✅ (rasgos de clase, salvaciones, buffs) — cerrado; lo no hecho pasó a Pendientes

| # | Mejora | Estado | Dónde |
|---|---|---|---|
| 1 | Segundo Aliento (Guerrero): tecla F, 1d10 + nivel PV, se recarga en cada piso nuevo | ✅ | `game/hero.py`, `game/dungeon.py`, `screens/gameScreen.py` |
| 2 | Ataque Furtivo (Pícaro): +d6 con ventaja y arma sutil | ✅ | `game/hero.py`, `game/char.py` |
| 3 | Rasgos en la ficha | ✅ | `screens/gameUi.py` |
| 3b | Subir de piso: los pisos visitados se guardan tal cual (mobs, ítems, explorado) | ✅ | `game/dungeon.py`, `game/hero.py` |
| 3c | Escaleras sólo con click (confirmar), no al pisarlas | ✅ | `game/hero.py`, `screens/gameScreen.py` |
| 4 | Tiradas de salvación (5e) | ✅ | `game/char.py`, `game/hero.py`, `game/mobs.py` |
| 4b | Trampas: las 8 de las Alcantarillas se activan, con salvaciones | ✅ | `game/traps.py`, `game/dungeon.py` |
| 4c | Estados alterados como datos, base para mods (veneno, sangrado, quemadura, parálisis, lisiado) | ✅ | `game/buffs.py` |
| 4d | CA de los mobs 2 puntos más baja (los primeros pisos eran muy duros) | ✅ | `game/mobs.py` |
| 4e | Menú de la partida (botón de menú y Esc), en vez de salir directo al título | ✅ | `screens/gameUi.py`, `screens/gameScreen.py` |
| 4f | Indicador de ocupado (debajo del botón de menú) | ✅ | `screens/gameUi.py` |
| 4g | Íconos: Zzz y ! sobre los mobs, estados y hambre en la barra | ✅ | `screens/gameScreen.py`, `screens/gameUi.py` |
| 5 | Conjuros (datos, espacios 5e, línea de tiro, proyectiles): Mago y Explorador (Marca del cazador, Curar heridas) | ✅ | `game/spells.py`, `game/ballistica.py`, `screens/gameUi.py` |
| 5b | Armas a distancia: arrojadizas de PD (dardos, incendiario, curare, jabalina, shuriken, tomahawk), bumerán de la Cazadora, arco corto con flechas (sprites propios) | ✅ | `game/items.py`, `game/hero.py`, `screens/particles.py` |
| 4h | Salvación visible: el dado muestra "CON CD11" con el total y borde verde/rojo, más texto flotante | ✅ | `game/char.py`, `screens/diceFx.py` |
| 4i | Texto flotante: números de daño, esquiva, estados, curación, subir de nivel | ✅ | `game/dungeon.py`, `screens/gameScreen.py` |
| 4j | Buscar (lupa / Q): destello en las casillas revisadas y aviso si no hay nada | ✅ | `game/hero.py`, `screens/gameScreen.py` |
| 4k | Examinar: elegir casilla -> mob (descripción, estado, vida), ítem o terreno | ✅ | `screens/gameScreen.py`, `screens/gameUi.py`, `game/terrain.py` |
| 4l | El pasto alto se pisa (queda pasto, con hojas); sin semillas ni rocío | ✅ | `game/dungeon.py` |
| 4m | Nombres en el idioma elegido (español e inglés): ítems, apariencias, mobs, clases, estados, terreno | ✅ | `game/lang.py`, `game/items.py`, `game/mobs.py`, `game/hero.py` |
| 4n | Selección de idioma en la configuración global, guardada en `Local.config` | ✅ | `game/lang.py`, `core/Config.py` |
| 4o | Ventana de configuración (idioma, pantalla completa, FPS): desde el título y desde el menú de la partida, sin salir de ella | ✅ | `screens/gameUi.py`, `screens/homeScreen.py`, `core/Screen.py` |
| 4p | Frases de la interfaz y del log en ambos idiomas | ⏳ | catálogo con `t()` |
| 6 | Segunda pasada de ítems (pociones/pergaminos con estados) | ⏳ | |
| 7 | Animaciones de personajes: quieto, correr, atacar, morir (cadáver que se desvanece), operar y leer; el héroe según su armadura | ✅ | `screens/charSprite.py`, `game/mobs.py` |
| 7b | Cofres, tumbas y restos se abren desde una casilla vecina; lo de adentro queda en el suelo y se recoge caminando encima | ✅ | `game/hero.py` |
| 7c | Efectos visuales de trampas: humo tóxico/paralizante, llamas, veneno, gritos, rayos, tajo, aparición de invocados; el que está en llamas echa fuego | ✅ | `screens/particles.py` |
| 7d | Las animaciones cambiaban de velocidad con el límite de FPS: dt medido con `perf_counter` | ✅ | `core/Screen.py` |
| 7e | Personaje paralizado: animación congelada | ✅ | `screens/charSprite.py` |

## Lote 3 — versión 0.6.2

| # | Mejora | Estado | Dónde |
|---|---|---|---|
| 1 | JUGAR (y BORRAR) en gris y bloqueado si no hay personajes | ✅ | `screens/rosterScreen.py`, `screens/gameUi.py` (`DrawButton(..., enabled)`) |

## Lote 2 — versión 0.6.1

| # | Mejora | Estado | Dónde |
|---|---|---|---|
| 1 | El daño se aplicaba antes de que el dado mostrara el valor: ahora el ataque espera a la animación | ✅ | `game/char.py`, `game/dungeon.py`, `game/actor.py`, `screens/diceFx.py` |
| 2 | El cálculo de las tiradas en el log pasa a ser opción de desarrollador (`[dev] dice_log`) | ✅ | `core/Config.py`, `Local.config` |
| 3 | El About no lleva historial de versiones (queda en `roadmap/versiones.md`) | ✅ | `screens/aboutScreen.py`, `core/Version.py` |
| 4 | Maximizar la ventana no ajustaba el contenido | ✅ | `core/Screen.py` |
| 5 | Arrastrar con el click principal mueve la cámara para inspeccionar el mapa | ✅ | `screens/gameScreen.py` |
| 6 | Quitar Rankings y Badges del título; botón Mods | ✅ | `screens/homeScreen.py`, `screens/modsScreen.py` |
| 7 | 2 ranuras de acceso rápido: click = usar (o elegir si está vacía), click derecho = elegir desde la mochila | ✅ | `screens/gameUi.py`, `game/hero.py` |

## Lote 1 — versión 0.6.0 ✅
Animación de dados (1 = rojo, máximo = dorado con chispas), lista de personajes, drag and drop en la creación,
pantalla completa, legibilidad (espaciado de la fuente), identify no se elige a sí mismo, mochila con fila de
equipo y oro aparte, requisitos de FUE en escala 5e, versiones.
