# Fase 1 — Generación de niveles (headless)

Objetivo: `game/level.py` genera un nivel de Alcantarillas (pisos 1–4) con el mismo algoritmo
que el original, a partir de `(seed, depth)`. Sin pygame.

## Estado
- **Paso A — escrito, pendiente de revisión manual.** STANDARD (con sus 6 variantes), ENTRANCE, EXIT,
  TUNNEL, puertas (incluidas las secretas), joinRooms, agua, pasto, trampas secretas, decoración de cloaca,
  flag maps y cleanWalls.
- **Paso B — pendiente:** salas especiales.

## Mapeo Java -> Python
| Java (`pixeldungeon/levels/` salvo indicado) | Python | Doc |
|---|---|---|
| `utils/Random.java` | `game/rng.py` | `docs/game/rng.md` |
| `utils/Rect.java` (+ constantes de `Level`) | `game/geom.py` | `docs/game/geom.md` |
| `Terrain.java` + `Level.Feeling` | `game/terrain.py` | `docs/game/terrain.md` |
| `Patch.java` | `game/patch.py` | `docs/game/patch.md` |
| `Room.java` + `utils/Graph.java` | `game/room.py` | `docs/game/room.md` |
| `painters/` | `game/painters.py` | `docs/game/painters.md` |
| `Level.create` + `RegularLevel` + `SewerLevel` | `game/level.py` | `docs/game/level.md` |

`Point` no se portó (se usan tuplas) y `PassagePainter` tampoco (esta versión no asigna PASSAGE).

## Pipeline (orden exacto)
`Level.create()`:
1. Feeling (sólo depth > 1): `Int(10)` -> 0 CHASM (si el siguiente no es jefe), 1 WATER, 2 GRASS, resto NONE.
2. Repetir hasta que `build()` dé True: llenar `map` con WALL (o CHASM).
3. `decorate()` -> `buildFlagMaps()` -> `cleanWalls()` -> `createMobs()` -> `createItems()`.

`RegularLevel.build()`:
1. `initRooms`: `split(Rect(0,0,31,31))` recursivo (min 7, max 9); falla si hay < 8 salas.
2. Entrada y salida al azar (≥ 4x4), distancia en el grafo ≥ `sqrt(nSalas)`; máx. 10 reintentos.
3. Conectar camino entrada->salida; `setPrice` sobre ese camino y conectar un 2º camino.
4. Conectar más salas hasta `nSalas * Float(0.5, 0.7)`.
5. `assignRoomType` -> `paint` -> `paintWater` -> `paintGrass` -> `placeTraps` (0 trampas en piso 1).

## Trampas de portado (resueltas)
- `split()` usaba `Math.random()` sin seed -> rng del nivel.
- `49 / rect.square()` es división entera en Java -> `//`.
- `HashSet`/`HashMap` -> `list`/`dict` (orden determinista).
- Buffers estáticos de `Patch` -> listas nuevas por llamada.
- Fall-through del `switch` de `StandardPainter` reproducido con `elif`.

## Pendiente
- **Paso B (especiales):** `SPECIALS` y `pitRoomNeeded`/`weakFloorCreated` son estado global entre pisos en Java
  -> irán en un objeto de partida (`Dungeon`) que se pase al nivel. LABORATORY forzado en piso 2.
- Ítems obligatorios de `Level.create` (comida, pociones...), `createMobs`, `createItems` y drops de tumbas -> fase 3.
- Ghost quest: fuera de alcance por ahora.

## Revisión manual (la hace el dueño)
- `python -m game.level <seed> <depth>` desde la raíz: imprime el mapa con leyenda.
- Mirar: misma seed -> mismo mapa; se puede ir de `<` a `>`; las puertas caen en paredes; en el piso 1 no hay trampas ni puertas secretas.
