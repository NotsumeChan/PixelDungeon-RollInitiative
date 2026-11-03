# core/Font.py

Fuente bitmap del original: port de `BitmapText.Font.colorMarked` (PD-classes).

- `Assets/Fonts/font1x.png` (1024x8) está copiado de `java/pixel-dungeon/assets/`.
- Los glifos van en una tira, separados por columnas totalmente transparentes, en el orden de `LATIN_FULL`.
  Las columnas transparentes del principio definen el ancho del espacio.
- `tracking = 1`: 1 píxel entre letras. El original usa -1, pero aquí las letras quedaban pegadas y costaba leer.
- `Font1x.Render(texto, color=None) -> Surface` con alpha. `color` tiñe el blanco (equivale a `hardlight`).
  Los caracteres que no existen en el font se omiten. Los acentos se quitan antes (á -> a, ñ -> n), porque
  font1x sólo tiene ASCII.
- `Wrap(texto, ancho) -> list[str]`: parte por palabras para que cada línea entre en `ancho` píxeles.
- Guarda en cache cada `(texto, color)`; al pasar de 512 entradas la vacía (el HUD y el log generan textos nuevos todo el tiempo).
- `Font1x` se crea al importar y necesita la ventana creada (`convert_alpha`). Por eso `core/__init__` lo importa
  después de `Screen`.
- Hay más tamaños en el Java (`font15x`, `font2x`, `font25x`, `font3x`, con alto de glifo explícito). Por ahora no hacen falta.
