import pygame as pg
from   time   import time

class Button():
    """Boton estilo DashboardItem del original: icono arriba, texto debajo, se ilumina al presionar.
    rect en pixeles virtuales (los de Screen.canvas). action(Delta) se llama al soltar el click encima."""
    def __init__(self, rect : pg.Rect, img : pg.Surface, action : callable | None = None,
                 label : pg.Surface | None = None) -> None:
        self.rect    : pg.Rect = rect
        self.img     : pg.Surface = img
        self.bright  : pg.Surface = img.copy() #version iluminada (image.brightness(1.5) en Java)
        self.bright.fill((80, 80, 80, 0), special_flags=pg.BLEND_RGB_ADD)
        self.label   : pg.Surface | None = label
        self.action  : callable | None = action
        self.pressed : bool = False

    def SetAction(self, action: callable) -> None:
        self.action = action

    def Draw(self, screen) -> None:
        img = self.bright if self.pressed else self.img
        screen.canvas.blit(img, (self.rect.x + (self.rect.w - img.get_width()) // 2, self.rect.y))
        if self.label:
            screen.canvas.blit(self.label, (self.rect.x + (self.rect.w - self.label.get_width()) // 2,
                                            self.rect.y + img.get_height() + 2))

    def EventHandler(self, event, mouse : tuple[int, int]) -> None:
        if event.type == pg.MOUSEBUTTONDOWN and event.button == 1 and self.rect.collidepoint(mouse):
            self.pressed = True
        elif event.type == pg.MOUSEBUTTONUP and event.button == 1 and self.pressed:
            self.pressed = False
            if self.action and self.rect.collidepoint(mouse):
                self.action(time())
