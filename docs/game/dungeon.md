# game/dungeon.py

`Dungeon(seed, hero_class)`: una partida completa. Equivale a lo que en Java es estático en `Dungeon.java` +
`Actor`. Vive en `Globals["dungeon"]`.

## Contenido
- `seed`, `heroes`, `hero` (el local: el que maneja la UI), `messages` (log, últimos 50).
- `depth`, `level` (`SewerLevel`), `mobs`, `scheduler`: el **piso en contexto** (ver "Varios héroes").
- `rng`: rng de juego (combate, IA, respawn), seed = la de la partida. La generación de cada piso usa su
  propio rng (`level_seed`), así que el mapa no depende de lo que pasó en el piso anterior.

## Pisos
- `descend(hero)` / `ascend(hero)` -> `go_to(hero, depth)`. En el piso 1, subir sólo avisa que no se puede salir.
- `floors`: todos los pisos visitados (incluido el actual), `depth -> (level, mobs, scheduler)`. Al irse un héroe,
  sale del planificador con lo suyo (`hero.ticks` = `Regeneration` + `Hunger`, y sus `Buff`); el `Respawner` se queda.
  Un piso sin héroes vivos no se procesa: queda congelado y al volver su tiempo no avanzó.
- Al llegar, el héroe cae en la escalera o, si está ocupada (otro héroe, un mob), en una vecina libre (`free_near`).
- Bajando se llega a la entrada; subiendo, a la salida del piso de arriba.
- **Piso nuevo:** genera el nivel, crea un `Scheduler` (héroe primero, luego mobs, `Respawner`, `hero.ticks`) y recarga el Segundo Aliento (sustituto del descanso; sólo en pisos nuevos para que subir y bajar
  no sea una recarga infinita).
- `go_to` corre dentro de `process()`: `process` corta el loop cuando `self.scheduler` cambia.
- `MAX_DEPTH = 4`: en la salida del piso 4 sólo avisa que es el fin de la demo.

## Ítems
- `known` / `looks`: identificación de pociones y pergaminos (ver `docs/game/items.md`).
- `generator` (se resetea en cada piso), `potions_of_strength`, `scrolls_of_upgrade` (cuotas garantizadas).
- `create_items()`: comida garantizada + cuotas de fuerza y mejora + `3 + (40% encadenado)` ítems al azar en
  montones, cofres o esqueletos + el contenido de las tumbas del cementerio. Usa el rng del nivel.

## Mobs
- `create_mobs()`: `2 + piso % 5 + Int(3)` mobs del Bestiary, dormidos, en salas STANDARD.
  Usa el rng del nivel: misma seed y piso = mismos mobs en el mismo lugar.
- `Respawner`: cada 50 de tiempo, si hay menos mobs que el objetivo, aparece uno despierto fuera de la vista.

## Utilidades
- `find_char(celda)`, `chars()`, `remove_char(c)`.
- `find_path(ch, desde, hasta, passable, visible)`: como `Dungeon.findPath`. Las criaturas que `ch` ve
  bloquean el paso; los voladores además pueden pasar por celdas `avoid`.
- `choose_enemy(mob)`: el héroe vivo más cercano que el mob ve (`mob.fov`); si no ve a ninguno, el más cercano del
  piso. Prefiere a los que están en pie; a un caído sólo si no hay otro. Empate: el primero de `heroes`.
- `process()`: corre los turnos de cada piso con héroes vivos hasta que cada uno espere a su héroe.
- `blocking()`: el héroe del piso en contexto al que el planificador espera (o None). Lo usa el modo de prueba.

- `dice_events` / `roll_event(etiqueta, valor, mín, máx)`: tiradas que el héroe vio, para la animación (máx. 20 en cola).

- `pending` / `resolve_pending()`: un ataque que el héroe ve no se aplica al momento. Queda en `pending` y
  `process()` frena el planificador hasta que la UI termina de mostrar el dado y llama `resolve_pending()`, que
  escribe el log y aplica el daño. No usa rng, así que el orden de las tiradas (y el determinismo) no cambia.
  Los ataques que el héroe no ve se aplican al instante.
- `verbose_rolls`: opción de desarrollador (`[dev] dice_log` en `Local.config`). True = el log muestra el cálculo
  completo (`d20 14+5=19 vs CA 12, 1d6(4)+3=7`); False = mensaje simple (`Warrior golpea a marsupial rat: 7`).

