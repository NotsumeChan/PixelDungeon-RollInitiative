# game/src/pathfinder.rs

`game/pathfinder.py` (utils/PathFinder.java): BFS en 8 direcciones desde el destino hasta encontrar el origen;
`get_step` devuelve la vecina con menor distancia, o -1. Port 1:1, incluida la rareza del prototipo: el origen se
puede anotar más de una vez antes de salir de la cola, y eso decide qué paso se elige en los empates.
