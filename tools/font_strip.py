"""Tira de glifos para app/src/font.rs a partir de una fuente TTF pixel (hoy Silkscreen, OFL).

    python3 tools/font_strip.py Assets/Fonts/silkscreen/Silkscreen-Regular.ttf Assets/Fonts/silkscreen/silkscreen.png

Formato (el de font1x.png del original): una fila de glifos blancos en el orden de `CHARS`, separados por una
columna transparente; las columnas transparentes del principio son el ancho del espacio. Un glifo con columnas vacías
por dentro (", %) lleva en ellas un píxel con alfa 1 (invisible) para que el separador no lo corte.
"""
import sys

from PIL import Image, ImageDraw, ImageFont

CHARS = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
SIZE = 8  # Silkscreen está dibujada para 8 px: nítida sin suavizado
HEIGHT = 8  # alto de la tira (el de font1x: la UI está medida para eso)


def glyph(font, ch):
    im = Image.new("L", (32, 32))
    d = ImageDraw.Draw(im)
    d.fontmode = "1"  # sin antialias
    d.text((4, 4), ch, font=font, fill=255)
    return im


def main(ttf, out):
    font = ImageFont.truetype(ttf, SIZE)
    ims = {c: glyph(font, c) for c in CHARS[1:]}
    rows = [y for im in ims.values() for y in range(32) if any(im.getpixel((x, y)) for x in range(32))]
    top, bottom = min(rows), max(rows)
    top = max(top, bottom - HEIGHT + 1)  # si no entra, se recorta arriba (sólo el $): las colas de , ; quedan
    pad = max(0, (HEIGHT - (bottom - top + 1)) // 2)
    space = max(2, round(font.getlength(" ")) - 1)
    cols = [[0] * HEIGHT for _ in range(space)]
    for c in CHARS[1:]:
        im = ims[c]
        xs = [x for x in range(32) if any(im.getpixel((x, y)) for y in range(32))]
        for x in range(min(xs), max(xs) + 1):
            col = [0] * HEIGHT
            for y in range(top, bottom + 1):
                if im.getpixel((x, y)) and 0 <= y - top + pad < HEIGHT:
                    col[y - top + pad] = 255
            if not any(col):
                col[0] = 1  # hueco dentro del glifo: no es separador
            cols.append(col)
        cols.append([0] * HEIGHT)  # separador
    strip = Image.new("RGBA", (len(cols), HEIGHT))
    for x, col in enumerate(cols):
        for y, a in enumerate(col):
            strip.putpixel((x, y), (255, 255, 255, a))
    strip.save(out)
    print(f"{out}: {len(cols)}x{HEIGHT}, filas {top}-{bottom} + {pad} arriba, espacio {space}")


if __name__ == "__main__":
    main(*sys.argv[1:3])
