# game/characters.py

Personajes creados, guardados en disco. Sin pygame.

- Archivo: `characters.json` en la carpeta de datos del usuario (`data_dir()`):
  - Windows: `%APPDATA%/PixelDND`
  - Linux: `$XDG_DATA_HOME/PixelDND` (por defecto `~/.local/share/PixelDND`)
  - Android: pendiente.
- Personaje: `{"id", "name", "class", "abilities", "created"}`. El `id` es un uuid en texto; la clase es un ID
  de texto (`pddnd:warrior`), así que una clase de mod se guarda igual.
- `load()`: si el archivo está roto, lo **renombra** (`characters.json.roto-FECHA`) en vez de pisarlo, para no
  perder datos, y devuelve una lista vacía.
- `save(lista)`: escritura atómica (archivo temporal + reemplazo).
- `new_character(nombre, clase, características)`.
- Las partidas en curso viven sólo en memoria (`Globals["runs"]`). Guardarlas es parte de la 0.8.
