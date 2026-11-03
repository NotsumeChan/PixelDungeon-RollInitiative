# game/buffs.py

Estados alterados (`actors/buffs/*.java`): veneno, sangrado, quemadura, parálisis, lisiado.
**Son datos:** `BUFFS[id]` con IDs `namespace:nombre` + una clase `Buff` genérica. Es la base para que los mods
agreguen o cambien estados.

## Para mods
- **Mod de datos:** agrega o pisa entradas de `BUFFS` combinando lo que ya existe. Nunca apunta a código.
- **Mod con código:** registra una función nueva en `TICKS` (`fn(buff)`) y la usa por nombre en `tick`.

| Campo | Qué hace |
|---|---|
| `name` | texto en la ficha y el log (obligatorio) |
| `icon` | índice en `Assets/buffs.png` (7x7, `BuffIndicator.java`); sin `icon` no se dibuja |
| `duration` | turnos por defecto. Sin duración dura hasta que su `tick` lo termine (`left = None`) |
| `tick` | comportamiento por turno, clave de `TICKS`: `poison`, `bleeding`, `burning` |
| `damage` | dados de daño por turno (`"1d4"`): daño continuo sin código |
| `speed` | multiplica la velocidad (`0.5` = la mitad) |
| `flags` | `"paralysed"`: no actúa, falla sola las salvaciones de FUE y DES, le atacan con ventaja |
| `fx` | efecto visual sobre el personaje mientras dura: `"flame"` (echa llamas, `screens/particles.py`) |
| `save_ends` | característica (5e) de la salvación que repite al final de cada turno; si la supera, se termina. CD = `Buff.dc` |

Ejemplo de mod de datos: `"mimod:acido": {"name": "Acido", "icon": 8, "duration": 5, "damage": "1d4"}`.

## Estados base
| ID | Java | Efecto |
|---|---|---|
| `pddnd:poison` | `Poison` | (turnos restantes / 3) + 1 de daño por turno |
| `pddnd:bleeding` | `Bleeding` | el nivel baja al azar entre la mitad y el total, y hace ese daño; en 0 termina |
| `pddnd:burning` | `Burning` | 1-5 por turno, 8 turnos; se apaga en el agua o al azar |
| `pddnd:paralysis` | `Paralysis` | 10 turnos, `paralysed`, salvación de CON cada turno (5e) |
| `pddnd:cripple` | `Cripple` | 10 turnos, velocidad a la mitad |

## Clase `Buff`
- Es un `Actor`: actúa en el planificador del piso una vez por turno. `left` (turnos), `level` (intensidad), `dc`.
- `act()`: `tick` -> `damage` -> resta un turno -> salvación de `save_ends` -> `detach()` si terminó o murió.
- Se aplica con `char.add_buff(id, duration, level, dc)` (ver `docs/game/char.md`). Los del héroe viajan con él
  al cambiar de piso (`Dungeon.go_to`).

## Pendiente
- Quemadura: no quema pergaminos ni prende el pasto (falta `Fire` / `Blob`).
- Pociones y pergaminos con estados (fase 4, segunda pasada) usan este mismo registro.

## Idioma
`name` puede ser un dict por idioma (los base lo son); `Buff.name` es una propiedad que usa `t()`.

## Marca del cazador
`pddnd:hunters_mark` (ícono 27): sin efecto propio ni duración; `Hero.roll_damage` suma 1d6 contra quien la tenga.

- La salvación de `save_ends` pasa el nombre del estado al recuadro del dado (`saving_throw(..., why)`).
