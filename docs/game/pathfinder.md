# game/src/pathfinder.rs

Caminos propios (0.9.28; antes, el port de PathFinder de PD): BFS en 8 direcciones desde el destino hasta quien
camina; el paso es su vecina más cerca. Empates (0.9.36; antes ganaba la primera de `neighbours`, una diagonal, y el
camino subía en diagonal y zigzagueaba): la vecina más pegada a la recta hasta el destino (producto cruz) y después
la más cerca del destino. Como se recalcula en cada paso, sale una línea tipo Bresenham. Enteros: determinista.
`to` puede no ser transitable (un mob, un cofre). Las vecinas no dan la vuelta por los bordes (mapas de cualquier
ancho). -1 = sin camino.
