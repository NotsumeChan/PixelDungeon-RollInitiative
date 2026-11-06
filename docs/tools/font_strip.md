# tools/font_strip.py

Pasa una fuente TTF pixel (hoy Silkscreen, `Assets/Fonts/silkscreen/`) a la tira de glifos que lee `app/src/font.rs`:
blancos sobre transparente, en el orden de `CHARS`, separados por una columna vacía; el espacio = las columnas vacías
del principio. Render a 8 px sin suavizado (Silkscreen está dibujada para 8 px), tira de 8 de alto (la UI está medida
para eso; si no entra, se recorta arriba). Un hueco dentro de un glifo (`"`, `%`) lleva un píxel con alfa 1 (no se ve)
para que no cuente como separador. Necesita Pillow (`python3 -m pip install pillow`). Regenerar:
`python3 tools/font_strip.py Assets/Fonts/silkscreen/Silkscreen-Regular.ttf Assets/Fonts/silkscreen/silkscreen.png`
