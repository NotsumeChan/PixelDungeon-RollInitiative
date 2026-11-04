# game/src/char.rs

`game/char.py`: lo común de héroe y mob (`Char`: celda, nombre, PV, velocidades, estado del turno de combate 5e).
`distance` (Chebyshev) y `adjacent`. El combate (`attack`, `damage`, `spend`, `move_char`) está en dungeon.rs porque
toca a atacante y objetivo.

- `start_turn(speed_ft)`: movimiento y acción del turno. `turn_over`: sin acción o sin movimiento.
- `stepped`: el próximo `spend` es un paso (gasta movimiento, no la acción).

`trail`: las celdas por las que pasó desde que el render miró (tope `TRAIL_MAX` = 16); el render las consume para
que el sprite siga el camino real. `move_char` verifica en debug que cada paso sea a una celda vecina y transitable.

`buffs`: los estados alterados del personaje (buffs.rs).
