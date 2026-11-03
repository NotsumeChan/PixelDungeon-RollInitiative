# game/terrain.py

Port de `levels/Terrain.java` y del enum `Level.Feeling`.

- **Constantes de terreno**: los mismos valores que en Java, a propósito. En el render (fase 2) cada valor es el
  índice del tile en `tiles0.png`. No renumerar.
- **Agua**: `WATER = 63`. Al construir el nivel, `build_flag_maps` cambia cada celda de agua a `48..63` según qué
  vecinos tiene (borde del charco). Para preguntar si algo es agua, usar el flag `LIQUID`, no `== WATER`.
- **`flags[valor]`**: bits `PASSABLE`, `LOS_BLOCKING`, `FLAMABLE`, `SECRET`, `SOLID`, `AVOID`, `LIQUID`, `PIT`,
  `UNSTITCHABLE`. `UNSTITCHABLE` = el tile no se "cose" visualmente con el agua/abismo vecino.
- **`discover(t)`**: puerta o trampa secreta -> su versión visible.
- **`Feeling`**: tema del piso (NONE, CHASM, WATER, GRASS). Vive aquí y no en `level.py` para que `painters.py`
  lo use sin import circular.

## Nombres y descripciones (examinar)
- `TILE_NAMES` / `TILE_DESCS`: `Level.tileName` / `tileDesc` + lo propio de `SewerLevel` (agua turbia, musgo,
  estantería), en español e inglés (`game/lang.py`). Las trampas ocultas se llaman "Suelo" y la puerta secreta "Pared".
- `tile_info(terreno, agua, abismo)` -> `(tile, nombre, descripción)`: el agua y el abismo se nombran por su flag,
  como `WndInfoCell`.
