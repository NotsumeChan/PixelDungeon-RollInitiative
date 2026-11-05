# game/src/traps.rs

`game/traps.py` (levels/traps/*.java + Level.press / mobPress): las 8 trampas de las Alcantarillas con salvaciones de
5e (CD = 10 + piso / 2). Se activan al pisarlas (`Dungeon::press`); los mobs sólo activan las visibles; la trampa queda
inactiva y corta la acción del héroe.

| Trampa | Salvación | Efecto |
|---|---|---|
| Gas tóxico | CON, 3x3 | envenenado 4 + piso turnos; supera = la mitad (regla `save_half`) |
| Fuego | DES, 3x3 | en llamas |
| Gas paralizante | CON, 3x3 | paralizado (repite la salvación cada turno) |
| Dardo envenenado | CON | envenenado 4 + piso / 2 |
| Alarma | - | todos los mobs del piso van hacia la trampa |
| Eléctrica | DES | 1/3 a 2/3 de la vida actual; supera = la mitad |
| Garras | DES | sangrando (piso + 3) y lisiado |
| Invocación | - | 1 a 3 mobs del piso alrededor, despiertos |

La salvación se tira al pisar (el orden del rng no depende del render) y el efecto (`TrapEffect`) se aplica cuando el
dado se ve (`Pending::TrapSave`). Faltan las nubes que se expanden (Blob) y las partículas.
