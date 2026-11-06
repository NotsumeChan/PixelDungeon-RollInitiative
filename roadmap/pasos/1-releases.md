# D1 — Releases en GitHub

**Objetivo:** cada merge a `main` publica la Release `v<versión>` con Linux, Windows y el APK firmado.

**Hoy:** `.github/workflows/build.yml` compila los 3, pero sólo deja *artifacts* (vencen a los 90 días).

## Qué hacer
1. Job en `push` a `main`: lee la versión de `Cargo.toml`; si el tag no existe, `gh release create` con los 3
   archivos (`permissions: contents: write`).
2. APK firmado con los secretos `PDRI_KEYSTORE_B64` y `PDRI_KEYSTORE_PASS`.
3. `android:hasFragileUserData="true"` en el manifiesto; verificar que `data_dir()` (`app/src/lib.rs`) sea
   almacenamiento interno de la app.

**Del dueño:** cargar los dos secretos.

## Para probar
- Merge a `main` → Release con 3 archivos; otro push sin subir versión no la duplica.
- El APK de la Release se instala sobre el actual.
- Al desinstalar aparece "Conservar datos"; marcada, al reinstalar vuelven los personajes.
