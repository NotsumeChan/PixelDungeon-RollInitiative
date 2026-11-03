# game/src/registry.rs

`game/registry.py`: `load("items")` devuelve el `Map` de `data/items.json` con las entradas `"base"` resueltas. Los
JSON van dentro del binario (`include_str!`). `serde_json` con `preserve_order`: el orden de las entradas es el del
archivo (decide tiradas, como el dict de Python). Una base que no existe o un ciclo es un error con el id.
