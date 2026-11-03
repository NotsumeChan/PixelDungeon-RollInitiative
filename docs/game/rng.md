# game/src/rng.rs

Port de `game/rng.py`, que era `random.Random` de CPython con los nombres de `com.watabou.utils.Random`. Para que la
misma seed dé el mismo piso que el prototipo, es **MT19937 con el seeding de CPython**:
`Rng::new(seed)` = `init_by_array` con los dígitos de 32 bits de |seed| (little endian; 0 -> `[0]`), y `random()` =
`genrand_res53` (dos u32, 53 bits).

| Prototipo | Rust |
|---|---|
| `random()` / `Float()` | `random()` / `float()` |
| `Float(max)` / `Float(min, max)` | `float_max` / `float_range` |
| `Int(max)` | `int` (con max <= 0 da 0 **sin gastar** rng) |
| `Int(min, max)` | `int_range` (siempre gasta, aunque el rango sea vacío) |
| `IntRange` / `NormalIntRange` | `int_range_incl` / `normal_int_range` |
| `chances(pesos)` | `chances` |
| `element(seq)` | `pick(len)` -> índice (None sin gastar si está vacía) |

- `as i32` trunca hacia 0, igual que `int()` de Python.
- Falta `shuffle` (`_randbelow` con `getrandbits`): lo usa la partida para las apariencias de pociones (R5).