## Trampas y estados
- `press(celda, ch)`: lo llama `Char.move`; delega en `game/traps.py`.
- `pending` acepta también funciones sin argumentos (salvaciones de trampas): `resolve_pending` las llama en orden.
- `add_mob(mob, delay)`: la invocación de trampas suma mobs que actúan 2 turnos después.
- Al cambiar de piso, los `Buff` del héroe salen del planificador viejo y entran al nuevo.

## Efectos para el render (`fx`)
- `fx`: lista que la pantalla consume cada frame. `status(ch, texto, tipo)` = texto flotante sobre el personaje
  (`CharSprite.showStatus`), sólo si se ve; tipos `default`, `pos`, `neg`, `warn`, `neutral`. `effect("checked", celda)`
  (búsqueda) y `effect("leaves", celda)` (pasto pisado).
- Se emiten: daño (`Char.damage`), "esquiva" (ataque fallado, al resolverse), estado nuevo, salvación de trampa,
  "+PV" del Segundo Aliento, "sube de nivel", "buscando".
- `roll_event(..., ok)`: `ok` = resultado de una salvación (el recuadro del dado usa borde verde/rojo).
- `press`: el pasto alto pisado pasa a pasto (`HighGrass.trample`, sin semillas ni rocío) y se recalcula la vista.

## Recuperación
Al llegar a un piso nuevo llama a `hero.recover()` (Segundo Aliento, espacios de conjuro, usos gratis).

## Varios héroes (0.9, paso 1)
- `heroes`: todos; cada uno con su piso (`hero.depth`) y su vista (`hero.fov`). `hero` = el local (la UI);
  `set_local(h)` lo cambia. `add_hero(clase, características)` suma uno en la entrada del piso 1.
- **Piso en contexto:** `level`, `mobs`, `scheduler` y `depth` son los del piso que se está procesando (`enter(depth)`).
  Así el código de personajes, trampas e ítems sigue usando `dungeon.level` sin saber de pisos. Fuera de `process()`
  (input, render) el contexto es el piso del héroe local.
- `here()`: héroes del piso en contexto. `chars()` = `here()` + `mobs`.
- `observe()`: recalcula la vista de cada héroe del piso; `level.visible` es la unión (lo que se dibuja y lo que
  decide si algo "se ve"). Las decisiones de cada héroe (atacar, apuntar, buscar, interrumpirse) usan `hero.fov`.
- `sees(celda)`: piso del héroe local y a la vista. Lo que se ve espera al dado (`pending`) y se resuelve con el
  contexto del piso local, así que lo de otros pisos se aplica en el acto. `effect` y `roll_event` descartan lo que no
  es del piso local.
- XP: ver `docs/game/mobs.md`. Muerte: ver `docs/game/hero.md`.
- Trampa: el log es uno solo (en el modo de prueba se mezclan los mensajes de los dos héroes).
- `defer(p)`: agrega a `pending`; si el piso en contexto no es el del héroe local, lo resuelve ya (no hay dado que
  mirar). Lo usan los conjuros y las salvaciones contra la muerte.

## Reglas de la partida (`rules`)
- `RULES` (valores por defecto = 5e) -> `dungeon.rules`, se guarda con la partida. Las cambia la ventana "Reglas de la
  partida" (menú de la partida, `screens/gameUi.py`), estilo configuración de mundo de Don't Starve Together. En red
  las decidirá el host.
- `dot_death_saves`: el daño por turno (un `Buff`: veneno, sangrado, quemadura) a un héroe caído suma fallos.
- `save_half`: superar la salvación del gas tóxico o la descarga = la mitad del efecto; False = sin efecto.
- Para sumar una regla: clave en `RULES`, fila en `gameUi.RULE_ROWS` y leerla con `d.rules["clave"]`.

## Línea de tiro de los héroes (`shot`)
`shot(desde, hasta)` -> (celda donde se frena, cobertura). Como `ballistica.cast`, pero pasa por los otros héroes
(no hay fuego amigo con proyectiles ni conjuros) y, si había alguno en medio, el objetivo tiene media cobertura (5e):
-2 a la tirada (`hero.cover`). Lo usan lanzar, el arco y los conjuros con objetivo.

## Combate 5e (0.9)
`blocking()` usa `scheduler.current()`: en combate, el héroe al que le toca por iniciativa (lo usa el hotseat).

`remove_hero(hero)`: un jugador de red se va; sale del planificador de su piso con lo suyo y de `heroes`. Nunca saca
al héroe local.
