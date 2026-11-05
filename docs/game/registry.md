# game/src/registry.rs

`game/registry.py`: `load("items")` devuelve el `Map` de `data/items.json` con las entradas `"base"` resueltas. Los
JSON van dentro del binario (`include_str!`). `serde_json` con `preserve_order`: el orden de las entradas es el del
archivo (decide tiradas, como el dict de Python). Una base que no existe o un ciclo es un error con el id.

`text(valor)`: un texto de contenido (`str` o `{"es", "en"}`) en el idioma del juego.

Idioma (`LANGS`, `set_lang`, `lang`, como `game/lang.py`): `text` elige el idioma actual; si falta, inglés y si no el
primero que haya. Sólo cambia textos, nunca una tirada (dos jugadores con distinto idioma ven la misma partida).

`tr!`/`trf!`: textos del código (UI, mensajes) en el idioma actual: `tr!("Atrás", "Back")`, `trf!("Recoges {n}", "You pick up {n}")`.

`content_hash`: FNV-1a de 64 bits de los JSON de los registros, en orden (estable entre plataformas). La red no deja
entrar a un cliente con otro contenido.
