# game/globals.py

`Globals: dict` es el diccionario global del juego, para el estado que comparten pantallas y sistemas.

```python
from game.globals import Globals
d = Globals.get("dungeon")
```

Está en `game/` (sin pygame) para que lo puedan importar tanto la lógica como las pantallas.

## Claves en uso
| Clave | Tipo | Quién la escribe | Qué es |
|---|---|---|---|
| `dungeon` | `game.dungeon.Dungeon` | `screens/gameScreen.NewGame` | partida actual (seed, piso, nivel, héroe, mobs) |
| `character` | dict | `screens/rosterScreen` / `createScreen` | personaje elegido (ver `docs/game/characters.md`) |
| `runs` | dict | `screens/gameScreen.NewGame` | id de personaje -> su `Dungeon` en memoria |

Al agregar una clave, anotarla en esta tabla. Las claves `seed`, `depth`, `level` y `hero_pos` de la fase 2
ya no existen: ahora están dentro de `Dungeon`.

## Ojo con el P2P
Es estado local de cada proceso. Lo que deba ser igual en todos los peers (seed, piso) tiene que
llegar por la red. No asumir que otro peer tiene el mismo `Globals`.
