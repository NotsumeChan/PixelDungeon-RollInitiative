# app/src/net.rs

Red LAN (R6, `net/` del prototipo), sólo `std`: hilos y canales, sin tokio. Los hilos de red nunca tocan `game` ni el
render; el juego llama a `pump` una vez por frame.

- **Descubrimiento** (`Browser`): el cliente manda `{"magic": "pdri", "t": "discover"}` por broadcast UDP al puerto
  47810 cada segundo; el host le contesta directo (unicast) con nombre, jugadores, versión, hash del contenido y puerto
  TCP. Así el teléfono encuentra la partida de la PC sin MulticastLock (recibir broadcasts en Android lo pide; recibir
  unicast no). Una partida sin respuesta en 3 s sale de la lista.
- **Conexión**: TCP al puerto 47811, un JSON por línea (máx. 8 MB; más largo o basura corta la conexión).
  Cliente -> host: `hello` (versión, hash, nombre, id del personaje, clase, características) y `action`.
  Host -> cliente: `state` (`you` = su índice en `heroes`, `save` = `save::dump`) y `refused` (motivo).
- **Acciones como datos** (`action_v` / `action_of`): `Action` <-> JSON con celdas, índices de ítems y ids. Frontera de
  confianza: lo que no se entiende o no valida (`valid`: celdas en rango, ítems que existen, sólo atacar mobs, ayudar a
  otro héroe que existe, conjuros que tiene) se ignora; el juego vuelve a validar al actuar.
- **`HostGame`**: `hello` válido (misma versión y contenido, clase cargada, características 1-20) suma un héroe al piso
  1 con el nombre del personaje; si el mismo personaje se reconecta en la sesión, vuelve a su héroe. `action` va a SU
  héroe. Al irse, su héroe queda (la app lo hace descansar). Manda el estado completo como mucho cada 50 ms y sólo si
  cambió.
- **`ClientGame`**: manda `hello` al conectar; `pump` devuelve la partida armada (`save::load`) con el último estado.
- `local_ip`: la IP de la PC en la LAN, para el log del host (UDP `connect` no manda nada).
- Tests (los únicos automáticos del proyecto, regla del dueño): acciones de ida y vuelta, acciones inválidas, una
  línea por TCP, y una partida real por loopback (el cliente entra, recibe su héroe, su acción llega a su héroe y al
  irse queda libre). `cargo test -p app --lib net` (usa los puertos del juego).

Pendiente (ponytail): el personaje entra con nivel 1 y equipo inicial (personaje portátil, paso 5), al irse su héroe
no sale de la partida, el cliente rearma la partida entera en cada estado (regenera los pisos) y no ve los dados ni
los efectos del host (ve el resultado en el log), estado completo en vez de diferencias.

Sala de espera: `HostGame` empieza sin correr la partida (`started = false`): suma a los que se unen y les manda
`{"t": "lobby", "players": [...]}` al cambiar la lista; `start` (EMPEZAR) manda la partida a todos. El estado lleva
`wait` (por héroe, si la partida espera que elija: `needs_input`) para la lista de quién falta.

Efectos y dados: el host manda `{"t": "fx", "fx": [...], "dice": [...]}` con lo que pasó en el piso de cada jugador
(animaciones de ataque, números flotantes, muertes, casillas revisadas, dados), en el acto. El cliente los pasa a su
partida y los ve igual que el host. Validados: animaciones conocidas, celdas en rango, dados con rangos razonables.
