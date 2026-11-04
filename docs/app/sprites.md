# app/src/sprites.rs

`screens/charSprite.py` + los efectos de `screens/gameScreen.py`: los personajes del piso local en pantalla (`Chars`).

- `animate`: cada sprite recorre, casilla por casilla, las celdas por las que pasó su personaje (`Char::trail`; en
  combate un mob hace varios pasos de una vez y en línea recta cortaría las esquinas de las paredes), a 1 casilla cada
  `MOVE_TIME` = 0,1 s, por dt; uno que no se ve aparece en su celda. Cada héroe del piso se suelta (`animating = false`) cuando su sprite llega: recién ahí sigue el planificador.
- Animaciones (`Anim`): las del héroe son `HERO_ANIMS` (HeroSprite, iguales en todas las clases); las de un mob, su
  `"anims"` en `data/mobs.json` (sin `"anims"`, el primer frame quieto). `update` elige: caído o muerto = morir (queda
  en el último frame), un ataque / operar en curso se termina, si el sprite viaja = correr (mirando hacia donde va),
  si no = quieto. `Fx::Anim` gira al personaje hacia la celda y reproduce la animación.
- Frames: el héroe usa la fila de su armadura (`tier` de `data/items.json`, la ropa es la 1) y el frame en x; un mob,
  los frames de izquierda a derecha bajando de fila (TextureFilm). `flip` = espejado en x (se invierten las u).
- `consume` lee `Dungeon::fx`: cadáveres (`Fx::Die`: animación de muerte y se desvanece en 3 s), textos flotantes
  (`Fx::Status`: suben 16 px en 1 s y se apagan; varios del mismo personaje se apilan), casillas revisadas (cuadrado
  azul que se achica en 0,8 s) y hojas del pasto (4 pixeles que saltan y caen).
- `draw_world` va debajo de la niebla (casillas revisadas, cadáveres, personajes); `draw_over` encima (hojas, textos
  y el nombre de los personajes sin hoja, en la capa del mundo).
- El azar es sólo visual: un `Rng` propio, no el de la partida.

- `portrait`: el primer frame achicado para entrar en un cuadrado (carrusel de iniciativa, examinar).
- Zzz sobre los mobs dormidos y ! durante 1,5 s cuando uno se despierta o pasa a cazar (CharSprite.showSleep /
  showAlert), con el pulso de EmoIcon; en `draw_over`.
