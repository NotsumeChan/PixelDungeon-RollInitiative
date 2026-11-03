from .geom import WIDTH, HEIGHT, LENGTH
from .rng import Rng


def generate(rng: Rng, seed: float, n_gen: int) -> list[bool]:
    """Patch.generate: ruido con densidad `seed` suavizado n_gen veces por autómata celular.
    Una celda vive con >= 4 vecinas vivas (si ya vivía) o >= 5 (si no)."""
    w, h = WIDTH, HEIGHT
    off = [rng.random() < seed for _ in range(LENGTH)]
    cur = [False] * LENGTH
    for _ in range(n_gen):
        for y in range(1, h - 1):
            for x in range(1, w - 1):
                pos = x + y * w
                count = (off[pos - w - 1] + off[pos - w] + off[pos - w + 1] + off[pos - 1] + off[pos + 1]
                         + off[pos + w - 1] + off[pos + w] + off[pos + w + 1])
                cur[pos] = count >= 4 if off[pos] else count >= 5
        off, cur = cur, off
    return off
