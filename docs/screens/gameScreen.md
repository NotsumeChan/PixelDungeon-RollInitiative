# screens/gameScreen.py

Pantalla de juego: dibuja la partida (`Globals["dungeon"]`) y convierte el input en acciones del héroe.
La lógica está en `game/`; esta pantalla no decide nada de reglas.

## Controles
| Entrada | Acción |
|---|---|
| Click en una celda | moverse ahí / atacar al mob / ir a un ítem y recogerlo / ir a la salida y bajar (al soltar) |
| Arrastrar el mapa | mover la cámara para inspeccionar; vuelve al héroe cuando camina |
| 1 / 2, click derecho en acceso rápido | usar / elegir el ítem del acceso rápido |
| WASD / flechas | un paso (dos teclas = diagonal); mantenidas, sigue caminando; contra un mob, ataca |
| I / botón mochila | abrir o cerrar la mochila (click en un ítem = ventana de ítem) |
| Espacio | esperar un turno |
| Q | buscar secretos alrededor (2 turnos) |
| F | Segundo Aliento (Guerrero): no gasta turno |
| Esc / botón de menú | menú de la partida (o cerrar la ventana abierta) |
| Click en una escalera | bajar / subir de piso (pisarla con teclas no cambia de piso) |
| 1–4 | partida nueva con Warrior / Mage / Rogue / Huntress (mismas características) |
| R | partida nueva con el mismo personaje (`Globals["character"]`) |
| F11 | pantalla completa |
| Esc | cerrar la ventana abierta; si no hay, volver al título (la partida sigue en `Globals`) |

## Loop por frame
1. Eventos: teclas y click -> `hero.handle(celda)` o `hero.action`.
2. Si el héroe está libre y hay una tecla de movimiento mantenida, `handle` del vecino.
3. `AnimateSprites`: cada sprite viaja a su celda a 1 casilla cada `MOVE_TIME` (usa `dt`). Cuando el sprite del
   héroe llega, se baja `hero.animating`.
4. `dungeon.process()`: corren los turnos hasta que el héroe espere. Los mobs se mueven en la lógica al instante
   y sus sprites los siguen animados.
5. Piso nuevo: se reinician los sprites para que no se deslicen por el mapa. Si `visible` cambió, se rehace la niebla.
6. Dibujo: agua, tiles, personajes (mobs sólo si se ven), niebla, HUD.

## Render (de `DungeonTilemap.java`, `GameScene.java`, `FogOfWar.java`)
- Tiles: `tiles0.png`, índice = valor de terreno. Agua: `water0.png` debajo, baja 5 px/s.
- Personajes animados con `charSprite` (ver `docs/screens/charSprite.md`), centrados en X y apoyados abajo en la celda.
  `CharImg` = primer frame (íconos y medidas). Los cadáveres de los mobs se dibujan antes que los personajes.
- Niebla: 33x33 colores por esquina (VISIBLE / VISITED / MAPPED / negro) interpolados tile por tile.
  Se arma tile por tile (ver "Niebla de guerra" abajo).
- Ítems en el suelo: ícono del de arriba (o cofre / tumba / esqueleto) donde ya se vio; la niebla los oscurece.
- UI: `screens/gameUi.py` (barra de estado, herramientas, mochila, ventana de ítem). El click va primero a la UI
  y, si no toca nada, al mapa. Arriba de la barra de herramientas, las últimas 4 líneas del log.

## Pendiente
- Animaciones de sprites (idle, caminar, atacar, morir), números de daño flotantes y barras de vida.
- Pre-renderizar el mapa en vez de dibujar 1024 tiles por frame, si pesa.
- Pantalla de selección de personaje (hoy, teclas 1–4). Ver `roadmap/futuro-mods-personajes-red.md`.

## Dados y partidas por personaje (0.6.0)
- `DiceFx` (`screens/diceFx.py`) consume `dungeon.dice_events` y se dibuja encima de todo.
- `NewGame()` usa `Globals["character"]` (con nombre) y guarda la partida en `Globals["runs"][id]`.

