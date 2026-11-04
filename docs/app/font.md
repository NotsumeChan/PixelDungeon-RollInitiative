# app/src/font.rs

Port de `core/Font.py` (BitmapText.Font.colorMarked del original): `font1x.png` es una tira de glifos separados por
columnas totalmente transparentes; las columnas vacías del principio definen el ancho del espacio. Tracking 1 px.

- `text(out, texto, x, y, color)`: agrega un `Sprite` por glifo (dibujado en el mismo lote que todo lo demás) y
  devuelve el ancho. El color tiñe el blanco de la fuente.
- font1x sólo tiene ASCII: `ascii()` quita acentos (á -> a, ñ -> n, ¿ -> ?). Lo que no está en la tira se omite.

`text_scaled` dibuja con cada pixel de la fuente de `scale` x `scale` (entero: nítido; los dados). `width` mide un
texto sin dibujarlo (centrar) y `height` es el alto de la fuente.
