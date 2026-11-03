# game/char.py

Base de héroe y mobs (`actors/Char.java`): vida, movimiento y el **combate con reglas de D&D 5e**.

## Ataque (`attack`)
1. `d20` con ventaja o desventaja (`advantage(objetivo)`).
2. Golpea si `d20 + attack_bonus >= armor_class` del objetivo. **20 natural:** siempre golpea y es crítico
   (dados de daño dobles). **1 natural:** siempre falla.
3. Daño = `roll_damage(crit, adv)`, mínimo 0. `adv` es la ventaja con la que se atacó (Ataque Furtivo). Se aplica con `enemy.damage`.
4. Si el héroe ve la pelea, el daño y el log esperan a la animación del dado. Con la opción de desarrollador,
   el log muestra el cálculo, por ejemplo:
   `Warrior: d20 14+5=19 vs CA 12, 1d6(4)+3=7` o `marsupial rat: d20 3+4=7 vs CA 13, falla`.

Ya no existe la reducción de daño (DR) del original: en 5e todo se resuelve con la CA.

Cada tirada que el héroe ve deja un evento en `dungeon.roll_event` (d20 y daño) para la animación, y el
resultado queda en `dungeon.pending` hasta que la animación lo muestra (ver `docs/game/dungeon.md`).

## A sobreescribir
| Método | Por defecto | Héroe / mob |
|---|---|---|
| `armor_class()` | 10 | armadura + DES / conversión de PD |
| `attack_bonus(objetivo)` | 0 | mod + competencia + nivel del arma / conversión de PD |
| `roll_damage(crit, adv)` -> (daño, texto, evento) | 1 | dados del arma + mod / rango del original |
| `is_unaware()` | False | el mob que no te vio (dormido o sorprendido) |
| `advantage(objetivo)` | ventaja si el objetivo `is_unaware()` | el héroe suma desventajas |
| `speed()`, `stealth()` | 1, 0 | |

## Movimiento
`move(step)`: cierra la puerta que deja y abre la que pisa. No gasta tiempo: lo gasta quien llama.

## Utilidades
`distance(a, b)` (Chebyshev, diagonal = 1) y `adjacent(a, b)`.

## Estados (`game/buffs.py`)
- `buffs`: lista de `Buff`. `buff(id)`, `add_buff(id, duration, level, dc)` (si ya lo tiene, se queda con lo mayor),
  `has_flag(flag)`, `buff_speed()` (producto de los `speed`; `speed()` ya lo aplica).
- Contra un objetivo con `paralysed` se ataca con ventaja.

## Tiradas de salvación (5e)
- `saving_throw(car, cd)` -> `(superada, detalle)`: d20 + `save_bonus(car)` >= CD. Sin éxito/fallo automático
  con 20 o 1 (como en 5e). Paralizado falla solo las de FUE y DES. Deja un evento de dado si se ve.
- `save_bonus(car)`: 0 por defecto; héroe = mod + competencia si la clase la tiene; mob = ver `docs/game/mobs.md`.
- `move` llama a `dungeon.press` (trampas) salvo los voladores.

## Texto flotante
- `damage` muestra el número de daño (`dungeon.status`); `add_buff` muestra el nombre del estado.
- `saving_throw` manda al dado el total y la CD (`"CON CD11"`, de `1 + bono` a `20 + bono`) con el resultado.

## Animación de ataque
`attack` emite `("anim", atacante, "attack", celda del objetivo)` en `d.fx` (el render gira el sprite y lo anima).

## roll_damage
`roll_damage(crit, adv, target)`: `attack` pasa el objetivo (lo usa la Marca del cazador).

## Puertas y vista
`move`: si al salir o entrar se abre o cierra una puerta, recalcula la vista de los héroes del piso (`Dungeon.observe`), como
`Door.enter` / `Door.leave` del original (`Dungeon.observe()`). Antes, si un mob abría una puerta (y moría ahí) la vista
del héroe seguía calculada con la puerta cerrada: no se veía lo de atrás ni los mobs que venían, hasta caminar.

`attack`: un ataque "visto" (`Dungeon.sees`: piso del héroe local) espera al dado; el resto se aplica en el acto.

`is_downed()`: False salvo en héroes caídos. Atacar a un caído da ventaja, y si el atacante está al lado el golpe es
crítico (5e).

`saving_throw(característica, cd, why="")`: `why` va en el recuadro del dado ("CON CD11 trampa", "CON CD11 paralisis"),
para que no se confunda con la salvación contra la muerte.

## Turno de combate 5e (0.9)
5e manda sobre PD: en combate no cuentan las demoras de PD (arma lenta, comer 3 turnos, velocidad del cangrejo).
- `start_turn()`: `moves = speed_ft() // 5` casillas + `has_action`. `speed_ft()` = `base_speed_ft` (30) × estados.
- `spend()` en combate: si el último `move()` marcó `stepped`, gasta 1 casilla; si no, gasta la acción y el turno
  termina. Fuera de combate, tiempo de PD como siempre.
- `can_step()`: sin casillas, marca `no_moves` (fin del turno) y `get_closer` devuelve True sin moverse, así la acción
  en curso (caminar, ir a atacar) sigue el turno siguiente. Con 0 casillas igual se puede atacar al lado.
- `roll_initiative()`: d20 + `initiative_bonus()`; el dado sólo se muestra para héroes. Guarda `self.init`.
- ponytail: sin Correr, Destrabarse, Esquivar ni ataques de oportunidad; moverse después de la acción no se puede
  (la acción cierra el turno). Agregar si el combate lo pide.
