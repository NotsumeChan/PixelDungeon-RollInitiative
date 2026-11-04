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
