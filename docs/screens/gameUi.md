# screens/gameUi.py

UI de la partida con los assets originales: `StatusPane`, `Toolbar`, `WndBag` y `WndItem` del Java.

## Modelo: UI inmediata
- Cada frame `DrawUi` limpia `Hot` y vuelve a dibujar todo. Cada elemento clickeable se registra con
  `Hotspot(rect, callback)`.
- `Click(pos)` recorre `Hot` de lo último dibujado a lo primero: lo de arriba gana. Si nadie lo toma, la pantalla
  lo trata como click en el mapa.
- Las ventanas registran primero un hotspot de pantalla completa (click afuera = cerrar) y después el suyo
  (click en el fondo = nada).
- `UiState.window`: `None`, `("bag", None)`, `("bag", (pergamino, filtro))` para elegir objetivo, o `("item", item)`.
- La UI **no cambia reglas**: sólo pone acciones en el héroe (`hero.use(...)`, `hero.action = ...`). La lógica
  las ejecuta en su turno. Así las acciones son lo único que tendría que viajar por red en el P2P.

## Piezas
| Pieza | Assets | Detalle |
|---|---|---|
| Barra de estado | `status_pane.png` (NinePatch 80/48), `hp_bar.png`, `exp_bar.png` | avatar, vida en (30,3), experiencia arriba, nivel, piso, hambre (texto; en el original son íconos), botón de menú |
| Barra de herramientas | `toolbar.png` | esperar, buscar, examinar (pendiente), mochila y 2 accesos rápidos |
| Mochila | `chrome.png` (ventana), `items.png` | 7 columnas. 1ª fila: arma, armadura, 2 anillos (vacíos) y, separado a la derecha, el oro. Debajo, con un espacio, exactamente las 19 ranuras de mochila |
| Ficha (click en el avatar) | `chrome.png` | clase 5e, nivel, características y modificadores, PV, CA, competencia, ataque, daño, dado de golpe, XP, oro y rasgos de clase (Segundo Aliento disponible/usado, dados de Ataque Furtivo) |
| Ventana de ítem (170 px de ancho) | `chrome.png` (ventana y botones) | ícono, nombre, descripción y botones EQUIPAR/QUITAR, COMER, BEBER, LEER, SOLTAR |

`DrawButton(canvas, rect, texto, fn, enabled=True)`: con `enabled=False` el botón se ve gris y no registra click.

`NinePatch(src, region, margen, w, h)` estira una región manteniendo las esquinas y guarda el resultado en cache.

## Pendiente
- Examinar (info de una celda) y ranura rápida.
- Íconos de buffs (`buffs.png`) en lugar del texto de hambre.
- Brillo al presionar los botones de la barra.

- Al elegir el objetivo de un pergamino, el propio pergamino y los de su mismo tipo no se pueden elegir
  (identify no se identifica a sí mismo). `Hero._use` lo valida igual.

## Acceso rápido (0.6.1)
- 2 ranuras a la derecha de la barra (`hero.quickslots`). Muestran el ícono y la cantidad del ítem.
- **Click** (o teclas 1 / 2): usa el ítem con su acción principal (la primera de `Verbs`: beber, leer, comer,
  equipar o quitar). Si la ranura está vacía o el ítem ya no está, abre la mochila para elegir.
- **Click derecho:** abre la mochila en modo "Acceso rápido N: elige un objeto"; el elegido queda en esa ranura.
  Los apilables se guardan por id (siguen funcionando al juntar más); el equipo, por el objeto.
- `Hotspot(rect, fn, alt)` / `Click(pos, alt)`: `alt` es la acción del click derecho.
- La mochila en modo selección usa `select = ("target", pergamino, filtro)` o `("quick", i)`.

## Menú, indicador de ocupado y estados
- **Menú de la partida** (`DrawMenuWindow`, `WndGame.java`): botón de menú (arriba a la derecha) o Esc. Botones:
  Nueva partida (sólo si el héroe murió), Menú principal, Salir, Volver al juego. Click afuera cierra.
  Falta Configuración (configScreen es otra pantalla con su propio loop).
- **Indicador de ocupado** (`BusyIndicator.java`): `icons.png (8, 0, 8, 8)` girando 720°/s mientras el héroe no
  espera input (acción en curso, animación, dado pendiente o paralizado). El original lo pone en (1, 1); aquí va
  debajo del botón de menú, como pidió el dueño.
- **Íconos de estado** (`BuffIndicator.java`): en (32, 11), cada 9 px; hambre (5), famélico (6) y el `icon` de cada
  `Buff` del héroe. `BuffIcon(i)`, `IconImg(recorte)`.
