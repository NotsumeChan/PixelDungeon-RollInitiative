# game/geom.py

Constantes del mapa y `Rect`.

- `WIDTH = HEIGHT = 32`, `LENGTH = 1024`. El mapa es una lista plana: `celda = x + y * WIDTH`.
- `NEIGHBOURS4`: N, E, S, O. El orden importa: `getWaterTile` usa su índice como bit.
- `NEIGHBOURS8`, `NEIGHBOURS9` (`[0] + 8`): mismo orden que `Level.java`.
- Los offsets no comprueban bordes: sumar `-1` en la columna 0 salta a la fila anterior (igual que en Java).

## Rect (`utils/Rect.java`)
- `right`/`bottom` son la celda de la pared, no "uno pasado el final": `width() = right - left`, y una sala
  `Rect(0,0,8,8)` ocupa 9x9 celdas contando las paredes.
- Sólo está lo que se usa (`width`, `height`, `square`, `intersect`). `Point` no se portó: se usan tuplas `(x, y)`.
