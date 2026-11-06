# game/src/pathfinder.rs

Caminos propios (0.9.28; antes, el port de PathFinder de PD): BFS en 8 direcciones desde el destino hasta quien
camina; el paso es su vecina más cerca (empates: la primera en el orden de `neighbours`, igual en todas las máquinas).
`to` puede no ser transitable (un mob, un cofre). Las vecinas no dan la vuelta por los bordes (mapas de cualquier
ancho). -1 = sin camino.
