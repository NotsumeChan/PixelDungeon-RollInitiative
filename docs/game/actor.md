# game/src/actor.rs

`game/actor.py`: el planificador. Los actores son `ActorId` (`Hero`, `Mob`, `Regen`, `Respawner`); qué hace cada uno
vive en `Dungeon` (dungeon.rs).

- `Scheduler::actors`: (actor, tiempo) en orden de llegada. Fuera de combate actúa el de menor tiempo; en empate, el
  agregado primero (`current`).
- Combate: `order` (personajes por iniciativa), `turn`, `round`, `calm`. La lógica de las rondas está en
  `Dungeon::combat_step` / `end_round`: cada personaje hace su turno (movimiento `speed_ft / 5` casillas + 1 acción);
  al final de la ronda la regeneración y el respawn avanzan 1 TICK; `CALM_ROUNDS` (2) rondas sin mobs cazando a la
  vista terminan el combate y todos vuelven al tiempo de PD parejos.

`Hunger(i)`: el hambre del héroe i; como `Regen`, viaja con él entre pisos y avanza 1 TICK por ronda en combate.
