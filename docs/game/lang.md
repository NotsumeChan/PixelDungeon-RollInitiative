# game/lang.py

Idioma del contenido. No viene del Java (el original sólo está en inglés).

- `LANGS`: idiomas disponibles (`es`, `en`). `LANG`: el actual. `set_lang(código)`.
- `t(texto)`: un texto de contenido puede ser un `str` (un solo idioma) o un dict `{"es": ..., "en": ...}`. Devuelve
  el del idioma actual; si falta, inglés; si tampoco, el primero que haya.
- Lo usan los nombres y descripciones de ítems, apariencias de pociones, mobs y su estado, clases, estados alterados,
  terreno y montones. **Un mod puede traer sus textos en los idiomas que quiera** con el mismo formato.
- Los nombres se resuelven al mostrarlos: cambiar el idioma en plena partida cambia los nombres de ítems, mobs,
  estados y terreno. El nombre del héroe se fija al crearlo.
- `core/Config.py` lee `[general] language` de `Local.config` al arrancar; la ventana de configuración lo cambia y lo guarda.

## Pendiente
Las frases de la interfaz y del log ("Recoges...", "golpea a", la ficha, la mochila) siguen sólo en español.
Siguiente paso: un catálogo de frases con `t()`.
