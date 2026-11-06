# D4 — Actualizador

✏️ Escrito (0.9.25), falta la prueba del dueño (necesita una Release publicada): `app/src/updater.rs`
(`docs/app/updater.md`), `MainActivity.installApk`, permiso sólo en la build con `updater`, jobs `release` y `beta`.
Canal beta para testers agregado a pedido del dueño.

**Objetivo:** las builds de GitHub y las propias avisan de una versión nueva y se actualizan. Las de Play y Steam no
traen nada de esto.

## Qué hacer
1. Feature `updater` en `app`, apagada por defecto: la prende el job de Releases (D1); quien compila a mano usa
   `--features updater`. Así una build de tienda no la lleva por olvido.
2. Al abrir (opción "Buscar actualizaciones", prendida): consultar `releases/latest` del repo (sólo estables). Si hay
   una nueva: **Actualizar** / Más tarde.
3. PC: bajar el binario, reemplazar el ejecutable (en Windows, renombrar el que corre) y reiniciar.
4. Android: bajar el APK y abrir el instalador del sistema (`REQUEST_INSTALL_PACKAGES`, desde `MainActivity.java`).
5. D3 lo usa para bajar una versión exacta (`releases/tags/vX`).
6. Sólo HTTPS y sólo del repo del juego; cliente HTTP chico y bloqueante en un hilo (p. ej. `ureq`).

## Para probar
- Con una Release más nueva: aviso al abrir, Actualizar en la PC y en el teléfono.
- Con la opción en No, no consulta nada.
