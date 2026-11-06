# game/src/actor.rs

`game/actor.py`: el planificador, uno por zona (escena `{x, y, z}`: una escena lateral tiene el suyo y su propio
combate; ver el glosario en `dungeon.md`). Los actores son `ActorId` (`Hero`, `Mob`, `Regen`, `Respawner`); qué hace cada uno
vive en `Dungeon` (dungeon.rs).

- `Scheduler::actors`: (actor, tiempo) en orden de llegada. Fuera de combate actúa el de menor tiempo; en empate, el
  agregado primero (`current`).
- **Quién pelea** (0.9.39, `Dungeon::joins_combat`): los héroes de la zona y los mobs que pelean o que algún héroe
  ve; los demás mobs siguen a su tiempo (1 TICK por ronda, en `end_round`) y se suman al final de una ronda si
  empiezan a pelear o se dejan ver. Sin enemigos vivos en la ronda, el combate termina en el momento
  (`finish_combat`); si quedan pero ninguno pelea, tras `CALM_ROUNDS` rondas.
- **Simultáneo fuera de combate** (propio, 0.9.35; `Dungeon::process_floor`): en PD un héroe sin acción frenaba todo
  la zona hasta que su jugador eligiera. Ahora, si le toca a un héroe quieto (sin acción ni animación) y otro héroe
  de la zona tiene algo en curso, deja pasar su turno (1 TICK) y la zona sigue. Sólo se espera input cuando ningún
  héroe de la zona tiene nada elegido. En combate no cambia (rondas por iniciativa).
- Combate: `order` (personajes por iniciativa), `turn`, `round`, `calm`. La lógica de las rondas está en
  `Dungeon::combat_step` / `end_round`: cada personaje hace su turno (movimiento `speed_ft / 5` casillas + 1 acción);
  al final de la ronda la regeneración y el respawn avanzan 1 TICK; `CALM_ROUNDS` (2) rondas sin mobs cazando a la
  vista terminan el combate y todos vuelven al tiempo de PD parejos.

`Hunger(i)`: el hambre del héroe i; como `Regen`, viaja con él entre zonas y avanza 1 TICK por ronda en combate.

`Buff(uid)`: un estado alterado (buffs.rs). `Trap`: una trampa como origen de daño; no actúa.
