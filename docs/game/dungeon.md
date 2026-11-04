# game/src/dungeon.rs

`game/dungeon.py`: la partida. `floors` guarda cada piso visitado (`Floor`: nivel, mobs, planificador); `depth` es el
piso en contexto. Varios héroes, cada uno en su piso; `hero` es el local (el de la UI).

- `go_to`: crea el piso la primera vez (héroe en la entrada, mobs con el rng del nivel, `Respawner`) o vuelve a uno
  ya visitado (al lado de la escalera por la que llega). El héroe se lleva su regeneración.
- `process`: corre cada piso con héroes vivos hasta que le toque decidir a un héroe, o haya algo en `pending`.
- Combate 5e: `start_combat` (todos tiran iniciativa d20 + DES), `combat_step`, `end_round`. `attack`: d20 + bono vs
  CA; 20 natural = crítico (dados dobles), 1 natural = falla; ventaja contra un mob que no te vio o un caído; caído y
  al lado = crítico. `spend` en combate: un paso gasta movimiento, cualquier otra cosa la acción.
- `pending`: lo visto por el héroe local espera a la animación del dado (`resolve_pending` lo aplica, sin rng nuevo).
  En R3 la app lo resuelve en el acto.
- `find_path` esquiva a las criaturas que se ven; `choose_enemy`: el héroe más cercano que el mob ve, prefiriendo a
  los que están en pie.
- `view_changed`: la vista del piso local cambió (la app sube la niebla).
- Falta: mezclar las apariencias de pociones y pergaminos al empezar (gasta rng en el prototipo; va primero de todo
  con los ítems, R5), ítems del piso, trampas.

Para el render (R4): `dice_events` (las tiradas que el héroe local ve: etiqueta, valor, mínimo, máximo y, en las
salvaciones, si se superó) y `fx` (`Fx::Anim`, `Status`, `Die`, `Checked`, `Leaves`). Sólo se emiten en el piso local
(`roll_event`, `effect`; `status` además pide que el personaje se vea). Los emite el juego en los mismos puntos que
el prototipo: ataque (animación, d20 y dado de daño), daño (número), esquiva, caer, levantarse, subir de nivel,
salvaciones y Medicina, iniciativa del héroe, buscar, muerte de un mob y pasto pisado.

Ítems (R5a): `known` y `looks` (identificación), `create_items` al crear un piso con el rng del nivel (comida
garantizada, cuotas de poción de fuerza y pergamino de mejora, 3+ montones / cofres / restos, tumbas del cementerio),
y el loot de los mobs. `place_char`: lleva a alguien a una celda lejana sin rastro de pasos (teletransporte).
Verificado: misma seed = mismas apariencias, mobs y montones (con nivel y maldición) que el prototipo.

R5b: `rules` (reglas de la partida, por defecto 5e: `dot_death_saves` = el daño por turno a un caído suma fallos,
`save_half` = superar el gas tóxico / la descarga = la mitad). `press` también activa trampas. La paralización da
ventaja contra el paralizado; los estados multiplican la velocidad (`start_turn`).

Emboscada (5e): si un héroe va a atacar fuera de combate, primero empieza el combate (`start_combat(true)`): todos tiran
iniciativa y su ataque va en su turno; los mobs que no lo habían visto quedan sorprendidos (`Char::surprised`: pierden
su primer turno). Así un enemigo que muere de un golpe igual tuvo su tirada de iniciativa. El combate que empieza
porque un mob caza (`start_combat(false)`) no sorprende a nadie, como el prototipo.
