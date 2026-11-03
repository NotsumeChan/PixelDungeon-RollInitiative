"""mechanics/ShadowCaster.java: campo de visión por shadowcasting en 8 sectores."""
import math

from .geom import WIDTH, HEIGHT, LENGTH

MAX_DISTANCE = 8

# _ROUNDING[d][p]: hasta qué q llega la fila p con radio d (recorta el cuadrado a un círculo).
# Math.round de Java = floor(x + 0.5); round() de Python redondea al par, no sirve.
_ROUNDING = [[]] + [[0] + [min(j, math.floor(i * math.cos(math.asin(j / (i + 0.5))) + 0.5)) for j in range(1, i + 1)]
                    for i in range(1, MAX_DISTANCE + 1)]

_SECTORS = [(+1, +1, 0, 0), (-1, +1, 0, 0), (+1, -1, 0, 0), (-1, -1, 0, 0),
            (0, 0, +1, +1), (0, 0, -1, +1), (0, 0, +1, -1), (0, 0, -1, -1)]


def cast_shadow(x: int, y: int, los_blocking: list[bool], distance: int = MAX_DISTANCE) -> list[bool]:
    """Celdas visibles desde (x, y). distance <= MAX_DISTANCE."""
    fov = [False] * LENGTH
    fov[y * WIDTH + x] = True
    limits = _ROUNDING[distance]
    for m1, m2, m3, m4 in _SECTORS:
        _scan_sector(fov, los_blocking, limits, distance, x, y, m1, m2, m3, m4)
    return fov


def _scan_sector(fov, los_blocking, limits, distance, cx, cy, m1, m2, m3, m4) -> None:
    # Obstáculos como intervalos de ángulo [a1, a2]; sólo cuentan los de filas anteriores (limit).
    obs_a1: list[float] = []
    obs_a2: list[float] = []
    limit = 0

    def blocked(a: float) -> bool:
        return any(obs_a1[i] <= a <= obs_a2[i] for i in range(limit))

    for p in range(1, distance + 1):
        dq2 = 0.5 / p
        for q in range(limits[p] + 1):
            x = cx + q * m1 + p * m3
            y = cy + p * m2 + q * m4
            if 0 <= y < HEIGHT and 0 <= x < WIDTH:
                a0 = q / p
                a1, a2 = a0 - dq2, a0 + dq2
                pos = y * WIDTH + x
                if not (blocked(a0) and blocked(a1) and blocked(a2)):
                    fov[pos] = True
                if los_blocking[pos]:
                    if len(obs_a1) > limit and a1 <= obs_a2[-1]:
                        obs_a2[-1] = a2
                    else:
                        obs_a1.append(a1)
                        obs_a2.append(a2)
        limit = len(obs_a1)
