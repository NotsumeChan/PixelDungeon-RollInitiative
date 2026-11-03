import pygame as pg

# ponytail: placeholder; el heroe real vive en game/ (fase 3), esto solo dibuja su sprite
class Player():
    __slots__ = ("icon", "x", "y")
    def __init__(self, x: int, y: int, img : str) -> None:
        self.icon : pg.Surface = pg.image.load(f"Assets/Characters/{img}").convert_alpha()
        self.x : int = x
        self.y : int = y

    def Draw(self, canvas) -> None:
        canvas.canvas.blit(self.icon, self.icon.get_rect(center=(self.x, self.y)))
