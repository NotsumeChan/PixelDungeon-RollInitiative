# D3 — Compatibilidad entre versiones

✏️ Escrito (0.9.24), falta la prueba del dueño: `PROTOCOL = 1` y el aviso de quién actualiza (`docs/app/net.md`). El
botón Actualizar llega con D4.

**Objetivo:** que 1.1.0 y 1.1.2 jueguen juntas si nada de la red o las reglas cambió; si no pueden, que el cliente
sepa qué hacer.

## Qué hacer
1. `PROTOCOL` en `app/src/net.rs`, que sube sólo cuando cambia la red o las reglas. Al unirse se comparan `PROTOCOL`
   y el hash del contenido, no la versión exacta. La regla, en `versiones.md`.
2. Si no son compatibles:
   - Cliente más viejo: "El host tiene la X" + **Actualizar** (con `updater`) que baja la Release `vX` exacta.
   - Cliente más nuevo: "El host tiene que actualizar" (Android no baja de versión).

## Para probar
- Mismo `PROTOCOL`, distinto parche: entran.
- `PROTOCOL` distinto: el aviso correcto en cada lado.
