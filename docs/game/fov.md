# game/fov.py

Port de `mechanics/ShadowCaster.java`. `cast_shadow(x, y, los_blocking, distance=8) -> list[bool]` devuelve
las celdas visibles desde `(x, y)`.

- Recorre los 8 octantes fila por fila, hasta `distance` (máximo `MAX_DISTANCE = 8`, el `viewDistance` normal).
- Cada celda que bloquea la vista (`los_blocking`: pared, puerta cerrada, pasto alto...) agrega un intervalo
  de ángulo `[a1, a2]` como obstáculo. Una celda queda oculta sólo si su centro y sus dos bordes caen dentro de
  obstáculos de filas anteriores.
- `_ROUNDING` recorta cada fila para que el área sea un círculo y no un cuadrado. Usa `floor(x + 0.5)` porque
  así redondea `Math.round` en Java (`round()` de Python redondea al par).
- La celda de origen siempre es visible y nunca bloquea: parado en una puerta se ve hacia ambos lados.
- Es puro y determinista: sin rng, sin estado global (en Java los buffers son estáticos).
