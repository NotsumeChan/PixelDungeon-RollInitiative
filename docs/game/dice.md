# game/src/dice.rs

`game/dice.py`: dados de D&D 5e. Todas las tiradas reciben el rng de la partida.

- `parse("2d6+3")` -> `(2, 6, 3)`; `roll(rng, notación, crit)` -> (total, texto, dados, caras); con `crit` tira el
  doble de dados. Cada dado es `IntRange(1, caras)`, igual que el prototipo.
- `d20(rng, adv)`: 1 = ventaja (mejor de 2), -1 = desventaja (peor de 2). El texto muestra los dos dados (`15[15/3 V]`).
- `modifier` redondea hacia abajo (`div_euclid`, como `//` de Python); `proficiency` = +2 en niveles 1-4, +3 en 5-8...
- `DEFAULT_SCORES`: array estándar (15, 14, 13, 12, 10, 8) hasta que exista la creación de personaje (R5).

`ABILITY_NAMES`: FUE, DES, CON, INT, SAB, CAR (en el orden de `ABILITIES`).
