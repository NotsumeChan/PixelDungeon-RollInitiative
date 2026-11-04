# app/src/config.rs

`core/Config.py`: `Local.config`, el mismo INI del prototipo. Rust lee y guarda sólo sus claves y conserva el resto del
archivo: `[general] maxFPS` (30 / 60 / 120 / 0 = sin límite; sin archivo, 60), `fullscreen` y `resolution`
(tamaño de la ventana, `RESOLUTIONS`: Basic 640x480, Hd 1280x720, Fhd 1920x1080; sin archivo, Hd), y la sección `[keys]`
(una por acción, con el nombre de la tecla en winit: `up = KeyW`). En escritorio el archivo está en la carpeta desde
donde se corre; en Android, en la carpeta de datos de la app (`DIR`, lo pone `android_main`).

- `Bind`: acciones con tecla configurable (moverse, esperar, buscar, mochila, accesos rápidos). Las flechas siempre
  mueven; Esc, R, C y F11 son fijas. `rebind` intercambia si la tecla ya era de otra acción.
- Cada cambio se guarda al momento; si no se puede escribir, sigue en memoria (aviso en el log).
- `language` (es / en): el idioma del contenido (`game::registry::set_lang`). Los textos del código siguen en
  español, como en el prototipo; falta la estructura para traducirlos.
- `[dev] dice_log = true`: el log muestra el cálculo de cada tirada (`Dungeon::verbose_rolls`). Sólo en el archivo.

`ui_scale` (escala de la interfaz, como el prototipo): la app la pasa a `Gpu::set_ui_scale`; `ui_base` es el zoom que entra, para armar las opciones.
