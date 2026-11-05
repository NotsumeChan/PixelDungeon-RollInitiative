# NPCs (pedido del dueño, 2026-11-05)

Meta: NPCs como en D&D. Hablar con ellos, conseguir misiones, objetos y pistas, comprar y vender, y también
persuadirlos, mentirles, intimidarlos, coquetearles o atacarlos, con tiradas de 5e. Todo como **datos**
(`data/*.json`): el juego base y los mods usan el mismo formato.

Estado: plan confirmado por el dueño (2026-11-05, ver "Decisiones del dueño").

## Modelo (propuesta)
- **NPC = mob con diálogo**: en `data/mobs.json`, `"npc": "<id del diálogo>"` y `"disposition"` inicial. Así reusa
  sprite, animaciones, guardado, red y combate. IA nueva `State::Passive`: no caza, deambula cerca de su casilla
  o se queda quieto. Tocarlo = **Hablar** (`Action::Talk(mob)`); en combate no se habla.
- **Disposición** (actitud de 5e: hostil / indiferente / amistoso, como número -2..2): la mueven las tiradas
  sociales y las acciones. Cambia las CD y lo que ofrece el diálogo. Atacarlo lo vuelve hostil (pasa a
  `Hunting`) y baja la disposición de los NPC que lo vean.
- **Diálogos** (`data/dialogs.json`): árbol de nodos. Nodo = `text` + `options`. Opción = `text`, `if`
  (condiciones), `check` (tirada: `{"skill", "dc", "ok", "fail"}`), `do` (efectos), `go` (nodo siguiente o fin).
  - Condiciones: `has` [ítem, n], `gold` n, `flag` [clave, op, valor], `quest` [id, estado], `disposition` n,
    `class` id, `level` n.
  - Efectos: `give` / `take` [ítem, n], `gold` ±n, `flag` [clave, valor], `quest` [id, estado], `disposition` ±n,
    `hostile`, `shop` id, `log` texto (pistas), `map` (revela el piso), `end`.
  - Textos `{"es", "en"}` como el resto del contenido.
- **Pericias 5e** (`data/skills.json`: id -> característica y nombre; Persuasión, Engaño, Intimidación,
  Interpretación, Perspicacia, ...) y `skills` (competencias) en `classes.json`. Tirada = d20 + mod + competencia
  contra la CD, con el dado grande (como las salvaciones). Coquetear = Persuasión o Interpretación, según el
  diálogo.
- **Misiones** (`data/quests.json`): nombre, descripción, objetivos como condiciones (`kill` [mob, n] con el
  contador `kills:<mob>`, `has` ítem, `flag`) y recompensa (efectos). Ventana de misiones en la ficha.
- **Comercio** (`data/shops.json`): stock fijo [ítem, cantidad] o sacado del generador; precio = `price` de
  `items.json` (nuevo campo), modificado por la disposición. Vender = la mitad. Ventana de tienda con la mochila.
- **Colocación**: `npcs` por piso (como `bestiary.json`), con un rng propio derivado de la seed del piso: no gasta el
  rng del nivel, así los pisos y los mobs siguen saliendo iguales.
- **Red**: `Talk` y `Choose(k)` son acciones como las demás (el host las valida). El diálogo abierto va en el estado
  del héroe: cada jugador ve el suyo y lo importante sale en el log de todos ("X convence a Y").
- **Guardado**: disposición, flags, contadores y misiones.

## Contenido base (del original, con sus sprites)
- **Fabricante de varitas** (`Wandmaker.java`, `wandmaker.png`): el NPC inicial. En el original aparece en las Prisiones
  y pide un objeto a cambio de una varita; acá, en las Alcantarillas, con su misión adaptada.
- **Mercader** (`shopkeeper.png`): en el original aparece en el piso 6; acá, uno en las Alcantarillas.
- Fuera por ahora (decisión del dueño): el Fantasma triste (`Ghost.java`).
- Para después (sprites que ya están en `Assets/Characters/`): blacksmith, imp, ratking, ghost.

## Pasos (cada uno se prueba a mano)
1. **N1 motor**: NPC pasivo, Hablar, diálogos como datos con condiciones y efectos, flags, ventana de diálogo, red.
2. **N2 social**: pericias y tiradas, disposición, atacar a un NPC.
3. **N3 misiones**: contadores, recompensas, ventana de misiones; el Fabricante de varitas.
4. **N4 comercio**: precios, tienda, comprar y vender; el Mercader.
5. **N5 mods**: el formato en `docs/data.md` y un mod de ejemplo con un NPC propio.

## Decisiones del dueño (2026-11-05)
1. Contenido inicial: el Fabricante de varitas y el Mercader; el Fantasma, por ahora no.
2. Misiones, flags y disposición son **de la partida** (el mundo): en la LAN el grupo las comparte; las recompensas
   quedan en el héroe que las recibe.
3. Tiradas sociales con **pericias 5e** (d20 + característica + competencia de la clase).
4. Atacar a un NPC lo **vuelve hostil** (pelea o huye) y los NPC que lo ven te tratan peor; se lo puede matar.
