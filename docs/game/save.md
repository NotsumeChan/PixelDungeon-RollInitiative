# game/src/save.rs

`game/save.py`: una partida por personaje en `<datos>/saves/<id>.json`, formato propio (`format: "pdri-rust"`,
`version`). Sin código en el archivo.

- `dump`: seed y estado del rng, mensajes, identificados, pesos del generador, cuotas, reglas, contadores de ids, los
  héroes (equipo, mochila, accesos rápidos, rasgos, espacios, salvaciones contra la muerte, estados) y cada piso
  visitado: terreno, lo visto, montones, mobs (con sus estados) y el planificador con el combate en curso.
- `load`: arma la partida con la seed (mismas apariencias), regenera cada nivel y pisa lo que cambió (`Level::set`
  sólo en las celdas distintas). Lo que no se puede leer (otra versión, un ítem o estado que ya no existe) devuelve
  None y `load_run` aparta el archivo (`.roto-<hora>`), no lo borra.
- `save_run` aplica lo pendiente del dado y escribe atómico; sin héroes vivos, borra el guardado (como el original).

Personaje portátil (R6): `portable` = lo que viaja con el personaje entre partidas (nivel, XP, oro, arma, armadura,
mochila, accesos rápidos); el piso, la posición, los estados y los PV son de cada partida. `set_portable` lo aplica a
un héroe: sube los niveles que falten como al ganar XP (`Hero::gain_level`, nunca baja), descansa y deja los PV al
máximo. Frontera de confianza (lo manda otro jugador): `parse_portable` pide nivel 1-20, XP menor que la del nivel, oro
0-1.000.000, ítems que existen (cantidad 1-999, más de 1 sólo si es apilable, nivel -20..20), arma y armadura en su
ranura y no más que la mochila; si algo no valida no se aplica nada (`portable_ok` para preguntar antes).

Versión 2 (M1a, 0.9.26): los pisos se guardan por zona ("x,y,z") y el héroe con `zone`. Las partidas de la versión 1
no se leen (se apartan); los personajes quedan.
