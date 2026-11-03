# game/src/painters.rs

`game/painters.py` (painters/*.java): STANDARD (con cementerio, quemada, rayada, estudio, puente y fisura), ENTRANCE,
EXIT y TUNNEL. Las salas especiales llegan después.

- El encadenado de variantes de STANDARD copia el `elif` del prototipo (el switch de Java con fall-through).
- `prange` = `range(a, b, step)` de Python con paso ±1 (vacío si no avanza hacia b).
- Puente: `{d1.x, d2.x} == {left, right}` del prototipo es "una puerta en cada lado". Se llama a `center()` entero
  aunque se use una coordenada (gasta las dos tiradas, como el prototipo).
- `fill` recorta al mapa (Python podía agrandar la lista; no pasa con mapas válidos).
