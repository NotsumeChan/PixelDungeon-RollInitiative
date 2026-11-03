# net/lan.py

Transporte LAN, sólo stdlib (fase 5, paso 4). Nada de pygame ni de `game/` salvo `registry` para el hash.

- **Descubrimiento**: `Host` manda por broadcast UDP (puerto `UDP_PORT` 47810) cada `ANNOUNCE_EVERY` (1 s)
  `{"magic": "pddnd", name, players, version, hash, port}`. `Browser` escucha y `games()` da los vistos en los últimos
  `GAME_TTL` (3 s). `set_info(players=...)` cambia lo anunciado (reemplaza el dict entero: sin carreras con el hilo).
- **Conexión**: TCP (`TCP_PORT` 47811) con `asyncio` en un hilo propio (`_Loop`). El juego habla sólo por `inbox`
  (`queue.Queue`) y `send` / `broadcast` / `kick` / `close`, que pasan al hilo de red con `call_soon_threadsafe`.
- **Mensajes**: un JSON por línea, dict con `"t"`. `join` / `leave` los genera la red (un cliente que los manda se
  corta). Línea > `MAX_LINE` (8 MB), JSON roto o sin `"t"` = se corta la conexión (frontera de confianza).
- `content_hash()`: sha256 de los `data/*.json` de cada carpeta de registros (paso 7: no entrar con otro contenido).
- Errores de conexión (puerto ocupado, host que no responde) se lanzan desde el constructor: los maneja la UI.
- ponytail: sin `drain()` (un cliente que no lee acumula buffer); alcanza para 2-4 jugadores en LAN.
- Firewall: hay que permitir UDP 47810 y TCP 47811 en la PC que hostea.
