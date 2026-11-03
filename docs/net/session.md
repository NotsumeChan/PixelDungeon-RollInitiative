# net/session.py

- `HostGame(dungeon, host)`: `pump()` una vez por frame. `hello` -> valida versión, hash y personaje (clase registrada,
  características con `dice.valid_scores`) y suma el héroe con `Dungeon.add_hero` (o manda `refused` y corta);
  `action` -> `actions.apply` con el piso de ESE héroe en contexto; `leave` -> `Dungeon.remove_hero`.
  Si el estado (`save.dump`, sin la hora) cambió, manda `{"t": "state", "you": índice, "save": ...}` a cada cliente.
- `ClientGame(client, version, name, class, abilities)`: manda `hello`; `send(net.actions...)`; `pump()` devuelve una
  `Dungeon` armada con el último estado (`save.load` con `local = you`) o None. `refused` / `closed` para la UI.
- ponytail: el cliente rearma la Dungeon entera en cada estado (regenera pisos con la seed: CPU y sprites que
  saltan). El paso 6 lo aplica sobre la existente, con ids estables por personaje.
- ponytail: el personaje entra nuevo (nivel 1); el personaje portátil es el paso 5. Los dados, textos flotantes y
  efectos del host no viajan todavía (el log sí, va en el guardado).
