# N6 — Paquetes de mods

Decisiones en [`decisiones.md`](../decisiones.md#personajes-y-mundos-2026-11-06) y en "Mods". Sólo plan.

## Hoy
- Una carpeta global `<datos>/mods/`: los activos se cargan al arrancar, después del base y de los incluidos
  (`data/mods/`), en orden alfabético; lo último pisa (`docs/mods.md`). ON / OFF y ajustes por mod en Local.config.
- Importar: un zip (sólo registros `.json`, `script.lua`, `mod.json`, `icon.png`). Reiniciar para cargar.
- LAN: el hash del contenido (mods incluidos) tiene que coincidir; si no, rechazo y a conseguirlos a mano.
- Un personaje con un ítem de un mod que el mundo no tiene: `parse_portable` rechaza **todo** lo portátil.

## N6 — Perfiles locales (primero)
- `<datos>/instances/<pack>/`: `pack.json` (`name`, `version`, `mods`: [{`id`, `version`, `sha256`}]) y los mods del
  perfil en su carpeta. Estilo Modrinth: varios perfiles, uno activo.
- Pantalla de packs: crear, duplicar, renombrar, borrar, elegir el activo (se aplica al reiniciar, como hoy). Un perfil
  "Base" sin mods de usuario. Los mods de la carpeta global de hoy pasan a un perfil.
- Ajustes y ON / OFF por perfil, no globales.

## N7 — Formato `.pdmod`
- Zip con `mod.json` obligatorio (`id` con namespace, `version`, `side`: `both` | `client`); sha256 del archivo.
  Importar a un perfil; dos versiones del mismo mod pueden convivir en perfiles distintos.
- `side: client` (gráficos, sonido): no entra en el hash de la LAN.

## N8 — Mundo y pack
- El mundo guarda el pack con que se creó (ids, versiones, sha256). Abrirlo con otro perfil: aviso (cambiar de perfil o
  seguir igual).

## N9 — Personaje y mods
- Lo portátil anota de qué mod es cada ítem. Al entrar a un mundo sin ese mod, sólo esos ítems quedan guardados
  aparte en el personaje (no se pierden ni se rechaza todo); vuelven al entrar a un mundo que lo tenga.

## N10 — Unirse a un host
- El host manda su pack; el cliente baja los mods que falten (caché por sha256, del host) y arma un perfil temporal
  con ellos. Mods con Lua: la advertencia de siempre antes de bajarlos.
