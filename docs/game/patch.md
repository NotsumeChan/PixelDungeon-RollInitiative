# game/patch.py

Port de `levels/Patch.java`. `generate(rng, seed, n_gen) -> list[bool]` de `LENGTH` celdas.

1. Cada celda arranca viva con probabilidad `seed` (es una densidad, no una seed de rng).
2. `n_gen` pasadas de autómata celular sobre el interior: una celda viva sigue viva con >= 4 vecinas vivas,
   una muerta nace con >= 5.

Uso en `SewerLevel`: agua `(0.45, 5)` o `(0.60, 5)` con feeling WATER; pasto `(0.40, 4)` o `(0.60, 4)` con GRASS.

- En Java los buffers son estáticos y se reusan entre llamadas; aquí son listas nuevas en cada llamada.
  Sólo cambia el borde del mapa, que siempre es pared, así que no se nota.
- Con `n_gen` par, el borde del resultado queda con el ruido inicial (por el intercambio de buffers), igual que en Java.
