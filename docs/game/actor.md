# game/actor.py

Port de `actors/Actor.java`: el sistema de turnos por tiempo de Pixel Dungeon.

- Cada `Actor` tiene `time`. `Scheduler.process()` elige siempre al de menor `time` y llama `act()`.
- `act()` **debe** gastar tiempo con `spend(t)`; si no, el planificador vuelve a elegir al mismo actor.
  Excepción buscada: un mob que pasa de WANDERING a HUNTING no gasta y actúa de nuevo enseguida, como en Java.
- `TICK = 1`: moverse o atacar cuesta 1 a velocidad normal. Velocidad 2 (Crab) = 0.5 por paso.
- `waiting()`: si el actor que toca devuelve True, `process()` se detiene. El héroe lo usa para esperar input
  o el fin de su animación. Mobs y buffs nunca esperan.
- En empate de `time` gana el que se agregó primero. Por eso el héroe se agrega primero en cada piso.
- Diferencia con Java: allí `Actor` es estático y global; aquí hay un `Scheduler` por piso dentro de `Dungeon`.
  Así podrá haber varios jugadores o pisos (P2P).
- `process(stop)`: si se pasa `stop`, se revisa antes de cada turno y corta el loop cuando da True
  (lo usa `Dungeon.process` para esperar a la animación de los dados).

## Combate 5e (0.9, paso 3)
- Fuera de combate sigue el tiempo de PD (arriba). Ya es "simultáneo": el mundo no pasa a un héroe que espera input.
- Entra en combate apenas un actor `fighting()` (un mob HUNTING que ve a su héroe): `start_combat()` tira
  iniciativa a todos los personajes del piso (`in_rounds = True`) y arma `order` (mayor primero; empate = agregado antes).
- Ronda: `_combat_step()` deja actuar a `order[turn]` hasta `turn_over()` (ver `docs/game/char.md`); un héroe sin
  acción elegida frena el loop (`waiting`). Al terminar la ronda (`_end_round`), los actores de tiempo (estados,
  regeneración, hambre, respawner: `in_rounds = False`) avanzan 1 TICK = 1 ronda.
- Los que llegan a mitad (respawn, héroe que baja) tiran iniciativa y entran en la ronda siguiente.
- Sale del combate tras `CALM_ROUNDS` (2) rondas sin nadie `fighting()`: todos los personajes quedan con `time = now`.
- `current()`: a quién le toca (lo usan `Dungeon.blocking` y la UI).
- ponytail: el combate no se guarda (`game/save.py`); al cargar a mitad, se vuelve a tirar iniciativa.
