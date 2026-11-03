# game/painters.py

Port de `levels/painters/`. Cada painter es una función `(level, room)` que escribe en `level.map`.
`PAINTERS` mapea `RoomType -> función`.

## Helpers (Painter.java)
- `fill(level, x, y, w, h, v)`: rectángulo de w x h celdas.
- `fill_room(level, room, m, v)`: la sala con sus paredes, achicada `m` celdas. `m=0` = todo, `m=1` = interior.
- `draw_inside(level, room, door, n, v)`: desde una puerta, n celdas hacia adentro.
- `set_cell(level, x, y, v)`.

## Painters implementados
| Tipo | Java | Qué hace |
|---|---|---|
| ENTRANCE / EXIT | `EntrancePainter`, `ExitPainter` | sala vacía, escalera a 1 celda de la pared; guarda `level.entrance` / `level.exit` |
| TUNNEL | `TunnelPainter` | pasillo en L/T desde cada puerta hasta el centro |
| STANDARD | `StandardPainter` | sala vacía; con 1/5 de probabilidad, una variante (abajo) |

Variantes de STANDARD (`Int(6)`):

| Valor | Variante | Condición |
|---|---|---|
| 0 | cementerio | lado mínimo >= 4 y máximo >= 6; con feeling GRASS cae a 1 |
| 1 | quemada | depth > 1 |
| 2 | rayada | lado máximo >= 4 |
| 3 | estudio | >= 6x6 |
| 4 | puente | 2 puertas y >= 4x4; con feeling WATER cae a 5 |
| 5 | fisura | lado mínimo >= 5 y no es piso de jefe ni el anterior |

Si la variante no cumple su condición, queda una sala vacía normal. La cascada de los valores 0 y 4 reproduce
el fall-through del `switch` de Java.

## Pendiente
- El cementerio guarda en `level.tombs` pares `(celda, tiene_item)`. Los montones TOMB (oro/ítem) llegan con los ítems.
- Salas especiales (paso B). PASSAGE no se porta: esta versión de Java no lo asigna nunca.
