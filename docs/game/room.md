# game/room.py

Port de `levels/Room.java` y `utils/Graph.java`.

## Room
- Hereda de `Rect`: el borde son las paredes y el interior va de `left+1` a `right-1`.
- `type`: `RoomType`. Empieza en `NULL`; `assign_room_type` lo cambia.
- `neigbours`: salas que comparten una pared de >= 3 celdas (posibles conexiones). Se mantiene la errata de Java.
- `connected`: `{sala: Door | None}`. `connect()` mete `None`; `place_doors` crea la `Door` al pintar.
  Es un dict, que conserva el orden de inserción: así el resultado es determinista (en Java es `HashMap`).
- `random(rng, m)`: celda al azar del interior, a `m` celdas de la pared.
- `center(rng)`: si el lado es par, desempata al azar (consume rng).
- El hash es por identidad, como en Java.

## Door / DoorType
`Door.set(t)` sólo sube de tipo: EMPTY < TUNNEL < REGULAR < UNLOCKED < HIDDEN < BARRICADE < LOCKED.
Por eso la puerta entre un túnel y una sala estándar termina REGULAR.

## SPECIALS
Orden original. En Java es una lista global que se reordena entre pisos. La copia viva irá en el estado de partida
cuando se implementen las especiales (paso B de la fase 1).

## Grafo (Graph.java)
- `build_distance_map(rooms, focus)`: distancias desde `focus` sobre `neigbours` (no sobre `connected`),
  ponderadas por el `price` de la sala que se expande.
- `build_path(start, end)`: baja por las distancias. Devuelve `None` si no hay camino (Java tiraría
  NullPointerException; aquí `build()` reintenta).
