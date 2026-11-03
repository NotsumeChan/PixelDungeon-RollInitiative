# game/src/geom.rs

`game/geom.py`: mapa de 32x32 plano (celda = x + y * WIDTH), vecinos en el orden del original (`NEIGHBOURS8` empieza
por +1, -1, +W, -W) y `Rect` (com.watabou.utils.Rect: right/bottom son la pared, width = right - left).
