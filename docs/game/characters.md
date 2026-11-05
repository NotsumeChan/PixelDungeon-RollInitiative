# game/src/characters.rs

`game/characters.py`: la lista de personajes en `<datos>/characters.json` ({"id", "name", "class", "abilities",
"created"}). Escritura atómica (`.tmp` y renombrar); un archivo roto se aparta (`.roto-<hora>`), no se pisa. Las
entradas se conservan tal cual (las del prototipo, con clases `pddnd:`, se ven como "clase desconocida").
La carpeta de datos la pone la app (la misma que el prototipo: `%APPDATA%/PixelDND`, `~/.local/share/PixelDND`; en
Android, la de la app).
