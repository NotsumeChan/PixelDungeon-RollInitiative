import pygame as pg

class AnimatedPlayer(pg.sprite.Sprite):
    def __init__(self, x : int, y : int, animationSpeed : float, sheet : pg.Surface, ammountFrames : int, ammountRows : int = 1):
        super().__init__()
        self.frames : list = self._splitFrames(sheet, ammountFrames, ammountRows)
        self.current_frame : float = 0.0
        self.animationSpeed : float = animationSpeed
        self.img = self.frames[0]
        self.rect = self.img.get_rect(center=(x, y))

    @staticmethod
    def _splitFrames(img : pg.Surface, ammountFrames : int, ammountRows : int) -> list:
        frameW : int = img.get_width()  // ammountFrames
        frameH : int = img.get_height() // ammountRows
        grid = [[img.subsurface(pg.Rect(col * frameW, row * frameH, frameW, frameH)) for col in range(ammountFrames)]
                for row in range(ammountRows)]
        return grid if len(grid) > 1 else grid[0]

    def update(self) -> None:
        self.current_frame = (self.current_frame + self.animationSpeed) % len(self.frames)
        self.img = self.frames[int(self.current_frame)]
