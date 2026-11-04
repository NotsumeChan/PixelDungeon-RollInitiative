# Fase 5 — Multijugador (versión 0.9: LAN, 2 jugadores)

Decisiones del dueño: `roadmap/00-plan.md` ("Decisiones del multijugador"). Resumen: host autoritativo; en combate
iniciativa 5e y fuera de combate turnos simultáneos; cada jugador puede estar en un piso distinto; salvaciones contra
la muerte 5e (si muere, puede volver a entrar con un personaje nuevo); el botín es de quien lo agarra; XP repartida con
un bonus para el último golpe; sólo LAN en la 1.0; cada jugador guarda su personaje (estilo Terraria).

## Punto de partida (0.8)
- `game/` ya es lógica pura y determinista, con un planificador por piso (`Dungeon.floors`) y guardado en JSON
  (`game/save.py`), que sirve de base para mandar el estado inicial por red.
- Todo supone **un solo héroe** (`dungeon.hero`): `game/dungeon.py` (7 usos), `game/hero.py` (4), `game/save.py` (5),
  `game/char.py`, `game/mobs.py`, `game/traps.py` (1 c/u); pantallas: `gameUi.py` (9), `gameScreen.py` (6),
  `rosterScreen.py` (3), `createScreen.py`, `charSprite.py` (1). La vista (`level.visible`) también es una sola.

## Estado
- Paso 1 ✅ probado por el dueño (hotseat): `Dungeon.heroes` con piso y vista por héroe, piso
  "en contexto" (`Dungeon.enter`), mobs que eligen al héroe más cercano que ven, XP repartida (+1 al último golpe),
  regeneración y hambre que viajan con el héroe, guardado v2 (migra v1). Modo de prueba: `[dev] hotseat = true`.
  Detalle en `docs/game/dungeon.md` ("Varios héroes").
- Paso 2 ✅ probado por el dueño: salvaciones contra la muerte (`docs/game/hero.md`). Click sobre un
  aliado caído = estabilizar (Medicina); Curar heridas pasa a ser de toque. Volver con un personaje nuevo queda para
  cuando exista unirse a una partida (pasos 5-6).
- Extras probados: reglas de la partida (Esc -> Reglas de la partida: daño por turno estando caído, "mitad si supera"),
  dados decisivos grandes y más lentos, sin fuego amigo a distancia (media cobertura). Ver `roadmap/mejoras.md` 2b-2e.
- Paso 3 ✏️ escrito, pendiente de prueba del dueño: decisión "5e puro, 5e manda sobre los stats de PD". Fuera de
  combate, el tiempo de PD (ya es simultáneo). En combate: iniciativa d20 + DES, cada turno movimiento (speed_ft / 5
  casillas, 30 ft = 6) + 1 acción; demoras de PD ignoradas; estados y timers avanzan 1 turno por ronda; sale tras 2
  rondas calmas. Detalle en `docs/game/actor.md` y `docs/game/char.md`. Pendiente: Correr, Destrabarse, Esquivar,
  ataques de oportunidad, moverse después de actuar, `speed_ft`/`initiative` reales por mob en `data/mobs.json`,
  guardar el combate.
- Paso 4 ✏️ escrito (`net/`, ver `docs/net/`), sin UI todavía: descubrimiento UDP, TCP con asyncio en un hilo y
  colas, mensajes JSON por línea (`hello`, `refused`, `action`, `state`), host autoritativo (`HostGame`) que suma o
  saca héroes y aplica acciones validadas; cliente (`ClientGame`) que recibe el estado completo. Se prueba con la UI
  del paso 6. Pendiente: aplicar el estado sin rearmar la Dungeon, que viajen dados y efectos, mandar diferencias.
- Siguiente: paso 5 (personaje portátil) y 6 (UI), juntos: sin la 6 no se puede probar la red.

## Pasos (en este orden; 1 a 3 se prueban sin red)
1. **Varios héroes en `game/`**: `Dungeon.heroes` (+ el héroe local para la UI). Cada héroe con su piso; cada piso con
   su planificador y los héroes que tiene. Vista por héroe (la UI dibuja la del local; los mobs eligen al héroe más
   cercano que ven). Regeneración y hambre por héroe. XP repartida entre los héroes del piso + bonus al último golpe.
   Modo de prueba: 2 héroes en la misma PC (por turnos con el teclado).
2. **Salvaciones contra la muerte (5e)**: a 0 PV cae inconsciente; al inicio de su turno tira d20 (10+ éxito; 3
   éxitos = estable; 3 fallos = muere; 20 = vuelve con 1 PV; 1 = dos fallos); daño estando caído = un fallo (crítico =
   dos); cualquier curación lo levanta. Muerto: espectador, o vuelve a entrar con un personaje nuevo.
3. **Modelo de turnos**: fuera de combate, simultáneo (el mundo avanza un turno cuando todos los héroes del piso
   eligieron acción). Combate (un mob te caza y te ve): se tira iniciativa (d20 + DES; mobs con dato propio) y se actúa
   por orden, una acción por turno; termina cuando ningún mob caza a nadie un par de turnos. Hay que decidir cómo
   encajan la velocidad y las demoras de PD (cangrejo rápido, lanza lenta) con "una acción por turno".
4. **Red (`net/`, stdlib)**: descubrimiento LAN por broadcast UDP (~1 s: nombre, jugadores, versión, hash de los
   registros); conexión TCP con `asyncio` en un hilo aparte que se comunica con el juego por colas (el hilo de red no
   toca pygame); mensajes JSON por línea: `hello` (versión, personaje), `welcome` (id, estado inicial), `action`
   (cliente -> host), `events` / `state` (host -> clientes). El host corre la partida; los clientes sólo dibujan y
   mandan input.
5. **Personaje portátil**: al unirse, el cliente manda su personaje (clase, características, nivel, XP, equipo); al
   salir, el host le devuelve el estado y el cliente lo guarda. El host valida rangos básicos.
6. **UI**: Jugar -> "Crear partida (LAN)" / "Unirse" con la lista de partidas encontradas; sala de espera; quién falta
   elegir acción; nombre sobre cada héroe.
7. **Hash de registros**: no se entra si el contenido (`data/`) no coincide. Prepara los mods.
8. **Chat de texto** (pedido del dueño, 2026-11-04): mensajes entre jugadores de la partida (cliente -> host -> todos,
   con nombre), en el log o en una ventana propia; en el teléfono con el teclado en pantalla.

## Riesgos
- El cambio de modelo de turnos toca el corazón del juego (`game/actor.py`, `hero.py`, `mobs.py`): hacerlo con el modo
  de prueba de 2 héroes antes de sumar la red.
- Android (después de la 1.0): el hilo de red y `asyncio` funcionan, pero el descubrimiento por broadcast puede
  necesitar permisos de multicast.
