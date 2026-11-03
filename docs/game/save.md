# game/save.py

Guardado de partidas en JSON, uno por personaje: `<datos del usuario>/saves/<id del personaje>.json`
(`characters.data_dir()`: `%APPDATA%/PixelDND` o `~/.local/share/PixelDND`).

## Qué se guarda (`dump`)
- Partida: seed, estado del rng de juego (`getstate`), log, ítems conocidos, pesos del generador, cuotas.
- Héroes (`heroes`, lista; `local` = índice del héroe local): nombre, clase, características, nivel, XP, PV, posición,
  piso, hambre, oro, equipo y mochila (ítems como
  [id, cantidad, nivel, maldito, nivel conocido, maldición conocida]), accesos rápidos (id o índice del ítem),
  Segundo Aliento, espacios de conjuro, usos gratis y a quién marcó.
- Cada piso visitado: **no el nivel entero** (se regenera con la seed): el terreno (puertas, trampas, pasto pisado),
  lo visto, lo mapeado, los montones, los mobs y el planificador con el tiempo y el orden de cada actor (héroe i, mobs,
  regeneración / hambre del héroe i, respawner, estados; el dueño de un estado es `["hero", i]` o el índice del mob).

## Cuándo
Al salir al menú, al cerrar la ventana o Salir (`Screen.OnQuit`) y en cada cambio de piso. Antes aplica lo que
esperaba al dado (`resolve_pending`). Si murieron todos los héroes, borra el guardado (como el original).

## Cargar (`load_run`)
Crea la partida con la misma seed (mismas apariencias de pociones y pergaminos), restaura el rng y pisa todo lo
guardado. Un archivo roto, de otra `VERSION` o con contenido que ya no existe se aparta con otro nombre
(`.roto-<fecha>`), sin borrarlo. Escritura atómica (archivo temporal + reemplazo). Sin pickle: un guardado nunca
ejecuta código.

## Falta
El personaje portátil (estilo Terraria: llevar nivel y equipo a la partida de otro host) llega con la red (0.9).

## Versiones del formato
- `VERSION = 2` (0.9): varios héroes. `_upgrade` convierte un guardado v1 (un solo héroe) al leerlo: el héroe pasa a
  `heroes[0]` con el piso de la partida, y de las regeneraciones / hambres de cada piso (en v1 había una por piso) se
  queda sólo la del piso donde está el héroe (en v2 viajan con él).
- Cada héroe guarda también `dead`, `saves` y `stable` (salvaciones contra la muerte). Un guardado sin esos campos
  toma 0 PV como muerto.
- `rules`: reglas de la partida (`Dungeon.rules`). Un guardado sin ellas usa las de 5e.
