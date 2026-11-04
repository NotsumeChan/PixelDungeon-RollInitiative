# app/src/dice_fx.rs

`screens/diceFx.py`: cada tirada que el héroe local ve (`Dungeon::dice_events`) se muestra en un recuadro: números al
azar mientras gira y después el resultado.

- Tamaños: salvación (`ok` no es None) x3 en el centro, un poco arriba del héroe (gira 0,7 s y queda 1,8 s); d20 de
  ataque x2 y el resto (daño...) x1, abajo a la derecha (giran 0,35 s y quedan 1,2 s). Con otra tirada en cola, el
  resultado queda al menos 0,7 s (1,2 s las salvaciones); con 5 o más en cola todo va al doble.
- Mínimo = borde rojo y vibración; máximo = borde dorado, chispas y vibración. Salvación: verde superada, roja
  fallada, con chispas si se superó.
- `busy()`: falta mostrar algún resultado. La app no aplica lo pendiente (`resolve_pending`) hasta que el dado lo
  mostró: el daño cae cuando se ve el número.
- El recuadro se escala por un entero (nítido). El azar es sólo visual (rng propio).
