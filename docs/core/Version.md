# core/Version.py

Única fuente de la versión. `VERSION` lo usan el título (abajo a la derecha) y el About.
El historial de versiones vive sólo en `roadmap/versiones.md` (el About no lo muestra).

- Al cerrar un lote de trabajo: subir `VERSION` y agregar la fila en `roadmap/versiones.md`.
- **1.0 = multijugador P2P listo.**
- Antes la versión salía de `Local.config`; ahora es del código (una config local no debería definir la versión).
