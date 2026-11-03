# core/Screen.py

Al importarse crea la ventana OpenGL (`Window`) y el quad de pantalla completa. Cada `Screen` es una pantalla
con su propio canvas y su propia textura.

## Resolución virtual (Zoom)
- `Zoom = max(1, min(ancho // 320, alto // 180))`: Hd -> 4, Fhd -> 6, Basic -> 2.
- `Screen.canvas` mide `resolución / Zoom` (320x180 en Hd). Todo se dibuja en **pixeles del juego**, como en el
  original (`PixelScene`), y OpenGL lo estira a la ventana con filtro NEAREST, sin costo de CPU.
- `screen.w` / `screen.h` son el tamaño virtual, no el de la ventana.
- `MousePos()` devuelve el mouse en coordenadas virtuales. Usarlo siempre en lugar de `pg.mouse.get_pos()`.

## Loop
- `FrameRate(fps)` sólo guarda el límite. `Update()` sube el canvas, hace flip y `clock.tick(fps)`, y deja en
  `self.dt` los segundos del frame.
- Antes `tick` se llamaba una sola vez, fuera del loop, así que los FPS no se limitaban. Usar `screen.dt` en vez
  de `deltaT()`, que queda por compatibilidad con `configScreen`.

## EventHandler(event, botones)
Pasa el evento y el mouse virtual a cada botón. Con `QUIT` cierra todo (`sys.exit`).

## Pantalla completa y tamaño de ventana
- **F11** en cualquier pantalla alterna la pantalla completa "de escritorio": no cambia la resolución del monitor ni
  pierde el contexto OpenGL. Usa `pygame.Window` y, si no está, `toggle_fullscreen`.
- `Local.config` -> `fullscreen = true` arranca en pantalla completa.
- La ventana es redimensionable y se puede maximizar. El tamaño sale del último evento de redimensión
  (`WindowSize`), porque con OpenGL algunos sistemas no actualizan `get_window_size` al maximizar.
- En cada `Update` se calcula `Viewport`: llena la ventana manteniendo la proporción, con bandas negras si
  sobra. Con `integer_scale = true` en `Local.config` usa sólo escalas enteras (píxeles perfectos, más bandas).
- `MousePos()` descuenta las bandas y la escala reales (`self.vp`).

## FPS elegidos en la configuración
`Screen.UserFps` (de clase): si no es `None`, todas las pantallas lo usan en `Update` en vez de su `fps`
(0 = sin límite). Lo pone la ventana de configuración; al arrancar vale `None` y manda `maxFPS` de `Local.config`.

## dt
`dt` se mide con `perf_counter` entre frames (tope `MAX_DT` = 0.1 s). `clock.tick` sólo limita los FPS: devuelve
milisegundos **enteros** (33 en vez de 33.3; 0 sin límite) y usarlo como dt hacía que las animaciones cambiaran de
velocidad con el límite de FPS.

## OnQuit
`Screen.OnQuit`: función que se llama antes de cerrar la ventana (la pantalla de juego guarda la partida).

## HiRes
`HiRes()`: superficie de resolución completa (virtual × zoom base) con su textura; si existe, `Update` la sube en vez
del canvas. La usa la pantalla de juego para el zoom de cámara nítido. Costo: ~3.7 MB por frame en HD (en Android
convendrá componer en OpenGL con dos texturas).
