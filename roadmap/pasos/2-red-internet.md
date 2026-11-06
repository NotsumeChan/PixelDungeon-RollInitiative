# D2 — Red por internet

**Objetivo:** jugar fuera de la LAN con un túnel (playit.gg, ngrok, Tailscale) o con el puerto abierto. El broadcast
no pasa por túneles; el TCP 47811 sí.

## Qué hacer
1. UNIRSE acepta `dominio:puerto` o `IP:puerto` (puerto por defecto 47811), resolviendo nombres (hoy
   `ClientGame::connect` recibe un `SocketAddr`). Teclado en pantalla en Android.
2. Contraseña opcional al CREAR; va en el `hello`; el host rechaza si no coincide. La lista de la LAN la pide.
3. La conexión en un solo lugar de `app/src/net.rs` (para sumar Steam después), sin abstracciones nuevas.
4. Tests: unirse por `localhost`, contraseña buena y mala.

## Para probar
- Unirse escribiendo la IP de la PC.
- playit.gg: túnel TCP a `127.0.0.1:47811`; el teléfono con datos móviles entra con esa dirección.
- Contraseña buena, mala y vacía.
