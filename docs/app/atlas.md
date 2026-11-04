# app/src/atlas.rs

Un solo atlas para todo el frame: las hojas PNG apiladas en vertical, en el orden en que se pasan (`rects[i]` =
dónde quedó la hoja `i`). Las hojas van dentro del binario (`include_bytes!` en `app/src/lib.rs`), así Android no
necesita AssetManager (son ~2,8 MB).

- `decode`: PNG -> RGBA8 (EXPAND + STRIP_16 + ALPHA de png 0.18; gris+alfa se pasa a RGBA).
- `sprite(x, y, w, h, pos, color)`: un recorte del atlas como `Sprite`, del tamaño del recorte.
- ponytail: apilado vertical; con muchas hojas puede pasar el alto máximo de textura de alguna GPU vieja
  (`max_texture_dimension_2d`): entonces, empaquetar en estantes.

Debajo de las hojas va un pixel blanco: `rect(pos, tamaño, color)` dibuja un rectángulo de color liso con el
mismo lote (fondos, bordes, partículas).
