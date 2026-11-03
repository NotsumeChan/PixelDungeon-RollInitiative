import unicodedata
import pygame as pg

LATIN_FULL : str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f"

class BitmapFont():
    """BitmapText.Font.colorMarked del original: los glifos estan en una tira separados por columnas
    totalmente transparentes. Los espacios iniciales de la tira definen el ancho del espacio."""
    def __init__(self, path : str, chars : str = LATIN_FULL, tracking : int = 1) -> None:
        img = pg.image.load(path).convert_alpha()
        w, h = img.get_size()
        empty = [all(img.get_at((x, y)).a == 0 for y in range(h)) for x in range(w)]

        self.h : int = h
        self.tracking : int = tracking
        self.glyphs : dict[str, pg.Surface] = {}
        self.cache : dict[tuple, pg.Surface] = {}

        pos = 0
        while pos < w and empty[pos]:
            pos += 1
        self.glyphs[" "] = pg.Surface((pos, h), pg.SRCALPHA)
        for ch in chars:
            if ch == " " or pos >= w:
                continue
            sep = pos + 1
            while sep < w and not empty[sep]:
                sep += 1
            self.glyphs[ch] = img.subsurface((pos, 0, sep - pos, h))
            pos = sep + 1

    def Render(self, text : str, color : tuple[int, int, int] | None = None) -> pg.Surface:
        """Texto en una Surface con alpha. color tiñe el blanco del font (hardlight del original).
        font1x sólo tiene ASCII: los acentos se quitan (á -> a, ñ -> n)."""
        text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode()
        key = (text, color)
        if key not in self.cache:
            glyphs = [self.glyphs[c] for c in text if c in self.glyphs]
            width = sum(g.get_width() + self.tracking for g in glyphs) - (self.tracking if glyphs else 0)
            surf = pg.Surface((max(width, 1), self.h), pg.SRCALPHA)
            x = 0
            for g in glyphs:
                surf.blit(g, (x, 0))
                x += g.get_width() + self.tracking
            if color:
                surf.fill((*color, 255), special_flags=pg.BLEND_RGBA_MULT)
            if len(self.cache) > 512: #HUD y log generan textos nuevos todo el tiempo
                self.cache.clear()
            self.cache[key] = surf
        return self.cache[key]

    def Wrap(self, text : str, width : int) -> list[str]:
        """Parte el texto en lineas que entren en width pixeles (por palabras)."""
        lines, line = [], ""
        for word in text.split():
            test = f"{line} {word}" if line else word
            if line and self.Render(test).get_width() > width:
                lines.append(line)
                line = word
            else:
                line = test
        return lines + [line] if line else lines

# Necesita la ventana creada (convert_alpha): core/__init__ importa este modulo despues de Screen.
Font1x : BitmapFont = BitmapFont("Assets/Fonts/font1x.png")
