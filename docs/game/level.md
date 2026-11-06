# game/src/level.rs (+ examples/level.rs)

`game/level.py` (Level.create + RegularLevel + SewerLevel): genera un piso de las Alcantarillas (1-4) con
`Level::new(seed, depth)`; el rng del piso es `Rng::new(seed * 1000 + depth)`.

Orden (no cambiar: cada paso gasta rng): feeling (pisos > 1) -> `build` (partición `split`, entrada y salida a
distancia >= sqrt(salas), dos caminos, conexiones extra, tipos de sala, pintado, puertas, agua, pasto, trampas;
se reintenta si falla) -> `decorate` -> `build_flag_maps` (agua cosida y bordes del abismo, en el lugar) ->
`clean_walls`.

- Trampas de portado resueltas: el `random()` de `split` se tira siempre que se llega a esa rama; las puertas
  REGULAR se pintan dos veces (una por sala) y gastan rng las dos; `secret_doors` no se reinicia entre reintentos.
- Celdas al azar (R3): `pick_room` (random_room con otro rng; `random_room` la usa con una copia del del nivel, mismas
  tiradas), `random_respawn_cell` (sala STANDARD, fuera de la vista y libre; -1 tras 10 intentos) y
  `random_destination` (cualquier sala, destino de los mobs que deambulan).
- Quedan para después: montones de ítems y `random_drop_cell` (R5).

**Comparar con el prototipo** (lo hace el dueño, a mano):

```sh
git worktree add ../pdri-py python-prototipe          # una vez
diff <(cd ../pdri-py && python -m game.level 1234 3) <(cargo run -q -p game --example level -- 1234 3)
```
Sin salida = mismo piso.

Montones de ítems (R5a): `heaps` (por celda, ordenados), `drop` (se apila en el montón que haya) y
`random_drop_cell` (sala STANDARD, transitable).

## Zonas (M1a, 0.9.26)
`Zone {x, y, z}`: las coordenadas de una zona del mundo; `z` es la profundidad y sigue siendo el "nivel de peligro"
de lo que depende del piso (mobs, ítems, trampas, NPC: usan `zone.z`). `Zone::depth(z)` = (0, 0, z), la pila de las
Alcantarillas; `Zone::NONE` = ninguna (héroe que todavía no entró); `key` / `parse` = "x,y,z" (guardados).
`level_seed(seed, zone)`: en (0, 0, z) es la de siempre (seed * 1000 + z), así los pisos salen iguales. Cada nivel
tiene `width` / `height`; el generador de las Alcantarillas genera `SEWER_SIZE` = 32.
