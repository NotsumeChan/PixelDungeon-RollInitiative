# Assets propios

Todo lo de `Assets/` viene del Pixel Dungeon original salvo lo listado aquí (hecho para este proyecto, a pedido del
dueño). Los archivos propios llevan el prefijo `pdri_`.

Ninguno por ahora.

**De terceros (no de PD):** `Assets/Fonts/silkscreen/`: la fuente Silkscreen (Jason Kottke, The Silkscreen Project
Authors, SIL Open Font License 1.1: `OFL.txt` al lado), su TTF y la tira que usa el juego (`silkscreen.png`, hecha
con `tools/font_strip.py`). (`Assets/Items/pdri_items.png`, generado con pygame y sin uso, se borró el 2026-11-06: nada de
arte generado.)

Sprite que falta: el ítem va sin `image` en `data/items.json` y se dibuja su nombre (`gameUi.TextIcon`). Lo mismo si
una hoja propia o de un mod no carga o el índice no existe. No generar sprites: esperar a uno hecho por una persona.
