# app/src/lib.rs (+ main.rs)

Bucle del juego sobre winit 0.30 (`ApplicationHandler`), el mismo para escritorio (`main.rs`) y Android
(`android_main`, NativeActivity).

- `resumed`: crea ventana y `Gpu` (en Android, cada vez que la app vuelve del fondo) y marca la niebla para subirla.
- `ControlFlow::Poll` + `request_redraw` en `about_to_wait`: se dibuja sin parar.
- `frame`: dt en segundos (tope 0,1 s), FPS por segundo (también al log: `adb logcat | grep FPS`).

R3: **jugar un piso** con el héroe (`World`: la partida `game::dungeon::Dungeon` + dónde se dibuja cada personaje).
- Cada frame (`World::update`, en el orden del prototipo): teclas mantenidas, los sprites caminan (`sprites.rs`; el
  héroe se suelta al llegar), lo que el dado ya mostró se aplica (`resolve_pending` sólo si `DiceFx` no está
  ocupado), `process`, y después efectos y dados avanzan.
- Input: un click / toque va primero a la UI (`ui.rs`); si no la toca, al mapa: examinar la casilla (si se eligió
  examinar) o `Dungeon::handle` (caminar con pathfinder, atacar, ayudar a un caído, cambiar de piso en la escalera);
  tocar al propio héroe = buscar. WASD / flechas = un paso (diagonal si se puede, si no por un eje; atacar si hay
  alguien). Espacio = esperar, Q = buscar, I = mochila, 1 / 2 = accesos rápidos, click derecho = elegir el acceso
  rápido, Esc = cerrar ventana / cancelar examinar / menú, R = otra partida del mismo personaje.
- Personajes animados (`sprites.rs`, hojas `CHAR_SHEETS` por el `sprite` de los JSON; sin hoja, el nombre). Lotes del
  mundo: atlas (mapa, cadáveres, personajes) | niebla | atlas otra vez (hojas, textos flotantes, nombres). Los dados
  (`dice_fx.rs`) van en la capa de UI.
- UI (`ui.rs`): panel de estado, carrusel de iniciativa, barra de herramientas, log, ventanas y tooltips; los dados
  van encima. FPS bajo el avatar (se apaga con la configuración, R5).
- Cámara: centrada en el sprite del héroe; arrastrar la desplaza, al caminar vuelve. Zoom con la rueda o pellizcando:
  factor entero de la UI -2 a +2 (siempre nítido).

Tope de FPS (`config.max_fps`): `about_to_wait` espera con `ControlFlow::WaitUntil` hasta el próximo frame; sin tope,
`Poll` y se dibuja sin parar. Las teclas de juego salen de `Config` (configurables); F11 = pantalla completa.

R5c: F = rasgo de clase (Segundo Aliento o el libro de conjuros); eligiendo el objetivo de un conjuro, el próximo click en el mapa lo lanza (Esc cancela).

El juego arranca en la pantalla de título (`title`): sólo se dibuja la capa de UI y la partida de atrás no avanza.
Jugar = la lista de personajes. Esc cierra la ventana abierta (en la creación de personaje, vuelve a la lista);
F11 = pantalla completa.

Creación de personaje: las teclas van primero a `Ui::create` (nombre o número que se escribe, con el texto de
`logical_key`); `Cr::Start` crea el personaje (sin nombre: el de la clase), lo guarda en la lista y empieza.
En Android, `sync_ime` muestra el teclado en pantalla mientras se escribe (`set_ime_allowed`).
Resolución: en escritorio la ventana se crea con `Config::window_size` y cambia al elegir otra (si no está en
pantalla completa).

Personajes y guardado: Jugar abre la lista (`open_roster`, lee el disco); JUGAR carga la partida guardada del personaje
si sigue vivo o empieza una nueva (`start`). Autoguardado (`autosave`) al cambiar de piso, al volver al menú principal,
al salir y al pasar al fondo en Android. Carpeta de datos: `data_dir()`. R = partida nueva del mismo personaje.

R5d: con `ui.throwing` el próximo click en el mapa es `Action::Throw` (Esc cancela). El arco no necesita UI: tocar
un mob a tiro dispara.

