# game/src/terrain.rs

`game/terrain.py` (Terrain.java): constantes de celda (`u8`, el valor es el índice en `tiles0.png`), `FLAGS` como tabla
`const` de 256, `discover` (secreto -> visible) y `Feeling`. Los nombres y descripciones de celdas para examinar
quedan para la UI (R4).

`info(terreno, agua, abismo)` (WndInfoCell): el tile que se muestra al examinar, su nombre y su descripción como
`[es, en]`. El agua y el abismo se nombran por su flag; una trampa oculta se ve como suelo.
