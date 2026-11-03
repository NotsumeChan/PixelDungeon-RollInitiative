"""mechanics/Ballistica.java: línea de tiro de proyectiles y conjuros."""
from .geom import WIDTH


def cast(level, frm: int, to: int, find_char) -> int:
    """Ballistica.cast(from, to, magic=False, hitChars=True): celda donde se detiene un proyectil que va de frm a to.
    Se frena en la primera criatura o en lo que bloquea la vista; contra una pared queda en la celda anterior.
    Si devuelve to, el camino está libre."""
    x0, y0, x1, y1 = frm % WIDTH, frm // WIDTH, to % WIDTH, to // WIDTH
    dx, dy = x1 - x0, y1 - y0
    step_x, step_y = (1 if dx > 0 else -1), (1 if dy > 0 else -1)
    dx, dy = abs(dx), abs(dy)
    if dx > dy:
        step_a, step_b, da, db = step_x, step_y * WIDTH, dx, dy
    else:
        step_a, step_b, da, db = step_y * WIDTH, step_x, dy, dx
    cell, err = frm, da // 2  # división entera, como en Java
    while cell != to:
        prev = cell
        cell += step_a
        err += db
        if err >= da:
            err -= da
            cell += step_b
        if not level.passable[cell] and not level.avoid[cell]:
            return prev
        if level.los_blocking[cell] or find_char(cell) is not None:
            return cell
    return to
