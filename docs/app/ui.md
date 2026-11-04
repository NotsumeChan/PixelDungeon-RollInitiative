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

R5a:
- Barra: mochila (I) y 2 accesos rápidos (1, 2): click = usar con su acción principal (vacío: elegir), click derecho =
  elegir. Muestran el ícono y la cantidad.
- Mochila (WndBag): fila de equipo (arma, armadura, 2 anillos vacíos), el oro aparte y 19 ranuras con cantidad y
  nivel conocido. Modos: ver (abre la ventana del ítem), elegir el objetivo de un pergamino (lo que no sirve sale
  tenue), elegir el acceso rápido.
- Ventana del ítem (WndItem): ícono, nombre, descripción con los números de 5e y un botón por acción.
- `item_icon`: recorte de items.png o, sin sprite, el nombre achicado a 16x16. `draw_heaps`: los montones del piso
  donde ya se vio (bajo la niebla).
- Ícono de hambre (buffs.png) en el panel de estado.

R5b: íconos de los estados del héroe en el panel (con su nombre de tooltip); "Estados:" en la ficha y al examinar un mob.

Configuración (WndSettings, desde el menú): FPS, pantalla completa (F11) y controles (se toca la acción y se
presiona la tecla nueva; Esc cancela). La mochila y la ventana del ítem tienen el marco más tenue (`BAG_TINT`) y las
ranuras más oscuras (pedido del dueño: estaban muy blancas).

R5c: libro de conjuros (F o desde la ficha): bono, CD, espacios y usos gratis; un botón por conjuro (gris si no se puede) que pide el objetivo en el mapa o se lanza sobre uno mismo. La ficha muestra el rasgo de clase con su botón (Segundo Aliento o Conjuros), para el teléfono.

Pantalla de título (`draw_title`, TitleScene.java): arcos de fondo que se desplazan (`arcs1.png` / `arcs2.png`, el de
adelante al doble), el banner con los letreros que laten, el dashboard (Jugar = elegir la clase; Acerca de), la
configuración arriba a la izquierda y la versión abajo a la derecha. Sin Mods (1.x). El menú de la partida suma "Menú
principal" (vuelve al título; la partida se pierde hasta que haya guardado).

Acceso rápido: usa la acción principal del ítem; si sólo se puede soltar (las arrojadizas hasta R5d), abre la ventana
del ítem en vez de soltarlo.
