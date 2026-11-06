# game/src/dice.rs

`game/dice.py`: dados de la 5.ª edición (5e). Todas las tiradas reciben el rng de la partida.

- `parse("2d6+3")` -> `(2, 6, 3)`; `roll(rng, notación, crit)` -> (total, texto, dados, caras); con `crit` tira el
  doble de dados. Cada dado es `IntRange(1, caras)`, igual que el prototipo.
- `d20(rng, adv)`: 1 = ventaja (mejor de 2), -1 = desventaja (peor de 2). El texto muestra los dos dados (`15[15/3 V]`).
- `modifier` redondea hacia abajo (`div_euclid`, como `//` de Python); `proficiency` = +2 en niveles 1-4, +3 en 5-8...
- `DEFAULT_SCORES`: array estándar (15, 14, 13, 12, 10, 8): la partida sin personaje (la de fondo del título).
- Creación de personaje: `roll_ability_score` = 4d6 quitando el menor (devuelve los 4 dados para mostrar el
  descartado), `roll_ability_pool` = 7 tiradas (se asignan 6 y sobra 1), `valid_scores` = las 6 puestas y entre
  `MIN_SCORE` y `MAX_SCORE` (1-20, para las escritas a mano).

`ABILITY_NAMES`: FUE, DES, CON, INT, SAB, CAR (en el orden de `ABILITIES`).
