# game/src/ballistica.rs

Línea de tiro propia (0.9.28; antes, el port de Ballistica de PD): Bresenham de `from` a `to`. Se frena en la primera
criatura (`stops`) o en lo que tapa la vista; contra una pared queda en la celda anterior. Si devuelve `to`, el camino
está libre.
