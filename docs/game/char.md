# game/src/char.rs

`game/char.py`: lo común de héroe y mob (`Char`: celda, nombre, PV, velocidades, estado del turno de combate 5e).
`distance` (Chebyshev) y `adjacent`. El combate (`attack`, `damage`, `spend`, `move_char`) está en dungeon.rs porque
toca a atacante y objetivo.

- `start_turn(speed_ft)`: movimiento y acción del turno. `turn_over`: sin acción o sin movimiento.
- `stepped`: el próximo `spend` es un paso (gasta movimiento, no la acción).
