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
- `item_icon`: recorte de items.png o, sin sprite, el nombre a tamaño real (nítido): hasta 2 palabras recortadas para entrar en 16x16. `draw_heaps`: los montones de la zona
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
la lista. El estado es `Ui::create` (`Create`, `Cr`); en vertical las clases van en 2 columnas.
Arrastrar (mouse o dedo, `Create::press` / `moved_to` / `release`, como el prototipo): una tirada a una característica
la pone, una característica a otra las cambia, una característica afuera la quita; lo arrastrado sigue al puntero y la
característica de abajo se resalta. Menos de 3 px entre apretar y soltar es un click.

Título: Jugar, Mods, Acerca de y Opciones (la configuración, antes arriba a la izquierda; usa el ícono PREFS de
`icons.png` al doble hasta que haya uno de 32x32). Los botones se separan por el texto más largo + 8 px (para que no
se toquen los nombres); en vertical se angostan para que entren los 4.
Configuración: en escritorio suma "Resolucion (ventana)" (640x480 / 1280x720 / 1920x1080), el `resolution` de
Local.config.

R5d: LANZAR en la ventana del ítem (o el acceso rápido de una arrojadiza) deja `throwing` = el ítem: el aviso de
arriba pide la casilla ("Elige adonde lanzar", Esc cancela) y el próximo click en el mapa lanza.

Configuración: Idioma = el del contenido (`game::registry::LANGS`: Español / English), se aplica al momento.

Red LAN: la lista de personajes suma CREAR LAN / UNIRSE (con el personaje elegido). `Window::Join` (`draw_join`): las
partidas encontradas ("nombre - N jugadores - IP"; gris si es de otra versión o contenido), el aviso de la red
(conectando, rechazado, se cortó) y VOLVER.

Tanda del 2026-11-04 (pedidos del dueño tras probar en el teléfono):
- Con 2+ héroes, fuera de combate: lista de los de la zona con ✓ / casilla vacía (Icons CHECKED / UNCHECKED) según ya
  eligieron qué hacer, y "Esperando a ..." cuando la partida espera a otro (en combate, al que le toca).
- El log muestra a cada uno sus avisos privados (`Dungeon::tell`) y los de todos.
- Barra: botón del rasgo de clase (ícono de la clase de Icons.java; gris si el Segundo Aliento ya se usó).
- Título: Salir arriba a la derecha (Icons.EXIT). Lista de personajes: VOLVER. Sala de espera (`Window::Lobby`).
- Configuración: "Mostrar" en la fila de FPS (el contador en la partida, `showFPS` en Local.config).
- Vertical (180 de ancho): la mochila usa 5 columnas, la ventana del ítem y la ficha no pasan del ancho (la ficha
  parte las líneas largas), la lista de personajes usa 3 líneas por fila, y en la creación el nombre va debajo del
  título. Creación en Android: teclado en pantalla al tocar el nombre o una característica.

Chat de la partida LAN (`Ui::chat`, `UiAction::Chat`): Enter o el botón "Chat" de la barra (sólo en red, `Ui::lan`)
abre la línea de arriba; Enter / LISTO manda, Esc cancela. En Android se escribe con el teclado del sistema
(`App::text_input`); debajo de la línea, LISTO y CANCELAR (`draw_text_buttons`), y tocar la línea lo vuelve a abrir.
La línea del título (`Ui::prompt`: dirección o clave de Unirse y la sala) funciona igual.

Diálogo con un NPC (`draw_dialog`): si el héroe local tiene un diálogo abierto (`Dungeon::dialog`), tapa el resto de
la UI: retrato, nombre, texto y las opciones como botones (`Action::Choose`). Esc / Atrás = `Action::Bye`.

Creación de personaje: NUEVO y TIRAR ya tiran los 4d6 y `Create::auto_assign` reparte las tiradas más altas según el
`ability_priority` de la clase (la app lo pide: `ability_order` en `lib.rs`); cambiar de clase las reacomoda si no se
tocó nada a mano. Los botones de abajo de la lista y de la creación miden `BOTTOM_BTN_H` (24) para el dedo.

Creación de personaje: clase y apariencia se eligen con dos selectores con flechas (`draw_picker`): entran todas las
clases de `data/classes.json` (también las de los mods) y todas las skins de `data/skins.json`.

Tienda (`draw_shop`): si el héroe local tiene una tienda abierta (`Dungeon::shop`), tapa el resto: lo que vende el NPC
y tu mochila en ranuras de 24, el ítem elegido (`Ui::shop_sel`) con su precio y COMPRAR / VENDER, y CERRAR (o Esc /
Atrás = `Action::Bye`).

Configuración por pestañas (0.9.29, `draw_settings`): **Gráficos** (pantalla completa, resolución, FPS, escala),
**Controles** (su ventana, `draw_controls`, con las mismas pestañas), **Actualizaciones** (sólo con `updater`: buscar
al abrir Sí / No, BUSCAR AHORA, el resultado y ACTUALIZAR si hay una nueva) y **Otros** (idioma). `Ui::settings_tab`;
en vertical, si los nombres no entran, las pestañas van de a dos. En Android no hay pestaña Controles ni pantalla
completa (0.9.33). El idioma es una lista vertical (un idioma por fila; con más de 5, flechas y `Ui::lang_scroll`).

Acción rápida y Seguir (0.9.38, `draw_quick`): arriba de la barra, a la derecha, sólo cuando el héroe local espera
input (`waiting`) y no hay ventana, chat ni diálogo. La acción rápida (`Dungeon::quick_action`) muestra el retrato del
enemigo (atacar), el ícono del ítem (recoger) o "Abrir" (cofre / tumba). Seguir (celeste, arriba a la derecha bajo el
lugar del carrusel de iniciativa, sólo fuera de combate): `resume` guarda la última caminata
pedida (`note_walk`, al tocar el mapa o con `UiAction::Act`); `resumable` la ofrece mientras no se haya llegado.
Arrojadizas: con `throwing` puesto, el aviso nombra al enemigo más cerca y el mismo acceso rápido la lanza ahí.
