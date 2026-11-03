# game/traps.py

Las 8 trampas de las Alcantarillas (`levels/traps/*.java` + `Level.press` / `mobPress`) con **tiradas de
salvación de D&D 5e**. Los efectos aplican estados de `game/buffs.py` por ID.

## Activación (`press`)
- `Char.move` llama a `dungeon.press(celda, ch)` al pisar (salvo los voladores).
- Trampa oculta: sólo la activa el héroe ("Una placa oculta hace click"). Los mobs sólo activan trampas visibles
  (y el pathfinding las evita).
- La trampa queda inactiva (`INACTIVE_TRAP`) y el héroe se interrumpe.

## Salvaciones
- CD = `trap_dc(piso)` = `10 + piso // 2`: 11 en los pisos 2 y 3, 12 en el 4 (en el piso 1 no hay trampas).
- `_save`: la tirada se hace en el momento (el orden del rng no depende del render). Si el héroe la ve, el
  efecto espera en `dungeon.pending` a que la animación del dado muestre el resultado, igual que los ataques.
  El efecto no puede usar el rng.

| Trampa | Salvación | Falla | Supera |
|---|---|---|---|
| tóxica | CON, área 3x3 | envenenado 4 + piso | la mitad |
| fuego | DES, área 3x3 | en llamas | nada |
| paralizante | CON, área 3x3 | paralizado (repite CON cada turno) | nada |
| veneno (dardo) | CON | envenenado 4 + piso/2 | nada |
| alarma | - | todos los mobs van a la trampa (`Mob.beckon`) | |
| eléctrica | DES | 1/3 a 2/3 de la vida actual | mitad de daño |
| garras | DES | sangrando (piso + 3) y lisiado | nada |
| invocación | - | 1 a 3 mobs del piso alrededor, despiertos, actúan 2 turnos después | |

## Trampas de portado
- Los gases y el fuego no forman nubes que se expanden (`Blob`): afectan una vez al área 3x3. Marcado con `ponytail`.
- `Source("trampa electrica")` hace de origen del daño para el log y la muerte.

## Cómo se ve
- El recuadro del dado muestra `"CON CD11"` con el total; borde verde si se supera, rojo si se falla.
- Al resolverse, texto flotante sobre el personaje (`"CON supera"` / `"CON falla"`) y el número de daño o el estado.

## Efectos visuales
`press` emite `("trap", terreno, celda)` en `d.fx` y la invocación `("appear", mob)` por cada mob; los dibuja
`screens/particles.py`.

## Varios héroes (0.9)
- `press`: cualquier héroe (`ch in d.heroes`) activa trampas ocultas y se interrumpe.
- `_save`: la salvación espera al dado sólo si `Dungeon.sees` (piso del héroe local); si no, se aplica en el acto.
- `_save(..., half=True)` (gas tóxico, descarga): con la salvación superada el log dice "la mitad del efecto" o "sin
  efecto", según la regla de la partida `save_half`. El recuadro del dado dice "trampa".
