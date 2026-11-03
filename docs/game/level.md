# game/src/level.rs (+ examples/level.rs)

`game/level.py` (Level.create + RegularLevel + SewerLevel): genera un piso de las Alcantarillas (1-4) con
`Level::new(seed, depth)`; el rng del piso es `Rng::new(seed * 1000 + depth)`.

Orden (no cambiar: cada paso gasta rng): feeling (pisos > 1) -> `build` (partición `split`, entrada y salida a
distancia >= sqrt(salas), dos caminos, conexiones extra, tipos de sala, pintado, puertas, agua, pasto, trampas;
se reintenta si falla) -> `decorate` -> `build_flag_maps` (agua cosida y bordes del abismo, en el lugar) ->
`clean_walls`.

- Trampas de portado resueltas: el `random()` de `split` se tira siempre que se llega a esa rama; las puertas
  REGULAR se pintan dos veces (una por sala) y gastan rng las dos; `secret_doors` no se reinicia entre reintentos.
- Quedan para después: visión, cambios de terreno en juego, montones de ítems y celdas al azar para mobs/ítems
  (llegan con R2-R5).

**Comparar con el prototipo** (lo hace el dueño, a mano):

```sh
git worktree add ../pdri-py python-prototipe          # una vez
diff <(cd ../pdri-py && python -m game.level 1234 3) <(cargo run -q -p game --example level -- 1234 3)
```
Sin salida = mismo piso.
