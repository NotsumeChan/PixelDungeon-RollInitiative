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
- `item_icon`: recorte de items.png o, sin sprite, el nombre a tamaño real (nítido): hasta 2 palabras recortadas para entrar en 16x16. `draw_heaps`: los montones del piso
  donde ya se vio (bajo la niebla).
- Ícono de hambre (buffs.png) en el panel de estado.

R5b: íconos de los estados del héroe en el panel (con su nombre de tooltip); "Estados:" en la ficha y al examinar un mob.

Configuración (WndSettings, desde el menú): FPS, pantalla completa (F11) y controles (se toca la acción y se
presiona la tecla nueva; Esc cancela). La mochila y la ventana del ítem tienen el marco más tenue (`BAG_TINT`) y las
ranuras más oscuras (pedido del dueño: estaban muy blancas).

R5c: libro de conjuros (F o desde la ficha): bono, CD, espacios y usos gratis; un botón por conjuro (gris si no se puede) que pide el objetivo en el mapa o se lanza sobre uno mismo. La ficha muestra el rasgo de clase con su botón (Segundo Aliento o Conjuros), para el teléfono.

Pantalla de título (`draw_title`, TitleScene.java): arcos de fondo que se desplazan (`arcs1.png` / `arcs2.png`, el de
adelante al doble), el banner con los letreros que laten, el dashboard (Jugar = elegir la clase; Acerca de), la
la versión abajo a la derecha. Sin Mods (1.x). El menú de la partida suma "Menú
principal" (vuelve al título; la partida se pierde hasta que haya guardado).

Acceso rápido: usa la acción principal del ítem; si sólo se puede soltar (las arrojadizas hasta R5d), abre la ventana
del ítem en vez de soltarlo.

Menús como el prototipo (2026-11-04):
- Título: Jugar, Mods y Acerca de (dashboard.png 0 / 2 / 1); Mods y Acerca de son pantallas de texto centrado sobre
  negro (`Window::Page`), click o Esc vuelve.
- Menú de pausa (WndGame): Nueva partida (si moriste), Configuración, Reglas de la partida, Menú principal | Salir,
  Volver al juego.
- Reglas de la partida (`Window::Rules`): `dot_death_saves` y `save_half`, se aplican al momento (la primera opción es
  la de 5e).
- Configuración: Idioma (sólo Español; English deshabilitado hasta la traducción), Pantalla completa, FPS y Escala de
  la interfaz (del zoom que entra hacia abajo, entero: 100% / 75% / 50% en HD); Controles en su propia ventana.

Lista de personajes (`Window::Roster`, rosterScreen.py): filas con avatar, nombre, estado ("en partida - piso N,
nivel M" o la clase y su dado de golpe) y características; JUGAR / NUEVO / BORRAR (confirma). NUEVO abre la creación.

Creación de personaje (`Window::Create`, `draw_create`, createScreen.py con su mismo layout en 320x180): título y
nombre arriba, las 4 clases con avatar y "clase d<dado>", las 6 características (valor y modificador), las 7 tiradas
4d6 (gris las ya usadas; la elegida muestra sus 4 dados con el descartado entre corchetes), ayuda y TIRAR / LIMPIAR /
EMPEZAR ("FALTAN STATS" en gris hasta tener las 6). Click / toque en una tirada y luego en una característica la pone
(si estaba en otra, se mueve); click en una característica sin tirada elegida deja escribir el número (1-20); Enter /
Tab pasa a la siguiente. TIRAR de nuevo borra lo asignado desde las tiradas viejas; LIMPIAR borra las 6. Esc vuelve a
la lista. El estado es `Ui::create` (`Create`, `Cr`); en vertical las clases van en 2 columnas. Sin arrastrar (el
prototipo arrastraba tiradas y cambiaba características de lugar): click-click hace lo mismo y anda en táctil.

Título: Jugar, Mods, Acerca de y Opciones (la configuración, antes arriba a la izquierda; usa el ícono PREFS de
`icons.png` al doble hasta que haya uno de 32x32). Los botones se separan por el texto más largo + 8 px (para que no
se toquen los nombres); en vertical se angostan para que entren los 4.
Configuración: en escritorio suma "Resolucion (ventana)" (640x480 / 1280x720 / 1920x1080), el `resolution` de
Local.config.

R5d: LANZAR en la ventana del ítem (o el acceso rápido de una arrojadiza) deja `throwing` = el ítem: el aviso de
arriba pide la casilla ("Elige adonde lanzar", Esc cancela) y el próximo click en el mapa lanza.
