# game/src/room.rs

`game/room.py` (Room.java + Graph.java). Las salas viven en `Level::rooms` y las puertas en `Level::doors`; se
referencian por índice (una puerta la comparten dos salas).

- `connected: Vec<(sala, Option<puerta>)>` en orden de conexión: reemplaza al dict de Python, cuyo orden de
  inserción decide el orden de las tiradas. Nunca un HashMap.
- `center()` tira primero para x y después para y (si el lado es impar), aunque sólo se use una coordenada.
- `build_distance_map` (BFS ponderado por `price` sobre las vecinas) y `build_path` como Graph.java.
