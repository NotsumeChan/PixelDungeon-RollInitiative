# game/src/fov.rs

Campo de visión propio (0.9.28; antes, el port del ShadowCaster de PD): *symmetric shadowcasting* de Albert Ford
(2021, dominio público). Cuatro cuadrantes, fila por fila; una fila se parte en dos donde hay pared. Una casilla se ve
si es pared que toca lo visible o si cae entera dentro de la fila (`symmetric`): eso la hace simétrica (si ves a un
mob, te ve). Radio `MAX_DISTANCE` = 8, recortado a un círculo (r² + r). Pendientes como fracciones enteras: sin
floats. Fuera del mapa cuenta como pared.