- La ficha muestra las salvaciones y los estados activos.

## Examinar (`Toolbar.informer`)
- Botón de info: activa `ui.selecting = "examine"` y muestra el aviso "Toca una casilla para examinarla"
  (`DrawPrompt`). Otro click en el botón o Esc lo cancela. El click en el mapa lo resuelve `gameScreen.Examine`.
- `DrawInfoWindow`: ventana `("info", ícono, título, texto, vida)`: ícono, título amarillo, barra de vida si es
  un mob (`HealthBar`) y el texto partido en líneas (respeta los saltos de línea). Click en cualquier lado cierra.

## Configuración (`DrawSettings`, `WndSettings.java`)
- Ventana reutilizable: se abre desde el ícono de preferencias del título y desde "Configuración" en el menú de la
  partida, **encima de la partida, sin salir de ella**.
- Opciones: idioma (`game/lang.py`), pantalla completa (igual que F11) y FPS (30 / 60 / 120 / sin límite). Cada cambio
  se aplica al momento y se guarda en `Local.config` (`core.Config.Save`).
- `core.Screen` y `core.Config` se toman con `importlib` porque `core.Screen`, como atributo del paquete, es la clase.
- Los textos de los menús de esta ventana y del menú de la partida usan `t()`.

## Libro de conjuros (`DrawSpellsWindow`)
- Tecla F con un lanzador (el Guerrero sigue usándola para Segundo Aliento). Muestra bono, CD, espacios (`nv1 2/2`),
  usos gratis y cada conjuro con su nivel y descripción; en gris los que no se pueden lanzar ahora.
- Click: si es sobre uno mismo, se lanza; si no, `ui.selecting = ("spell", id)` y aviso "Elige el objetivo" (Esc cancela).
- La ficha muestra "Conjuros (F): ataque +X, CD Y".

## Lanzar
- Verbo LANZAR en las arrojadizas (primero, así el acceso rápido lanza). Pone `ui.selecting = ("throw", ítem)` y el
  aviso "Elige dónde lanzar"; el click en el mapa lo resuelve `gameScreen`.
- `ItemIcon(índice)`: entero = `items.png`; `(archivo, índice)` = otra hoja (propia o de un mod), celdas de 16x16.

## Tooltips y accesos rápidos
- `Hotspot(rect, fn, alt, tip)`: `tip` = texto al pasar el mouse. `DrawTooltip` muestra el del botón de más arriba bajo
  el mouse (`ui.mouse`, lo pone la pantalla), si no hay una ventana abierta. Lo tienen la barra de herramientas
  (con su tecla), los accesos rápidos (ítem y tecla), el avatar y el menú.
- `QUICK_X`: el acceso rápido 1 está a la izquierda y el 2 a la derecha (antes estaban al revés que las teclas).

## Escala de la interfaz
Opción de la ventana de configuración (`UI_SCALES`: del zoom base hacia abajo, 50% / 75% / 100% en HD), guardada en
`Local.config` (`ui_scale`, `core.Config.UiScale`). Afecta a la interfaz de la partida. No hay más de 100%: la UI
está pensada para 320x180 y ventanas como la mochila no entrarían.

## Reglas de la partida
`DrawRulesWindow` (ventana `("rules",)`, botón en el menú de la partida): una fila por regla de `RULE_ROWS`
(clave, texto, opciones; la primera es la de 5e) y la opción elegida con borde amarillo. Cambia `dungeon.rules` al
momento; "Volver" regresa al menú.

## Combate 5e (0.9)
`DrawInitiative` (desde `DrawStatus`): carrusel arriba a la derecha, bajo piso/menu (`CAROUSEL_Y`). Retratos
(primer frame, `charSprite.Frame`) en orden de iniciativa empezando por al que le toca (borde amarillo); fondo azul =
héroe, rojo = mob; número = iniciativa; tooltip con nombre. Lo que el héroe local no ve sale como "?" (no revela
mobs). Al pasar el turno se desliza una casilla (`CAROUSEL_SPEED`, con tiempo real). Máximo `CAROUSEL_MAX` retratos.
Debajo: "Ronda N", y en tu turno los pies que quedan y si queda la acción. El indicador de ocupado gira mientras no
es tu turno.

`UiState.center` + botón `CENTER_BTN` (bajo el avatar, con una mira): sólo con 2+ héroes; lo aplica `gameScreen`.

`TextIcon` / `ItemIcon`: un ítem sin sprite, o cuya hoja no carga, se dibuja con su nombre (una palabra por línea,
achicado a 16x16). Vale para inventario, montones en el piso, ventanas y proyectiles.
