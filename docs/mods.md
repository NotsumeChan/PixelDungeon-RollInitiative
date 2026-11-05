# Mods

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

## Ejemplo: `mods/ejemplo-herrero/`

Un herrero troll en el piso 3 (desconfiado: actitud -1) con tienda de armas, una misión (matar 3 gnolls) que paga con
un ítem propio (`herrero:troll_sword`, hecho con `base` sobre la espada) y una tirada de Persuasión. Para probarlo,
copia la carpeta a `<datos>/mods/`.