Arrastrar en la creación de personaje: `press` / `drag` / `release` pasan primero por `Ui::create` (si se apretó sobre
una tirada o una característica con valor), antes que el arrastre de la cámara. El idioma se aplica antes de crear la
partida de fondo (`App::new`) y al cambiarlo se rehacen las filas de clases. `[dev] dice_log` se copia a
`verbose_rolls` cada frame.

Hotseat (`[dev] hotseat = true`, el modo de prueba de la red del prototipo): `World::new` suma un segundo héroe igual
al personaje ("<nombre> 2"). Tab pasa al siguiente héroe vivo. `World::auto_switch` (cada frame): si el local murió o
cayó, la UI pasa al primero en pie; si el piso espera la acción de otro héroe (`Dungeon::blocking`), pasa a ese. Con los
héroes en pisos distintos, cada piso espera a su héroe: se cambia con Tab. Con 2+ héroes, `sprites.rs` dibuja el
nombre sobre cada uno (el local amarillo, otros blanco, caídos gris). La partida guarda los dos héroes.

Red LAN (`net.rs`, `Net`: `Off` / `Host` / `Client`): CREAR LAN en la lista de personajes juega con el elegido y abre
la partida (el log muestra la IP); UNIRSE busca partidas (`Browser`) y al tocar una se conecta con el personaje
elegido; con el primer estado se entra a jugar. `net_pump` (cada frame, también en el título): el host aplica las
acciones y manda el estado; el cliente manda lo que eligió su héroe (la UI arma la acción sobre su copia, como en
local) y espera el estado (`awaiting`, para no repetir pasos con la tecla mantenida). El cliente no corre la partida
(`World::update(dt, remote)`), no guarda, y R / nueva partida no hacen nada en red. Si la conexión se corta o el host
lo rechaza, vuelve a la ventana Unirse con el motivo. Los héroes que nadie maneja (un jugador que se fue o una partida
guardada con más héroes, fuera del hotseat) descansan solos. Menú principal cierra la partida LAN.

Sala de espera: CREAR LAN carga la partida del personaje y abre `Window::Lobby` (quién se unió e IP); EMPEZAR
(`StartLan`) entra a jugar, VOLVER cierra la partida LAN. El cliente espera en Unirse viendo quién está en la sala.
`ui.waiting` (por héroe, ¿la partida espera que elija?): el host lo calcula cada frame y el cliente lo recibe con el
estado. `UiAction::Trait` = F o el botón de la barra. Atrás de Android = Esc. Sin IME: el teclado del sistema no le
pasa letras a una NativeActivity; la creación de personaje tiene teclado en pantalla.

Android: al terminar el bucle (Salir) `android_main` cierra el proceso (`process::exit`): si no, Android lo deja vivo y
al volver a abrir la app queda trabada esperando un bucle que ya terminó.

Chat LAN: mientras `ui.chat` está abierto, las teclas son texto (Esc cancela, Enter manda, Borrar); `UiAction::Chat`
lo manda al host (`HostGame::chat`) o al host remoto (`ClientGame::send_chat`). Enter abre el chat sólo en red y si no
está asignado a otra acción.

## Teclado del sistema en Android (0.9.23)
NativeActivity no le da a la app el texto del teclado: el teclado del sistema lo manda por una InputConnection
(`commitText`) y NativeActivity no tiene. `MainActivity.input()` es una vista invisible de 1x1 que se presenta como
campo de texto (`VISIBLE_PASSWORD`: letra por letra, sin sugerencias); su `commitText` llama a `nativeChar`, y Enter
y borrar los manda como teclas. `dispatchKeyEvent` cubre un teclado físico (`getUnicodeChar`). `nativeChar` que `android_main` registra por
JNI (`native_char` -> `TYPED`). Enter, borrar y Atrás siguen por winit. `App::text_input`, una vez por frame, abre o
cierra el teclado (`MainActivity.setKeyboard`) según haya algo que escribir (dirección, clave, chat, nombre o
características) y pasa las letras de `TYPED` a eso. En Android se ignoran los `Key::Character` de winit para no
escribir dos veces. Si Atrás cierra el teclado, tocar la línea lo vuelve a abrir (`kb_again`).
