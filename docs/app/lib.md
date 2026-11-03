# app/src/lib.rs (+ main.rs)

Bucle del juego sobre winit 0.30 (`ApplicationHandler`), el mismo para escritorio (`main.rs`) y Android
(`android_main`, NativeActivity).

- `resumed`: crea ventana y `Gpu`. En Android se llama cada vez que la app vuelve del fondo (la superficie se
  destruye en `suspended`), por eso se recrean ahí.
- `ControlFlow::Poll` + `request_redraw` en `about_to_wait`: se dibuja sin parar, tan rápido como presente la
  superficie.
- `frame`: dt en segundos (tope 0,1 s), FPS por segundo (también al log: `adb logcat | grep FPS`).
- R0: dibuja el FPS, el título y la versión con la fuente, y un guerrero que cruza la pantalla a `HERO_SPEED` px/s
  (prueba de que todo va por dt).
- `SHEETS`: las hojas del atlas, dentro del binario.
