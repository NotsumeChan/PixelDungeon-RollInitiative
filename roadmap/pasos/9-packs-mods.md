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
  mundos**: `<datos>/instances/<pack>/` con `pack.json` (`name`, `version`, `mods`: [{`id`, `version`, `sha256`,
  `side`}]), `mods/`, `characters/`, `worlds/` ([C1](8-personajes-mundos.md)) y sus ajustes.
- Pantalla de packs: crear, duplicar (con o sin personajes y mundos), renombrar, borrar, elegir el activo (se aplica
  al reiniciar, como hoy). Un pack "Base" sin mods de usuario. Lo de hoy (mods de la carpeta global, personajes)
  pasa a un pack.
- Elegir pack es elegir con qué personajes y mundos se juega: un personaje o mundo nunca se abre con otros mods.
- Ajustes de cada pack: ON / OFF de cada mod y las opciones de cada uno (`settings` de `mod.json`). Los de cliente
  tienen además un interruptor para todos juntos (apagar / prender todos los de cliente); los requeridos se apagan
  de a uno, sin interruptor masivo.
- Exportar / importar un pack como archivo (`.pdpack`: `pack.json` + sus mods, sin personajes ni mundos): para
  pasarlo a mano (Discord, un pendrive) y para respaldos.

## N7 — Formato `.pdmod`
- Zip con `mod.json` obligatorio (`id` con namespace, `version`, `side`): `both` = **requerido** (cambia el contenido
  del juego: entra en el hash de la LAN y hace falta para unirse); `client` = **de cliente / opcional** (gráficos,
  sonido, interfaz: no entra en el hash, cada uno lo prende o apaga). sha256 del archivo.
- Importar a un pack; dos versiones del mismo mod pueden convivir en packs distintos.
- Actualizar un mod dentro de un pack: aviso si sus mundos o personajes usan algo que la versión nueva ya no trae.

## N8 — Personaje en el mundo de otro
- Al unirse a un host con otro pack, el personaje viaja (lo portátil). Lo que sea de un mod que el host no tiene queda
  guardado aparte en el personaje (no se pierde ni se rechaza todo) y vuelve al jugar en su pack.

## N9 — Pedir un pack (compartir entre jugadores)
- No es parte de unirse a una partida: va en la lista de packs, con **PEDIR PACK**. Muestra los packs que comparten
  los jugadores de la LAN (el descubrimiento UDP de siempre; por internet, con la dirección a mano como D2), sin que
  haga falta una partida abierta: alcanza con tener el juego en la pantalla de packs con algo compartido.
- Quien comparte decide qué packs: en los ajustes de cada pack, **"Visible para descarga directa"** (apagado por
  defecto). Al pedirlo, quien lo comparte ve el pedido y lo acepta (o lo rechaza).
- Se baja el pack entero de una vez: requeridos y de cliente (sin personajes ni mundos). Después, en los ajustes del
  pack, cada uno apaga los de cliente que no quiera. Caché por sha256 (lo que ya está no se baja de nuevo).
- Mods con Lua: la advertencia de siempre antes de instalar el pack. Validar lo que llega como un zip importado
  (`registry::is_mod_file`, sin rutas con `..`, tope de tamaño).
- Después, unirse a la partida con ese pack activo. Al conectarse se validan los requeridos: los prendidos tienen que
  ser los mismos que los del host (ids, versiones, sha256; el hash sólo cuenta los requeridos prendidos). Si no
  coinciden, el aviso dice cuáles faltan, sobran o cambian de versión. Los de cliente no se comparan.
