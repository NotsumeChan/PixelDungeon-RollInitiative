# C — Personajes y mundos (estilo Terraria)

Decisiones en [`decisiones.md`](../decisiones.md#personajes-y-mundos-2026-11-06). Sólo plan: partir cada punto en
versiones chicas al llegar.

## Hoy
- `characters.json`: identidad (id, nombre, clase, características, apariencia).
- Una partida por personaje (`saves/<id>.json`): lo portátil (nivel, XP, oro, arma, armadura, mochila, accesos
  rápidos; `save::portable`) vive **dentro** de esa partida. En la LAN del amigo viaja validado (`parse_portable`) y al
  salir vuelve a la partida propia del personaje (`autosave`). Personaje y partida son lo mismo: el progreso en el
  mundo de otro se mezcla con el propio. No se puede duplicar ni editar un personaje.

## C1 — Formato: personaje y mundo separados
- `characters/<id>.json`: identidad + lo portátil (nivel, XP, oro, equipo, mochila, accesos rápidos; más adelante
  competencias, conjuros y rasgos de [P](7-progresion-5e.md)) + estado: vivo, o caído (en qué mundo y cuándo).
- `worlds/<id>.json`: nombre, seed, zonas, mobs, flags y misiones, reglas del mundo, pack de mods (ver
  [N6](9-packs-mods.md)) y por personaje su estado **en ese mundo** (zona, posición, PV, estados, salvaciones):
  `heroes: {char_id: ...}` y `fallen: [char_id]`.
- Sin migración (regla del proyecto): los personajes quedan, las partidas viejas se apartan.

## C2 — Elegir personaje y mundo
- Lista de personajes -> lista de mundos: nuevo (generado o prefabricado, M2), entrar, renombrar, borrar.
- Cualquier personaje vivo entra a cualquier mundo propio; aparece donde quedó en ese mundo o en la entrada.
- Autoguardado: el mundo guarda su estado; el personaje, lo portátil (siempre en su archivo, juegue donde juegue).

## C3 — Muerte hardcore y baneo
- Morir mata al personaje para siempre; el mundo lo anota en `fallen` y no lo deja volver a entrar: para seguir en
  ese mundo, otro personaje. La lista de personajes lo muestra "caído en <mundo>".
- En la LAN el host rechaza a un personaje caído en su mundo (por id).
- Las excepciones (el DM que salva a un personaje) van por las reglas del mundo (C5) y la idea del DM (`pendientes.md`).

## C4 — Duplicar y editar
- Editar: nombre y apariencia. Duplicar: copia con otro nombre (todo lo portátil).
- Ojo: duplicar copia inventario; en mundos de otros, el host decide si acepta personajes duplicados (regla del mundo).

## C5 — Reglas del mundo
- Al crear un mundo (generado o prefabricado: one-shot, historia larga) se eligen sus modificadores generales: las
  `Rules` de hoy, las de [P4](7-progresion-5e.md), excepciones a la muerte, aceptar duplicados...
- Se pueden cambiar a lo largo de la partida, pero sólo desde fuera: salir, cambiar la configuración del mundo y
  volver a entrar (nada cambia en caliente). En la LAN, las del host.

## Red
- Ya está (R6): lo portátil validado al entrar, y de vuelta al archivo del personaje al salir. Con C1, el host guarda
  en su mundo sólo el estado de cada personaje en ese mundo; nunca el progreso portátil de otros.
