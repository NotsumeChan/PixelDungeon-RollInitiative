# game/rng.py

Port de `com/watabou/utils/Random.java`. `Rng` hereda de `random.Random`, así cada nivel tiene su propio
generador con seed. Nunca usar el `random` global.

Los métodos se llaman igual que en Java (`Int`, `Float`, `IntRange`, `NormalIntRange`, `chances`, `element`)
para portar copiando.

| Método | Rango |
|---|---|
| `Int(max)` | `[0, max)`; 0 si max <= 0 |
| `Int(min, max)` | `[min, max)` |
| `IntRange(min, max)` | `[min, max]` |
| `Float()` / `Float(max)` / `Float(min, max)` | `[0,1)` / `[0,max)` / `[min,max)` |
| `chances(pesos)` | índice elegido según pesos |
| `element(lista)` | elemento al azar; `None` si la lista está vacía |

- No da los mismos números que Java (otro generador). Lo que importa es que sea igual entre peers de Python.
- Faltan `shuffle`, `oneOf`, `index`: agregarlos cuando se usen.
