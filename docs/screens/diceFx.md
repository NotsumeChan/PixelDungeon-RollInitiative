# screens/diceFx.py

Animación de dados (pedido del dueño): cada tirada que el héroe ve aparece en un recuadro (tamaño y lugar según la tirada, ver "Tamaños").

- `Feed(dungeon.dice_events)`: toma los eventos `(etiqueta, valor, mínimo, máximo)` que deja `Char.attack`
  (el d20 del ataque y los dados de daño) y los encola.
- Por tirada: `ROLL_TIME` (0,35 s) mostrando números al azar entre mínimo y máximo, y luego `SHOW_TIME` (1,2 s)
  el resultado real con su etiqueta (`d20`, `1d8`, `2-5`...). Si hay otra tirada en cola, el resultado se ve
  al menos `MIN_SHOW` (0,7 s; antes 0,15 y no se alcanzaba a leer) y pasa a la siguiente.
- `Busy()`: True mientras falte mostrar algún resultado. La pantalla de juego no aplica el daño (ni sigue con
  los turnos) hasta que `Busy()` es False: primero se ve el dado, después llega el daño.
- **Mínimo** (el 1 del d20): borde rojo + vibración. **Máximo** (20, o el máximo de los dados de daño): borde
  dorado + chispas + vibración. Si mínimo y máximo coinciden, no hay efecto.
- Con 5 o más tiradas en cola todo va al doble de rápido, para no atrasarse.
- Todo usa `dt` (igual a cualquier FPS).
- El azar de la animación es **sólo visual** (`random` global); el resultado ya lo decidió `game/` con su rng.

## Perillas
`BOX_W/BOX_H` (tamaño base), `BOX_MARGIN` (posición desde abajo a la derecha), `SMALL` / `D20` / `DECIDE`
(escala y tiempos de cada tamaño), `DECIDE_RISE`,
`ROLL_TIME`, `SHOW_TIME`, `TICK_TIME`, `SHAKE_TIME`, `SHAKE_PX`, `FAST_QUEUE`, colores.

## Salvaciones
- Cada evento es `(etiqueta, valor, mínimo, máximo, ok)`. `ok` = `None` en ataques y daño; en salvaciones, el borde
  queda verde (superada) o rojo (fallada). La vibración y las chispas siguen sólo para el mínimo y el máximo.
- El recuadro se ensancha si la etiqueta no entra (`"CON CD11"`), alineado a la derecha.

## Tamaños (pedido del dueño)
`Tier(evento)` elige escala y tiempos; el recuadro se escala por un entero (nítido):
- **Decisiva** (`DECIDE`, x3): toda tirada con resultado (`ok` no es None: salvaciones de trampas y estados, contra la
  muerte, Medicina). Centrada, `DECIDE_RISE` por encima del centro para no tapar al héroe. Gira 0,7 s (suspenso) y el
  resultado queda 1,8 s (1,2 s con cola). Superada: borde verde, vibración y el doble de chispas. Fallada: borde rojo y vibración.
- **d20** (`D20`, x2): el d20 de ataque (rango de 20 valores), abajo a la derecha.
- **Chica** (`SMALL`, x1): daño y el resto, abajo a la derecha.
- Las chispas siguen al último recuadro (`anchor`) y crecen con su escala.
