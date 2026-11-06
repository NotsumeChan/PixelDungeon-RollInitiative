# game/src/geom.rs

Mapa plano (celda = x + y * ancho). No hay tamaño global (M1a): cada `Level` tiene `width` / `height` y sus
vecinos (`n4`, `n8`, `n9`, en el orden del original: `n8` empieza por +1, -1, +W, -W), `distance` (Chebyshev),
`adjacent` e `inside`. `MAX_SIZE` (128) es el tope de una zona (la niebla de la GPU se reserva para eso). El
generador de las Alcantarillas genera 32x32 (`level::SEWER_SIZE`, un parámetro suyo). `Rect`: com.watabou.utils.Rect
(right/bottom son la pared, width = right - left).
