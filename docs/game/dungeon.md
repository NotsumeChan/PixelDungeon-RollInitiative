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
