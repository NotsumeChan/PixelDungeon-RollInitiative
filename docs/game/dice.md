# game/dice.py

Dados y fórmulas básicas de D&D 5e. Toda tirada recibe el rng explícito (determinismo para el P2P).

| Función | Qué hace |
|---|---|
| `parse("2d6+3")` | `(2, 6, 3)` |
| `roll(rng, "1d8+1", crit=False)` | `(total, texto, dados, caras)`. Con `crit` se tiran el doble de dados (el bono no se duplica) |
| `d20(rng, advantage)` | `1` = ventaja (mejor de 2), `-1` = desventaja (peor de 2). El texto muestra ambos dados |
| `modifier(valor)` | `(valor - 10) // 2`: 8 → -1, 10 → 0, 15 → +2 |
| `proficiency(nivel)` | +2 en niveles 1–4, +3 en 5–8... |
| `roll_ability_score(rng)` | 4d6 quitando el menor: `(valor, [los 4 dados])` |
| `roll_ability_pool(rng)` | creación de personaje: 7 tiradas (6 características + 1); se asignan 6 y sobra 1 |
| `valid_scores(dict)` | las 6 características presentes y entre 1 y 20 |

- `ABILITIES`: claves internas `STR DEX CON INT WIS CHA`. `ABILITY_NAMES`: cómo se muestran (FUE DES CON INT SAB CAR).
- `DEFAULT_SCORES`: array estándar de 5e (15, 14, 13, 12, 10, 8); se usa si no hay personaje creado.
