# Tamaño del mapa (Level.java). El mapa es una lista plana: celda = x + y * WIDTH.
WIDTH = 32
HEIGHT = 32
LENGTH = WIDTH * HEIGHT

NEIGHBOURS4 = [-WIDTH, +1, +WIDTH, -1]
NEIGHBOURS8 = [+1, -1, +WIDTH, -WIDTH, +1 + WIDTH, +1 - WIDTH, -1 + WIDTH, -1 - WIDTH]
NEIGHBOURS9 = [0] + NEIGHBOURS8


class Rect:
    """com.watabou.utils.Rect. Ojo: right/bottom son coordenadas de celda (la pared), width = right - left."""
    __slots__ = ("left", "top", "right", "bottom")

    def __init__(self, left: int = 0, top: int = 0, right: int = 0, bottom: int = 0) -> None:
        self.left, self.top, self.right, self.bottom = left, top, right, bottom

    def width(self) -> int:
        return self.right - self.left

    def height(self) -> int:
        return self.bottom - self.top

    def square(self) -> int:
        return self.width() * self.height()

    def intersect(self, o: "Rect") -> "Rect":
        return Rect(max(self.left, o.left), max(self.top, o.top), min(self.right, o.right), min(self.bottom, o.bottom))

    def __repr__(self) -> str:
        return f"Rect({self.left},{self.top},{self.right},{self.bottom})"
