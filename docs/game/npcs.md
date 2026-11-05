# game/src/npcs.rs

NPCs (`roadmap/npcs.md`, paso N1). Un NPC es un mob con `"npc": "<id de diálogo>"` en `data/mobs.json` que aparece
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
  `["class", id]`, `["level", n]`, `["not", condición]`. Una desconocida no se cumple.
- Efectos (`do`, en orden): `["give", ítem, n]` (mochila llena: al suelo), `["take", ítem, n]`, `["gold", ±n]`,
  `["flag", clave, valor]`, `["add", clave, ±n]`, `["log", texto]` (sólo para quien habla), `["map"]` (el mapa del
  piso, sin las puertas secretas), `["attack"]` (lo vuelve hostil y lo ataca: inicia el combate y lo sorprende),
  `["end"]`. Uno desconocido no hace nada.
- `Dungeon::flags` (clave -> entero): de la partida, compartidos en la LAN; se guardan.
