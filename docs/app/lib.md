# app/src/lib.rs (+ main.rs)

Bucle del juego sobre winit 0.30 (`ApplicationHandler`), el mismo para escritorio (`main.rs`) y Android
(`android_main`, NativeActivity).

- `resumed`: crea ventana y `Gpu` (en Android, cada vez que la app vuelve del fondo) y marca la niebla para subirla.
- `ControlFlow::Poll` + `request_redraw` en `about_to_wait`: se dibuja sin parar.
- `frame`: dt en segundos (tope 0,1 s), FPS por segundo (también al log: `adb logcat | grep FPS`).

R2: **recorrer un piso** con un observador (el guerrero; lo reemplaza el héroe en R3), como la fase 2 del prototipo.
- Caminar: WASD / flechas (mantenidas: un paso cada vez que llega a su celda, diagonal con dos teclas) o click / toque
  en una celda (camina hacia ella en línea recta, sin pathfinder hasta R3). Las puertas se abren y cierran al pasar y
  la vista se recalcula en cada paso.
- Cámara: centrada en el sprite; arrastrar (mouse o un dedo) la desplaza; al caminar vuelve. Zoom con la rueda o
  pellizcando: factor entero de la UI -2 a +2 (siempre nítido).
- R = otra seed, N = siguiente piso (1-4). Seed nueva = microsegundos actuales.
- El sprite viaja a su celda a 1 casilla cada `MOVE_TIME` (0,1 s), por dt.
