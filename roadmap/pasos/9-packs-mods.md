# N6 — Paquetes de mods

Decisiones en [`decisiones.md`](../decisiones.md#personajes-y-mundos-2026-11-06) y en "Mods". Sólo plan.

## Hoy
- Una carpeta global `<datos>/mods/`: los activos se cargan al arrancar, después del base y de los incluidos
  (`data/mods/`), en orden alfabético; lo último pisa (`docs/mods.md`). ON / OFF y ajustes por mod en Local.config.
- Importar: un zip (sólo registros `.json`, `script.lua`, `mod.json`, `icon.png`). Reiniciar para cargar.
- LAN: el hash del contenido (mods incluidos) tiene que coincidir; si no, rechazo y a conseguirlos a mano.
- Un personaje con un ítem de un mod que el mundo no tiene: `parse_portable` rechaza **todo** lo portátil.

## N6 — Packs como instancias (primero)
- Como Modrinth / los gestores de mods de Minecraft: cada pack es una instancia con sus mods **y sus personajes y
  mundos**: `<datos>/instances/<pack>/` con `pack.json` (`name`, `version`, `mods`: [{`id`, `version`, `sha256`}]),
  `mods/`, `characters/`, `worlds/` ([C1](8-personajes-mundos.md)) y sus ajustes (ON / OFF y opciones de cada mod).
- Pantalla de packs: crear, duplicar (con o sin personajes y mundos), renombrar, borrar, elegir el activo (se aplica
  al reiniciar, como hoy). Un pack "Base" sin mods de usuario. Lo de hoy (mods de la carpeta global, personajes)
  pasa a un pack.
- Elegir pack es elegir con qué personajes y mundos se juega: un personaje o mundo nunca se abre con otros mods.

## N7 — Formato `.pdmod`
- Zip con `mod.json` obligatorio (`id` con namespace, `version`, `side`: `both` | `client`); sha256 del archivo.
  Importar a un pack; dos versiones del mismo mod pueden convivir en packs distintos.
- `side: client` (gráficos, sonido): no entra en el hash de la LAN.
- Actualizar un mod dentro de un pack: aviso si sus mundos o personajes usan algo que la versión nueva ya no trae.

## N8 — Personaje en el mundo de otro
- Al unirse a un host con otro pack, el personaje viaja (lo portátil). Lo que sea de un mod que el host no tiene queda
  guardado aparte en el personaje (no se pierde ni se rechaza todo) y vuelve al jugar en su pack.

## N9 — Unirse a un host
- El host manda su pack; el cliente baja los mods que falten (caché por sha256, del host) y arma un pack temporal
  con ellos. Mods con Lua: la advertencia de siempre antes de bajarlos.
