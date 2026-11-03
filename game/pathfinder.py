"""utils/PathFinder.java: BFS en 8 direcciones sobre el mapa plano."""
from collections import deque

from .geom import WIDTH, LENGTH

_DIRS = [-1, +1, -WIDTH, +WIDTH, -WIDTH - 1, -WIDTH + 1, +WIDTH - 1, +WIDTH + 1]
_INF = 2**31 - 1


def _distance_map(frm: int, to: int, passable: list[bool]) -> list[int] | None:
    """Distancias desde `to` hasta encontrar `frm` (que no necesita ser passable). None si no hay camino."""
    if frm == to:
        return None
    dist = [_INF] * LENGTH
    dist[to] = 0
    queue = deque([to])
    while queue:
        step = queue.popleft()
        if step == frm:
            return dist
        nd = dist[step] + 1
        for d in _DIRS:
            n = step + d
            if n == frm or (0 <= n < LENGTH and passable[n] and dist[n] > nd):
                dist[n] = nd
                queue.append(n)
    return None


def get_step(frm: int, to: int, passable: list[bool]) -> int:
    """Siguiente celda desde frm hacia to, o -1."""
    dist = _distance_map(frm, to, passable)
    if dist is None:
        return -1
    best, min_d = frm, dist[frm]
    for d in _DIRS:
        n = frm + d
        if dist[n] < min_d:
            best, min_d = n, dist[n]
    return best
