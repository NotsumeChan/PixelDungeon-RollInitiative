# game/registry.py

Carga los registros de contenido de `data/<nombre>.json`: `classes`, `mobs`, `bestiary`, `items`, `buffs`, `spells`
(los usan `hero.HERO_CLASSES`, `mobs.MOBS` / `BESTIARY`, `items.ITEMS`, `buffs.BUFFS`, `spells.SPELLS`).

- `DATA_DIRS`: carpetas en orden; lo último pisa a lo anterior. Hoy sólo `data/` del juego; los mods agregarán las suyas.
- `"base": "<id>"`: la entrada copia esa otra y aplica sus campos encima (custom basado en uno del juego). Se resuelve
  en cadena; una base inexistente o circular es un error al cargar.
- El orden de las claves del JSON es el orden del registro: el generador de ítems y el bestiario dependen de él
  (determinismo entre peers).
- En JSON las tuplas son listas: el código las desempaqueta igual; `ItemIcon` convierte la imagen `[archivo, índice]`
  a tupla para la caché. Las claves del bestiario son texto y `mobs.py` las pasa a `int`.
- Qué campos tiene cada registro: `docs/data.md`.
