# data/*.json — contenido del juego

Registros que lee `game/registry.py`. IDs `namespace:nombre` (`pddnd:` = juego base). Textos: str o dict por idioma.

- **classes.json** (`HERO_CLASSES`): ver el comentario en `game/hero.py` (`dnd_class`, `hit_die`, `weapon_prof`,
  `armor_prof`, `save_prof`, `traits`, `spellcasting`, `start_weapon`, `start_armor`, `items` (id o [id, cantidad]),
  `quick`, `known`, `awareness_base`, `hunger_step`, `sprite`, `speed_ft` (5e, pies; 30)).
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
- **buffs.json** (`BUFFS`): ver `docs/game/buffs.md`. **spells.json** (`SPELLS`): ver `docs/game/spells.md`.
- Lo que sigue en código (no son datos): los efectos por nombre (`EFFECTS`, `TICKS`, `KINDS`), las tablas de espacios
  de conjuro y `FREE_CASTS`.
