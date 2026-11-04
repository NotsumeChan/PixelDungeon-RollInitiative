# game/src/fov.rs

`game/fov.py` (mechanics/ShadowCaster.java): shadowcasting en 8 sectores, radio `MAX_DISTANCE` = 8. Los obstáculos son
intervalos de ángulo; sólo cuentan los de filas anteriores. El recorte a círculo usa `floor(x + 0.5)` (Math.round de
Java), no el redondeo al par.
