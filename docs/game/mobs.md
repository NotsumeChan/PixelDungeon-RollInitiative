# game/mobs.py

Mobs de las Alcantarillas. Port de `actors/mobs/Mob.java` (IA), `Bestiary.java` y las stats de cada mob.

## Datos, no clases
- IDs de texto `pddnd:rat`, `pddnd:gnoll`... (namespace del juego base; los mods usan el suyo).
- `MOBS[clave]` = dict de stats: `name`, `sprite` / `frame` (archivo y tamaño del primer frame), `HT`,
  `defense`, `damage=(min, max)`, `attack`, `dr`, `exp`, `max_lvl`, y opcionales `speed` y `flying`.
- `BESTIARY[piso]` = `[(clave, peso)]`. `pick_mob(rng, piso)` elige uno.
- Una sola clase `Mob` lee esas stats. **Este es el punto de entrada para los addons:** agregar un mob es
  agregar una entrada a `MOBS` y sumarla a `BESTIARY`. Ver `roadmap/futuro-mods-personajes-red.md`.

## Combate 5e (conversión desde PD)
Las stats de la tabla siguen siendo las del original; se convierten al vuelo:
- `mob_ac` = `8 + (defense + dr) // 2`: Rat 10, Gnoll 11, Crab 12, Swarm 10, Skeleton 15, Thief 15. Antes era base 10 y los primeros pisos eran muy duros.
- `mob_attack_bonus` = `attack // 2`: Rat +4, Gnoll +5, el resto +6.
- Daño: el rango del original (`NormalIntRange`); con crítico se tira dos veces.
- `is_unaware()` = no vio al enemigo (dormido o sorprendido): se le ataca con **ventaja** (en PD su evasión era 0).
- Si el balance no convence, se ajustan esas dos funciones.

## IA (estados de Mob.java)
Cada turno el mob calcula su propio campo de visión. Su enemigo es `dungeon.choose_enemy` (hoy, el héroe).

| Estado | Qué hace |
|---|---|
| SLEEPING | si ve al enemigo, despierta con probabilidad `1 / (distancia + stealth)`; pasa a HUNTING |
| WANDERING | camina hacia un destino al azar; si ve al enemigo (prob. `1 / (dist/2 + stealth)`) o lo atacaron, HUNTING |
| HUNTING | si está al lado ataca; si no, va hacia la última posición vista; si la pierde, vuelve a WANDERING |

- Los mobs nacen dormidos. Los que aparecen después (`Respawner`) nacen deambulando.
- Si no vio al enemigo, se le ataca con ventaja (ataque sorpresa).
- Recibir daño despierta al mob y lo pone en alerta (al siguiente turno pasa a cazar).
- Al morir da `exp` al héroe si su nivel es <= `max_lvl`, y suelta `loot = (id, probabilidad)` si su nivel es
  <= `max_lvl + 2`: Gnoll oro (50%), Crab mystery meat (16,7%).

## Pendiente
- Habilidades especiales: Swarm se divide, Skeleton explota al morir, Thief roba y huye, Skeleton suelta armas.
- Estados FLEEING y PASSIVE (necesitan `getStepBack`).
- Animaciones (el render usa el primer frame).

## Salvaciones, estados y trampas
- `save_bonus(car)`: `"saves"` en la entrada de `MOBS` (`{"CON": 3}`) o, si no hay, `attack // 4` (Rat y Gnoll +2, el resto +3).
- `beckon(celda)`: trampa de alarma; si no está cazando pasa a WANDERING con esa celda como destino.
- Paralizado: pierde el turno y no te ve (`enemy_seen = False`, se le ataca con ventaja).
- Pisan trampas visibles (no las ocultas) salvo los voladores. Ver `docs/game/traps.md`.

## Textos
- `name` y `desc` por idioma (español e inglés del original, `description()` de cada mob); `desc` la usa la ventana
  de examinar. `Mob.name` es una propiedad: siempre en el idioma actual (el setter ignora el `"?"` de `Char`).
- `MOB_STATES`: texto de cada estado de la IA (`AiState.status`), por idioma.

## Animaciones
`anims` en cada entrada de `MOBS` (las usa `screens/charSprite.py`), armado con `_anims(idle, run, attack, die)`
con los `(fps, frames)` de cada `*Sprite.java`. `frame` es el tamaño de cada frame. `die()` emite `("die", mob)`
para que el render deje el cadáver animado.

## Varios héroes (0.9)
- `choose_enemy` (`Dungeon`): el héroe vivo más cercano que el mob ve.
- XP al morir: cada héroe vivo del piso recibe `ceil(exp / n)` (con `exp` según su propio nivel, `max_lvl`) y el que
  dio el último golpe (`src`) +1 si hay más de uno. Con un solo héroe es igual que el original.
- Loot: se compara `max_lvl + 2` con el héroe de menor nivel del piso.

## Combate 5e (0.9)
- `fighting()`: HUNTING y viendo a su héroe = mete al piso en combate (`game/actor.py`).
- `speed_ft` (30 por defecto) e `initiative` (mod de DES, 0 por defecto) salen de `data/mobs.json`. El `speed` de PD
  sólo vale fuera de combate. Cada turno: camina hasta sus casillas y ataca si llega al lado.
- ponytail: el mob hace todos sus pasos en el mismo frame; si el sprite "salta", animar paso a paso.
