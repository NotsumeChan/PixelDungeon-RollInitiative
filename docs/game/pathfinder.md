# game/pathfinder.py

Port de `utils/PathFinder.java`: BFS en 8 direcciones (las diagonales cuestan lo mismo que lo recto).

- `get_step(frm, to, passable) -> celda | -1`: el siguiente paso de frm hacia to.
- El BFS sale desde `to` y para al llegar a `frm`, que puede no ser passable (el que camina está parado ahí).
- No mira bordes de fila: confía en que el borde del mapa nunca es passable (igual que Java).
- Sólo está `getStep`. `find` (camino completo) y `getStepBack` (huir) se portan cuando se usen: huir hace
  falta para el Thief y para el estado FLEEING.
- Quién puede pasar lo decide el que llama: ver `Dungeon.find_path` y `Hero.get_closer`.
