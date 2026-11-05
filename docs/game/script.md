# game/src/script.rs

Scripts Lua de los mods (`docs/mods.md`) con `mlua` (Lua 5.4 compilado con el juego, `vendored`: también en
Android). Un estado de Lua por hilo (`thread_local`), creado la primera vez que se llama: carga los `script.lua` de
`registry::scripts()` en orden de carpeta.

`call(función, ctx, args, seed)`: llama la función global con el contexto y los args (JSON <-> Lua con `serialize`)
y devuelve lo que devuelva como JSON. `math.randomseed(seed)` antes de cada llamada (la condición usa 0; el efecto,
el rng de la partida). Tope: `BUDGET` * 10 000 instrucciones por llamada (hook) y `MEMORY` de memoria. Sin `io`, `os`,
`package`, `debug`, `dofile`, `loadfile` ni `require`.

Lo usan las condiciones y efectos `["lua", ...]` de los diálogos (`npcs.rs`, `lua_call`): el script no toca la
partida, devuelve efectos que el juego aplica (un `"lua"` dentro de esos efectos se ignora).
