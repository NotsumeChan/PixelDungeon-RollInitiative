# screens/particles.py

Efectos visuales del original: `effects/Speck.java`, `particles/FlameParticle`, `PoisonParticle`, `SparkParticle`,
`Lightning.java`, `Wound.java`, `WandOfBlink.appear`. Clase `Effects`, una por partida en la pantalla de juego.
Coordenadas en píxeles del mundo; todo avanza con `dt`; el azar es sólo visual.

## Trampas (`Trap(terreno, celda, nivel)`, desde `d.fx` `("trap", terreno, celda)` si la celda se ve)
| Trampa | Efecto |
|---|---|
| tóxica / paralizante | humo verde / amarillo (`Speck` TOXIC / PARALYSIS: crece, gira y se desvanece en 1-3 s) que brota cada 0.6 s durante 2 s en las celdas no sólidas del 3x3. No hay `Blob`: es sólo visual |
| fuego | 5 llamas (`FlameParticle`: suben acelerando y se achican en 0.6 s) |
| dardo | 3 gotas (`PoisonParticle.SPLASH`: violeta a verde, crecen y caen) |
| alarma | 3 gritos (`Speck.SCREAM`) cada 0.3 s que se expanden |
| eléctrica | rayo vertical y horizontal por la celda (`Lightning`, 0.3 s, el punto medio tiembla) + 3-4 chispas |
| garras | tajo (`Wound`, 0.8 s, se angosta y se desvanece) |
| invocación | cada mob aparece en 0.4 s (`Alpha`) con 3 destellos (`Speck.LIGHT`), desde `("appear", mob)` |

## Otros
- `Pour(clave, rect, dt)`: una llama cada 0.06 s dentro del sprite. Lo usa la pantalla para los estados con `fx: "flame"`
  (en llamas).
- Partículas `lightMode` del original (llamas, veneno, chispas): se suman con `BLEND_ADD`.
- Los humos sólo salen en celdas visibles (como `BlobEmitter`). `Clear()` al cambiar de piso.

## Proyectiles (`Bolt`)
`MagicMissile`: viaja a 200 px/s de la celda del lanzador a la del objetivo soltando una estela (llamas, frío celeste,
blanco, rosa para Dormir). `mark` no viaja: 3 destellos sobre el objetivo.

## Armas en vuelo (`Missile`)
`MissileSprite`: el ícono del ítem vuela a 240 px/s. Con giro 0 (dardos, jabalinas, flechas) vuela apuntando
(`135 - atan2(dx, dy)`: el sprite original mira al noreste); si no, gira (`spin` grados/s; shuriken y bumerán 1440).
