# game/level.py

`SewerLevel(seed, depth)`: genera un piso de Alcantarillas (1-4). Junta en una clase `Level.create`,
`RegularLevel` y `SewerLevel` de Java, porque sólo existe este tipo de nivel.

## Uso
```python
from game.level import SewerLevel, to_ascii
lvl = SewerLevel(seed=42, depth=1)
lvl.map           # list[int] de LENGTH, valores de terrain.py
lvl.entrance      # celda de la escalera de subida; lvl.exit = bajada
lvl.passable      # y los demás flags: los_blocking, solid, avoid, water, pit, secret, flamable, discoverable
lvl.rooms         # list[Room]
lvl.visible / lvl.visited / lvl.mapped   # visión: ahora / alguna vez / por magia
lvl.observe(*posiciones)  # visible = unión de las vistas desde cada posición (game/fov.py), suma a visited;
                          # devuelve la vista de cada una (sin posiciones: nada visible)
lvl.set(pos, t)   # cambia terreno y sus flags (Level.set)
lvl.enter(pos) / lvl.leave(pos)  # puertas: se abren al pisarlas y se cierran al salir (Door.enter/leave)
lvl.discover(pos)                # secreto encontrado -> versión visible
lvl.random_respawn_cell(rng, occupied)  # celda libre en sala STANDARD fuera de la vista, o -1
lvl.random_destination(rng)      # celda transitable al azar (mobs que deambulan)
lvl.heaps                        # {celda: Heap}, ítems en el suelo (game/items.py)
lvl.drop(item, celda, tipo)      # deja un ítem (se apila en el montón existente)
lvl.random_drop_cell(rng)        # celda transitable de una sala STANDARD (donde caen los ítems)
```
Para revisar a mano desde la raíz del proyecto: `python -m game.level <seed> <depth>`. Imprime el mapa en ASCII con leyenda.

## Seed
`level_seed(seed, depth) = seed * 1000 + depth`: estable entre procesos y peers. No usar `hash()`.

## Orden de generación
1. Feeling (sólo depth > 1).
2. `build()` hasta que salga bien:
   1. `init_rooms`: `split` del mapa entero y cálculo de vecinas.
   2. Entrada y salida lejos entre sí (distancia >= raíz del nº de salas).
   3. Conexiones: 2 caminos entrada->salida y luego salas extra hasta un 50-70 %.
   4. `assign_room_type`, `paint`, `paint_water`, `paint_grass`, `place_traps`.
3. `decorate` (SewerLevel) -> `build_flag_maps` -> `clean_walls`.

## Diferencias con Java (a propósito)
- `split` usa el rng del nivel, no `Math.random()`. Además `49 / square` se porta como `//` porque en Java es división entera.
- Las salas son una lista (en Java `HashSet`), para que el orden sea determinista.
- Todavía no se agregan los ítems obligatorios de `Level.create` (comida, pociones...), que consumen rng antes
  del feeling. Al agregarlos cambian los mapas de cada seed; es esperado.
- `createMobs` / `createItems`: fase 3.
- Paso A: sin salas especiales (`self.specials = []`).
- Si `build_path` no encuentra camino, `build()` reintenta en vez de fallar.

## Ojo
- `observe` todavía no aplica buffs (ceguera, MindVision, Awareness).
- `set` no recalcula el cosido del agua (en Java tampoco).
- `paint_doors` se llama desde ambas salas de cada puerta, así que en pisos > 1 la tirada de puerta secreta
  se hace dos veces y vale la última. Es igual en Java y se mantiene.
- `build_flag_maps` cambia `map`: el agua pasa a `48..63` y los bordes de abismo a `CHASM_*`.
