# data/*.json — contenido del juego

Registros que lee `game/registry.py` (los mods los pisan o suman: `docs/mods.md`). IDs `namespace:nombre` (`pdri:` = juego base). Textos: str o dict por idioma.

- **classes.json** (`HERO_CLASSES`): ver el comentario en `game/hero.py` (`dnd_class`, `hit_die`, `weapon_prof`,
  `armor_prof`, `save_prof`, `traits`, `spellcasting`, `start_weapon`, `start_armor`, `items` (id o [id, cantidad]),
  `quick`, `known`, `awareness_base`, `hunger_step`, `speed_ft` (5e, pies; 30), `ability_priority` (las 6
  características de la más a la menos importante: la creación de personaje les da las tiradas más altas),
  `unarmored_ac` (característica de la Defensa sin armadura), `features` (rasgos con usos: [conjuro, base, cada]),
  `spellcasting.slots` (full / half / pact)).
- **mobs.json** (`MOBS`): `name`, `desc`, `sprite`, `frame` [ancho, alto], `anims` (nombre -> [fps, [frames], se repite]),
  `HT`, `defense`, `damage` [mín, máx], `attack`, `dr`, `exp`, `max_lvl`, `speed` (PD, fuera de combate), `speed_ft` (5e, pies; 30), `initiative` (mod DES; 0), `flying`, `loot` [id, prob], `saves`.
  **bestiary.json**: piso -> [[mob, peso], ...].
- **items.json** (`ITEMS`): todos tienen `name`, `category`, `image` (entero de `items.png` o [archivo, índice]; opcional: sin él se dibuja el nombre),
  `gen_weight`, `stackable`, `desc`.
  - `weapon`: `tier`, `dice`, `dly` (tiempo por ataque), `kind` (simple / martial), `finesse`, `STR` (10 + tier*2).
    Arco: `ranged`, `ammo`, `range` [normal, largo].
  - `armor`: `tier`, `ac`, `dex_max` (null = sin tope), `kind` (clothing / light / medium / heavy), `STR` (8 + tier*2).
  - `missile`: `thrown`, `dice`, `kind`, `finesse`, `range`, `STR`, `dly`, `qty` [mín, máx], `spin`, `on_hit`
    [estado, duración, nivel, CD], `returns`, `upgradable`. Munición: `flight_image`.
  - `food`: `energy`. `potion` / `scroll`: `effect` (nombre en `items.EFFECTS`), `target`.
- **skins.json**: apariencias de los héroes, aparte de la clase: id -> `name` y `sprite` (hoja de
  `Assets/Characters/` con el layout del héroe del original: una fila por armadura). Se eligen en la creación de
  personaje. Las del juego base son los 4 héroes de Pixel Dungeon, con los ids de sus clases de antes.
- **skills.json**: pericias de 5e, id -> `name` y `ability`. En `classes.json`, `skill_prof` = las pericias con
  competencia.
- **items.json** `price`: precio de una unidad en la tienda (0 = no se vende; vender da la mitad).
- **quests.json**: misiones: id -> `name`, `desc`, `objectives` ([["kill", mob, n] | ["has", ítem, n]]); los diálogos
  las empiezan y las entregan (`docs/game/npcs.md`). Un ítem de misión: `category` "quest" y `gen_weight` 0.
- **NPCs** (`docs/game/npcs.md`): en `mobs.json`, `npc` (id del diálogo) hace NPC al mob y `disposition` (-2..2) es su
  actitud inicial; `inventory` ([[ítem, n]]), `stock` ([[categoría, n]] al azar), `weapon` y `armor` (su equipo). **npcs.json**: piso (o `"*"`, cualquier piso) ->
  [[mob, probabilidad (número o lista por profundidad: piso 1, 2...; después, 0), opciones (`{"near": "exit"}`: junto a la bajada)]]; `"wander": true` en el mob: deambula. **dialogs.json**: id -> `start` (nodo inicial) y `nodes`: nodo -> `text` y `options`
  ([{`text`, `if`, `do`, `go`}, y para una tirada `check` [pericia, CD], `fail` y `fail_do`]; sin `go`, la charla
  termina).
- **buffs.json** (`BUFFS`): ver `docs/game/buffs.md`. **spells.json** (`SPELLS`): ver `docs/game/spells.md`.
- Lo que sigue en código (no son datos): los efectos por nombre (`EFFECTS`, `TICKS`, `KINDS`), las tablas de espacios
  de conjuro y `FREE_CASTS`.
