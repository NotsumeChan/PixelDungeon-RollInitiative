# game/items.py

Ítems: port de `items/*.java`, `Generator.java`, `Heap.java` e `ItemStatusHandler` (apariencias sin identificar).

## IDs de texto
Todo el contenido usa IDs `namespace:nombre`. El juego base usa **`pddnd:`** (no `pd:`, para no representar al
original); un mod usará el suyo (`mimod:espada_rota`). Nunca IDs numéricos.

## `ITEMS[id]` (datos)
| Campo | Para qué |
|---|---|
| `name`, `category` | categorías: `gold`, `food`, `potion`, `scroll`, `weapon`, `armor` |
| `image` | índice en `Assets/Items/items.png` (grilla de 8x16 px, `ItemSpriteSheet.java`). Pociones y pergaminos usan su apariencia |
| `stackable` | se apila en la mochila y en el suelo |
| `gen_weight` | peso dentro de su categoría en el Generator (0 = nunca al azar) |
| `energy` | comida: cuánto hambre quita |
| `effect` | poción/pergamino: **nombre** de una función de `EFFECTS`. Un mod de datos sólo puede nombrar efectos existentes |
| `target` | pergamino que pide elegir un objeto: nombre de un filtro de `TARGETS` (`unknown`, `upgradable`) |
| `dice`, `kind`, `finesse`, `dly`, `STR`, `tier` | armas (5e): dados de daño, `simple`/`martial`, sutil (FUE o DES), tiempo por ataque, FUE mínima `10+tier*2` = 12..20 (debajo: desventaja y -1 al ataque por punto que falta). `weapon_dice(item)` suma el nivel |
| `ac`, `dex_max`, `kind`, `STR`, `tier` | armaduras (5e): CA base, tope de DES (`None` sin tope, 0 no suma), `clothing`/`light`/`medium`/`heavy`, FUE mínima `8+tier*2` = 12..18, la ropa no pide (debajo: más lento y desventaja al atacar). `armor_ac(item, mod_des)` suma el nivel |

Tabla de dados y CA de cada arma y armadura: `roadmap/dnd5e.md`.

## Contenido actual
- **Comida:** ration (260), pasty (360) y mystery meat (100, sólo la suelta el Crab, sin efectos raros todavía).
- **Pociones:** healing, experience, strength.
- **Pergaminos:** identify, teleportation, remove curse, magic mapping, upgrade.
- **Armas cuerpo a cuerpo:** las 11 del original (tier 1–5).
- **Armaduras:** las 5 del original.
- **Oro.**

Falta todo lo que necesita buffs, gases o lanzar: el resto de pociones y pergaminos, varitas, anillos, semillas,
armas arrojadizas y bombas.

## Item (instancia)
`id`, `quantity`, `level`, `cursed`, `level_known`, `cursed_known`. `split(n)` saca n de una pila.

## Identificación
- Cada partida reparte al azar (con `dungeon.rng`) los 12 colores de poción y las 12 runas de pergamino entre
  las pociones y pergaminos registrados (`dungeon.looks`).
- `dungeon.known` = qué tipos ya se conocen (al usarlos, con identify, o por clase al empezar).
- Equipo: nivel y maldición se conocen al equiparlo o con identify. En Java se descubren con el uso: simplificado.
- `item_name`, `item_image`, `item_info` muestran sólo lo que el jugador sabe.

## Generator
- Pesos de categoría como el original (arma 15, armadura 10, poción 50, pergamino 40, oro 50). Cada vez que sale
  una categoría, su peso se divide a la mitad; se resetea en cada piso.
- Armas y armaduras: se sacan dos y queda la de FUE requerida más cercana a la FUE del héroe.
  `roll_equipment`: 40% tiene nivel ±1..3; los negativos salen malditos.
- `make_gold(rng, piso)`: `Int(20 + piso*10, 40 + piso*20)`.

## Heap (montón en el suelo)
- Tipos `heap`, `chest`, `skeleton`, `tomb`. Los tres últimos se abren al pisarlos y pasan a `heap`.
- `items[0]` es el de arriba (el que se ve y se recoge primero). `add` apila los apilables.

## Efectos (`EFFECTS`, firma `(hero, item, target)`)
`heal`, `strength`, `experience`, `identify`, `upgrade` (+1 y quita la maldición), `remove_curse`,
`magic_mapping` (marca `mapped` y descubre secretos), `teleport`.

## Nombres por idioma
`name` y `desc` son dicts por idioma (`{"es": "daga", "en": "dagger"}`, ver `docs/game/lang.md`); `item_name` e
`item_info` usan `t()`. Las apariencias también: `POTION_LOOKS` + `POTION_NAME` ("poción turquesa" / "turquoise
potion"); los pergaminos conservan las runas (`SCROLL_NAME`). Los IDs (`pddnd:dagger`) no cambian.

## Armas a distancia
- **Arrojadizas** (`category: "missile"`, `thrown`, helper `_missile`; `items/weapon/missiles/`): dardo, dardo
  incendiario, dardo de curare, jabalina, shuriken, tomahawk y el **bumerán** de la Cazadora. Campos: `dice`, `kind`,
  `finesse` (si no, FUE), `range` = (normal, largo) en casillas (5 pies = 1), `qty` (montón al generarse), `STR` (la
  del original), `dly`, `spin` (giro en vuelo; 0 = vuela apuntando), `on_hit` = (estado, duración, nivel, CD) al
  acertar, `returns` (vuelve a la mano), `upgradable` (el bumerán acepta el pergamino de mejora).
- **Arco corto** (`pddnd:shortbow`, categoría `weapon`, `ranged`, `ammo`, `range` 16/64): no es del original.
  **Flechas** (`pddnd:arrow`, munición, no se lanzan a mano).
- Como en PD, lo que acierta se pierde y lo que falla cae al suelo (salvo el bumerán).
- Generador: categoría `missile` (peso 5; `weapon` bajó de 15 a 10 para que el total quede igual que en PD, donde
  las arrojadizas son parte de las armas). Los dardos sólo vienen en el equipo inicial (`gen_weight` 0, como en PD).
- `image` puede ser un entero (`items.png` del original) o `(archivo, índice)` de otra hoja: el arco y las flechas
  usan `Assets/Items/pddnd_items.png` (`PDDND_SHEET`). `flight_image` = sprite en vuelo (flecha suelta).

## Varios héroes (0.9)
- Mapeo mágico y teletransporte recalculan la vista con `Dungeon.observe()` (todos los héroes del piso).

`item_image`: entero (items.png), `[archivo, índice, nombre]` (hoja propia o de mod; el nombre es el fallback si no
carga) o `["text", nombre]` si el ítem no tiene `image` (arco corto y flechas, hasta que haya sprites hechos por una
persona). La flecha en vuelo usa `flight_image` o, sin él, el mismo ícono.
