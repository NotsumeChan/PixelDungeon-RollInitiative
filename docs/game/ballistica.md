# game/src/ballistica.rs

`game/ballistica.py` (mechanics/Ballistica.java): Bresenham de `from` a `to`; se frena en la primera criatura
(`stops`) o en lo que bloquea la vista, y contra una pared queda en la celda anterior. Devuelve `to` si el camino está
libre. La usan los conjuros (`Dungeon::shot`) y las armas a distancia (R5d).
