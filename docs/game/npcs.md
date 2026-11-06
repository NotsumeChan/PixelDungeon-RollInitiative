# game/src/npcs.rs

NPCs (`roadmap/estado.md`, pasos N1 y N2). Un NPC es un mob con `"npc": "<id de diálogo>"` en `data/mobs.json` que aparece
tranquilo (`State::Passive`): no caza, no entra al combate, no corta el paso del héroe y no cuenta para el respawner.
Si lo dañan (o el diálogo lo ataca) se despierta como un mob dormido y pelea.

- `create_npcs`: al crear un piso, `data/npcs.json` (piso -> [[mob, probabilidad]]) con un rng propio derivado de la
  seed del piso: los pisos, sus mobs y sus ítems salen igual que sin NPCs.
- Tocar a un NPC tranquilo = `Action::Talk` (se acerca y abre el diálogo; en combate no). `Choose(k)` elige una opción,
  `Bye` cierra. No gastan tiempo. El diálogo abierto (`Hero::talking`: NPC y nodo) se guarda y viaja en el estado de
  la LAN: cada jugador ve el suyo.
- `dialog(i)`: lo que muestra la UI (`DialogView`): texto y las opciones cuyas condiciones se cumplen. `{name}` en un
  texto = el nombre del héroe.
- Condiciones (`if`, todas): `["has", ítem, n]`, `["gold", n]`, `["flag", clave, "=" | "!=" | ">=" | "<", valor]`,
  `["class", id]`, `["level", n]`, `["disposition", n]` (la actitud del NPC es al menos n), `["skill", pericia]` (la
  clase tiene competencia), `["not", condición]`. Una desconocida no se cumple.
- Efectos (`do`, en orden): `["give", ítem, n]` (mochila llena: al suelo), `["take", ítem, n]`, `["gold", ±n]`,
  `["flag", clave, valor]`, `["add", clave, ±n]`, `["log", texto]` (sólo para quien habla), `["map"]` (el mapa del
  piso, sin las puertas secretas), `["disposition", ±n]` (su actitud, entre -2 y 2), `["attack"]` (lo vuelve hostil y lo ataca: inicia el combate y lo sorprende),
  `["end"]`. Uno desconocido no hace nada.
- `Dungeon::flags` (clave -> entero): de la partida, compartidos en la LAN; se guardan.

Tiradas sociales (N2): una opción con `"check": [pericia, CD]` tira d20 + la característica de la pericia
(`data/skills.json`) + la competencia si la clase la tiene (`skill_prof`), con el dado grande, y lo anota en el log de
todos. Bien: `go` y `do`; mal: `fail` y `fail_do`. La UI muestra la pericia, el bono y la CD en el botón.
- Actitud (`Mob::disposition`, -2 hostil .. 2 muy amistoso; inicial `disposition` en `mobs.json`): cada punto baja la
  CD en 2 (desconfiado la sube); la ventana la muestra arriba a la derecha. Se guarda con el piso.
- Atacar a un NPC (pegarle, o el efecto `attack`) hace que los otros NPCs tranquilos que lo vieron desconfíen más (-1).
- `skill_bonus` también lo usa Medicina al estabilizar; `proficient_skills`, la ficha.

Misiones (N3, `data/quests.json`): `name`, `desc` y `objectives` (`["kill", mob, n]`: muertes de ese mob desde que se
aceptó, contadas en el flag `kills:<mob>`; `["has", ítem, n]`: en la mochila o equipado). El estado es el flag
`quest:<id>` (0 sin empezar, 1 en curso, 2 hecha): de la partida, compartido en la LAN.
- Condición `["quest", id, "none" | "active" | "ready" | "done"]` ("ready" = en curso y con todo cumplido).
- Efectos `["quest", id, "start" | "done"]` (el log de todos lo avisa) y `["place", ítem]` (en unos restos de esqueleto
  del piso, o en unos nuevos). La recompensa la dan los efectos de la opción que entrega.
- `quest_lines`: las empezadas con su progreso, para la ficha.

Inventario, equipo y tienda (N4): en `mobs.json`, `inventory` ([[ítem, n]]), `stock` ([[categoría, n]], sacados del
generador sin maldiciones), `weapon` y `armor`. Un NPC con arma pega con sus dados y con armadura tiene su CA (DES =
`initiative`); al morir suelta todo. Se guarda con el piso (`Mob::pack`, `weapon`, `armor`).
- Efecto `["shop"]`: cierra la charla y abre su tienda (`Hero::trading`). `Action::Buy(k)` compra una unidad del ítem
  k del NPC; `Action::Sell(k)`, una del ítem k de la mochila (lo equipado no). No gastan tiempo; `Bye` cierra.
- Precio (`price` de items.json): comprar = precio -10 % por punto de actitud (+10 % si desconfía); vender = la mitad
  +10 % por punto. Precio 0 = no está a la venta / no le interesa. Lo vendido pasa a su inventario.
- `give` saca del inventario del NPC si lo tiene (si no, crea uno); `take` lo guarda en su inventario.
- `shop(i)`: lo que muestra la UI (`ShopView`).

Mercader ambulante (mod incluido `data/mods/mercader-ambulante`): `create_npcs` suma la tabla `"*"` de `npcs.json`
(cualquier piso) a la del piso; `mob_passive` (`mobs.rs`) hace deambular a un NPC con `"wander": true` (`mob_roam`,
igual que un mob que deambula) salvo con un héroe al lado o hablando o comerciando con él.
