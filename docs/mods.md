# Mods

**Instalar**: la primera vez, la pantalla de Mods pide aceptar el aviso de responsabilidad (`mods_ack` en
Local.config: no vuelve a aparecer). Mods -> IMPORTAR ZIP abre el selector de archivos del sistema (Windows, Linux y Android); el juego
descomprime el zip en `<datos>/mods/<mod>/` (el nombre es la carpeta de adentro del zip, o el del zip). Sólo copia los
`<registro>.json`, `script.lua`, `mod.json` e `icon.png` (en la raíz del zip o en una carpeta; tope 16 MB); reinicia el juego para cargarlo.
En escritorio también se puede copiar la carpeta a mano.

Un mod es una carpeta dentro de `<datos>/mods/` con archivos de registro con el mismo nombre que los de `data/`
(`classes`, `mobs`, `bestiary`, `items`, `buffs`, `spells`, `npcs`, `dialogs`, `skills`, `skins`, `quests`; `.json`).
`<datos>` es la carpeta de personajes y partidas: Windows `%APPDATA%/PixelDND`, Linux `~/.local/share/PixelDND`,
Android la carpeta de datos de la app.

- Se leen al arrancar, después del juego base, carpetas en orden alfabético: **lo último pisa a lo anterior**, entrada
  por entrada (un id repetido reemplaza la entrada entera; para cambiar sólo unos campos, `"base": "<id>"`).
- Usa tu propio namespace en los ids (`herrero:blacksmith`), nunca `pdri:` salvo para pisar algo del juego base.
- El formato de cada registro está en `docs/data.md`; los NPC, diálogos, misiones y la tienda en `docs/game/npcs.md`.
- Textos: `str` o `{"es": ..., "en": ...}`; si falta el idioma, inglés y si no el que haya.
- Sprites: sólo las hojas que trae el juego (`CHAR_SHEETS` en `app/src/sprites.rs`); un mod no trae imágenes.
- En `npcs.json` (piso -> NPCs) un piso que ya existe se reemplaza: para sumar un NPC a ese piso, copia también los suyos.
- Un archivo que no es un objeto JSON se salta (aviso en la consola). Un error de contenido (un id que no existe) sí
  rompe la partida: probalo.
- La red compara el hash del contenido, mods incluidos: todos en la LAN tienen que tener los mismos mods.

## mod.json, ícono y ajustes

`mod.json` (opcional): `name` y `desc` (textos), `author`, `version` y `settings`, las opciones que el jugador cambia
en Mods -> AJUSTES: `[{"id", "name", "type": "int" | "bool", "default", "min", "max", "step"}]`. Se guardan en
Local.config (`[mod:<carpeta>]`) y cambian en caliente: los scripts las leen en `ctx.settings["<carpeta>"]` y
`npcs.json` puede escalar una probabilidad con `{"scale": ["<carpeta>", "<opción>"]}` (la opción en %).
`icon.png` (opcional, hasta 64x64): el ícono de la lista. En Mods, ON / OFF activa o desactiva un mod (también los
incluidos; `[mods] disabled` en Local.config): se aplica al reiniciar.

## Scripts Lua

Un mod puede traer `script.lua` (Lua 5.4) con funciones que los diálogos llaman:
- condición `["lua", "mi_funcion", args...]`: verdadera salvo que devuelva `nil` o `false`;
- efecto `["lua", "mi_funcion", args...]`: devuelve una lista de efectos con el formato de los diálogos
  (`{{"gold", 10}, {"log", {es = "...", en = "..."}}}`); el juego los aplica.
- Primer argumento `ctx`: `hero` (`name`, `class`, `level`, `hp`, `ht`, `gold`), `npc` (`id`, `name`,
  `disposition`), `depth`, `flags`, `settings` (los ajustes de los mods). Después, los args del JSON.
- Todos los scripts comparten el entorno global: nombra tus funciones con un prefijo propio.
- Sandbox: `math`, `string` y `table`; sin `io`, `os`, `require` ni archivos. Un tope de instrucciones por llamada y
  de memoria (un bucle infinito da error en la consola). `math.random` sale de la partida: repetible.
- Usa enteros (`//` para dividir): un `10.0` no vale como cantidad.
- En la LAN corre sólo en el host; el script entra en el hash del contenido.

## Mods incluidos

`data/mods/<mod>/` van dentro del juego, después del base y antes de los del usuario (que pueden pisarlos):
`mercader-ambulante` (el mercader ambulante: `"*"` en `npcs.json` = cualquier piso, con 25 % en el piso 1 que baja
hasta 0 en el 6 y junto a la bajada; `"wander": true` en el mob =
deambula; es un mob con `base` sobre el mercader).

## Ejemplo: `ejemplo-herrero` (repo de mods)

Los mods del equipo viven en su propio repo (`../pdri-mods/`, GPL v3), no en el del juego. El de ejemplo:

Un herrero troll en el piso 3 (desconfiado: actitud -1) con tienda de armas, una misión (matar 3 gnolls) que paga con
un ítem propio (`herrero:troll_sword`, hecho con `base` sobre la espada), una tirada de Persuasión y una apuesta
con un dado hecha en Lua (`script.lua`). Para probarlo,
copia la carpeta a `<datos>/mods/`.