## Cambios 0.6.1
- El click en el mapa se decide al **soltar**: si el mouse se movió más de `DRAG_MIN_PX`, fue un arrastre de
  cámara (`pan`) y no se mueve al héroe.
- Antes de `d.process()`: si hay ataques en `d.pending` y `DiceFx.Busy()` es False, `d.resolve_pending()`.
  Así el daño llega después de que el dado muestra el resultado.
- `d.verbose_rolls = DevDiceLog` (opción de desarrollador).

## Íconos sobre los mobs (`EmoIcon.java`)
- Zzz (`icons.png (13, 45, 9, 8)`) sobre los mobs dormidos y ! (`(22, 45, 8, 8)`) durante `ALERT_TIME` (1.5 s)
  cuando uno se despierta o pasa a cazarte. Laten entre 1 y 1.2 / 1.3 de escala (`Pulse`), con dt.
- `UpdateEmos` detecta los cambios de estado comparando con el frame anterior; se reinicia al cambiar de piso.
- Esc: cierra la ventana abierta o abre el menú de la partida (antes salía directo al título).

## Examinar
`Examine(d, state, celda)`, como `Toolbar.informer`: no explorada = "No sabes qué hay ahí"; el héroe = ficha;
mob visible = descripción, estado de la IA, CA, PV y estados con barra de vida; montón = el ítem de arriba (o cofre,
tumba, restos); si no, el terreno (`terrain.tile_info`) con su tile como ícono.

## Efectos (`UpdateFx` / `DrawChecked` / `DrawFloats`)
- Consume `d.fx`. **Texto flotante** (`FloatingText.java`): sube 16 px y se desvanece en 1 s; varios sobre el mismo
  personaje se apilan cada 8 px. Colores de `CharSprite` (blanco, verde, rojo, naranja, amarillo).
- **CheckedCell**: cuadrado azul que se achica y se desvanece en 0.8 s, debajo de los personajes.
- **Hojas** del pasto pisado: 4 píxeles verdes que saltan y caen (1 s).
- Todo se reinicia al cambiar de piso. El azar es sólo visual (`random` global).

## Niebla de guerra (`BuildFog` / `FogTile`)
- Como `FogOfWar.java`: un color por esquina de celda (visible / visitada / mapeada si las 4 celdas que la tocan lo
  son; el borde del mapa queda negro), interpolado.
- Se arma tile por tile: `FogTile(4 colores)` estira un 2x2 a 16x16 (cacheado, hay pocas combinaciones) y se pega con
  `BLEND_RGBA_MAX` sobre un surface transparente. Se dibuja en `(ox, oy)`, sin desplazamiento.
- Trampa: antes se estiraba el mapa entero de esquinas con `smoothscale`. pygame reparte los píxeles con `(n-1)/m`
  (no `1/16` como el LINEAR de OpenGL) y la niebla se corría hasta medio tile hacia los bordes: se veía detrás de
  las paredes simples. No volver a escalar el mapa entero.

## Efectos de trampas y estados
`UpdateFx` pasa `("trap", ...)` y `("appear", mob)` a `particles.Effects`, hace echar llamas a los personajes con un
estado `fx: "flame"` y actualiza los efectos. Se dibujan después de los personajes y antes de la niebla. Un mob recién
invocado se dibuja semitransparente mientras aparece.

## Conjuros
- F: Segundo Aliento (Guerrero) o el libro de conjuros (lanzadores).
- Con `state.selecting = ("spell", id)`, el click en el mapa pone `hero.action = ("cast", id, celda)`.
- `("bolt", desde, hasta, fx)` en `d.fx` -> `particles.Effects.Bolt`.

## Lanzar y disparar
- `state.selecting = ("throw", ítem)`: el click en el mapa pone `hero.action = ("throw", ítem, celda)`.
- `("missile", desde, hasta, imagen, giro)` en `d.fx` -> `particles.Effects.Missile` (si se ve alguna punta).

