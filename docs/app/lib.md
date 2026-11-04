# app/src/lib.rs (+ main.rs)

Bucle del juego sobre winit 0.30 (`ApplicationHandler`), el mismo para escritorio (`main.rs`) y Android
(`android_main`, NativeActivity).

- `resumed`: crea ventana y `Gpu` (en Android, cada vez que la app vuelve del fondo) y marca la niebla para subirla.
- `ControlFlow::Poll` + `request_redraw` en `about_to_wait`: se dibuja sin parar.
- `frame`: dt en segundos (tope 0,1 s), FPS por segundo (también al log: `adb logcat | grep FPS`).

R3: **jugar un piso** con el héroe (`World`: la partida `game::dungeon::Dungeon` + dónde se dibuja cada personaje).
- Cada frame: los sprites viajan a su celda (1 casilla cada `MOVE_TIME`, por dt); cuando el del héroe local llega,
  se baja `animating` y el planificador sigue. Después las teclas mantenidas y `run()`: `process` + `resolve_pending`
  hasta que el héroe tenga que decidir (lo visto se resuelve en el acto hasta la animación de dados, R4).
- Input: click / toque = `Dungeon::handle` (caminar con pathfinder, atacar, ayudar a un caído, cambiar de piso en la
  escalera); tocar al propio héroe = buscar. WASD / flechas = un paso (diagonal si se puede, si no por un eje; atacar
  si hay alguien). Espacio = esperar, Q = buscar, R = otra partida, C = otra clase (sin creación de personaje hasta
  R5).
- Personajes: primer frame de su hoja (`CHAR_SHEETS`, por el `sprite` de los JSON); sin hoja se dibuja el nombre.
  Sólo los que están a la vista; el héroe caído en gris.
- HUD: nombre, nivel, PV (rojo a 1/3), CA, XP y piso; en combate, ronda y de quién es el turno (y las casillas que
  te quedan); las últimas 4 líneas del log.
- Cámara: centrada en el sprite del héroe; arrastrar la desplaza, al caminar vuelve. Zoom con la rueda o pellizcando:
  factor entero de la UI -2 a +2 (siempre nítido).
