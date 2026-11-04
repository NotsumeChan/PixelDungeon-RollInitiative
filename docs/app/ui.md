# app/src/ui.rs

`screens/gameUi.py` + el log y examinar de `screens/gameScreen.py`, con los assets originales (`chrome.png`,
`status_pane.png`, `toolbar.png`, `hp_bar.png`, `exp_bar.png`, `icons.png`). UI inmediata: cada frame se dibuja todo y
se registran zonas clickeables (`hot`, con su tooltip); `click` busca de arriba hacia abajo y devuelve un `UiAction`
que aplica la app. Un click que no toca la UI va al mapa.

- Panel de estado (StatusPane): el recorte del original estirado a lo ancho; avatar de la clase (abre la ficha), barra
  de vida, barra de experiencia arriba a lo ancho, nivel, piso, botón de menú e indicador de ocupado (mientras el
  héroe no espera input; titila en vez de girar porque el lote no rota sprites).
- Carrusel de iniciativa (combate 5e), arriba a la derecha: retratos por iniciativa desde el que le toca (borde
  amarillo), azul los héroes y rojo los mobs, "?" lo que no se ve; se desliza al pasar el turno. Debajo, la ronda y,
  en tu turno, los pies que te quedan y si te queda la acción.
- Barra de herramientas: esperar (Espacio), buscar (Q), examinar (el próximo click en el mapa examina esa casilla).
- Log: las últimas 4 líneas sobre la barra; en el centro, "Has muerto" o las salvaciones contra la muerte en curso.
- Ventanas (nine-patch de `chrome.png`): examinar (WndInfoCell / WndInfoMob: terreno con su tile, o mob con retrato,
  barra de vida, descripción, estado, CA y PV; tu propia casilla abre la ficha), ficha 5e (características,
  PV, CA, competencia, ataque, daño, dado de golpe, salvaciones, XP) y menú (nueva partida si moriste, salir,
  volver). Click afuera o Esc cierran.
- Tooltips: el texto de la zona bajo el mouse, encima de él y sin salirse de la pantalla (sólo con mouse).
- Faltan (R5): mochila, accesos rápidos, íconos de estados, configuración, reglas de la partida.