## Guardado
`SaveRun()` guarda la partida del personaje (`game/save.py`): al ir al menú, al cerrar (`Screen.OnQuit`) y al cambiar de
piso. Sin personaje (partida de prueba) no guarda.

## Zoom de cámara
- Rueda del mouse o pellizco (`MULTIGESTURE`, acumulado hasta `PINCH_STEP`) cambian entre `ZOOMS` = 0.5 / 1 / 2.
  Se guarda en `Local.config` (`[general] camera_zoom`, `core.Config.CameraZoom`).
- Con zoom distinto de 1, el mundo (nivel, personajes, efectos, niebla, textos flotantes) se dibuja en una superficie
  del tamaño de la vista (`WorldSurface`) y se escala al canvas (alejar: `smoothscale`, para no perder líneas finas;
  acercar: `scale`, píxeles nítidos). La UI va encima sin zoom.
- El click en el mapa y el arrastre convierten con el zoom (el pan está en píxeles del mundo).
- Trampa: con 0.5 los textos flotantes del mundo se ven chicos (se escalan con el mundo).

## Zoom nítido (corrige el pixelado al alejar)
Antes el mundo se achicaba dentro del canvas chico (320x180) y después OpenGL lo agrandaba: al alejar se perdían
píxeles. Ahora, como `Camera.zoom` del original, el zoom es un número **entero** de píxeles de pantalla por píxel del
mundo (`ZOOMS` = escalas zoom base −2 .. +2; en HD 2..6):
1. El mundo se dibuja en `WorldSurface` (lo que se ve, en píxeles del juego).
2. Se escala con ese factor entero a `GameScreen.HiRes()` (resolución completa, w×zoom base).
3. La UI se dibuja en `UiSurface` (transparente, 320x180) y se escala ×zoom base encima con un blit normal con alfa.
   Trampa: NO usar `BLEND_PREMULTIPLIED`. Sobre una capa transparente pygame copia el píxel tal cual (no premultiplica)
   y los píxeles invisibles de textos y sprites suelen ser blancos con alfa 0: con ese modo se suman y cada
   recuadro de UI se vuelve un rectángulo blanco.
`Screen.Update` sube la HiRes en vez del canvas.

## Escala de la interfaz
La capa de UI mide resolución completa / `UiScale()` (entero) y se escala ×`UiScale()`: con 75% la UI es más chica y
hay más lugar. `UiPos` pasa el mouse del canvas virtual a la capa de UI (clicks y tooltips); el mapa sigue usando el
canvas virtual. `DrawLog` usa el tamaño de la capa.

## Varios héroes (0.9)
- Se dibuja el piso del héroe local (`d.hero`). `hero.animating` se baja para todos los héroes al llegar su sprite; los
  de otros pisos no se dibujan, así que no esperan animación.
- Fin de partida ("Has muerto") cuando no queda ningún héroe vivo.
- **Modo de prueba** (`[dev] hotseat = true` en `Local.config`): `NewGame` suma un segundo héroe igual al personaje
  ("<nombre> 2"). Tab cambia de héroe. Si el piso espera la acción del otro héroe (`Dungeon.blocking`), o el local
  muere, la UI pasa sola a ese héroe (`AutoSwitch`) y el log avisa "Turno de ...". Con los héroes en pisos distintos,
  hay que cambiar con Tab (cada piso espera a su héroe).
- Héroe local caído: aviso "Inconsciente" con éxitos y fallos (o "estable"). En el modo de prueba la UI pasa sola al
  héroe que sigue en pie.

## Multijugador (0.9)
- `DrawNames`: con 2+ héroes, el nombre sobre cada héroe del piso que se ve (tuyo amarillo, otros blanco, caídos gris).
  Va en la capa del mundo, así que escala con el zoom. ponytail: los números flotantes nacen a la misma altura.
- Centrar: tecla C o el botón de `gameUi` (`state.center`) devuelven la cámara a tu héroe (`pan = [0, 0]`). Sólo con
  2+ héroes.
