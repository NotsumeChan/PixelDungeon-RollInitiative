# Roadmap

Leer esto primero. Un archivo por tema:

| Archivo | Qué tiene |
|---|---|
| [`decisiones.md`](decisiones.md) | Lo que decidió el dueño |
| [`estado.md`](estado.md) | Lo escrito por fase (R0-R6, N1-N5) y qué probar |
| [`pendientes.md`](pendientes.md) | Lo que falta, sin orden |
| [`versiones.md`](versiones.md) | Regla de versiones e historial |
| [`reglas5e.md`](reglas5e.md) | Reglas 5e y equipo |
| [`sprites-5e.md`](sprites-5e.md) | El arte que hace falta para 5e (personajes, equipo, mobs, interfaz) |
| [`pasos/`](pasos/) | Próximos pasos, uno por archivo |

## Ahora
Rama `dev`, **0.9.45** (`versionCode` 55). Casi todo está ✏️ (sin probar): lo que falle en la prueba del dueño va
primero.

## Próximos pasos
| # | Paso | Etiqueta |
|---|---|---|
| 1 | ✏️ [Releases en GitHub](pasos/1-releases.md) | `#D1` |
| 2 | ✏️ [Red por internet](pasos/2-red-internet.md) | `#D2` |
| 3 | ✏️ [Compatibilidad entre versiones](pasos/3-compatibilidad.md) | `#D3` |
| 4 | ✏️ [Actualizador](pasos/4-updater.md) | `#D4` |
| 5 | ✏️ M1a, M1b, M1c, M1d a medias · [Zonas, mapas y editor](pasos/5-mapas-editor.md) | `#M1`… |
| 6 | [Sprites como datos y apariencias por capas](pasos/6-sprites.md) | `#S1`… |
| 7 | [Personaje 5e: creación y progresión](pasos/7-progresion-5e.md) | `#P1`… |
| 8 | Plan · [Personajes y mundos (estilo Terraria)](pasos/8-personajes-mundos.md) | `#C1`… |
| 9 | Plan · [Paquetes de mods](pasos/9-packs-mods.md) | `#N6`… |
| 10 | ✏️ L1, L2 en parte · [Nombre propio y avisos legales](pasos/10-nombre-avisos.md) | `#L1`… |

Estética: en conversación, sin paso.

**Meta a largo plazo:** separarse por completo de Pixel Dungeon (ver [`decisiones.md`](decisiones.md#meta-un-juego-propio-2026-11-06)).
Lo nuevo nace propio; lo de PD se reemplaza de a poco.

## Cómo se trabaja
- Sólo compilar (`cargo clippy --workspace --all-targets`, `cargo ndk -t arm64-v8a check -p app --lib`) y tests de
  red (`cargo test -p app --lib net`). `cargo ndk` necesita `ANDROID_NDK_HOME` (fuera de `android/build.sh` no está):
  `export ANDROID_NDK_HOME=$(ls -d ~/.buildozer/android/platform/android-ndk-r*/ | sort -V | tail -1)`. Nada de correr el juego ni simulaciones: el dueño prueba a mano.
- Commits en `dev`, a nombre del dueño, sin coautoría, con la etiqueta del paso. Cada cambio sube la versión.
- Al terminar algo: su "Para probar" en `estado.md` y esta tabla al día.
- Referencias: prototipo en la rama `python-prototipe` (con sus planes viejos en `roadmap/`), Java original en
  `~/Documents/programacion/personal/java/pixel-dungeon/`.

## Estado al cerrar la sesión (2026-11-06, tarde)
**Para retomar:** leer esta sección. No hay nada a medio hacer: todo está en commits en `dev` (**0.9.45**, `PROTOCOL`
4), sin pushear. Notas locales (no del proyecto) en `NOTAS.md`. Glosario de zona / escena / profundidad en
`docs/game/dungeon.md`. Reglas nuevas en `CLAUDE.md`: nunca nombrar D&D (ver L).

### Qué probar (todo ✏️, lo que falle va primero)
Lo anterior a esta sesión sigue sin probar: D1-D4, M1a (0.9.26), fuente Silkscreen (0.9.27), visión / pathfinder /
línea de tiro propios (0.9.28), configuración por pestañas (0.9.29, 0.9.33), M1b y el generador propio (0.9.31-32).
De esta sesión, por versión:
- **0.9.34 M1c, bordes y portales:** mod con dos zonas lado a lado y un portal (ejemplo en `docs/data.md`; pasos en
  [`pasos/5-mapas-editor.md`](pasos/5-mapas-editor.md)). Un borde con zona al lado se pasa con sólo pisarlo (mouse y
  teclado); sin zona al lado, pared invisible. El mundo base debe verse igual; el borde del mapa ya no se ve negro.
- **0.9.35 turnos fuera de combate:** en LAN u hotseat, un jugador quieto no frena al otro (los mobs también se mueven);
  los dos quietos = se espera. Aliado quieto en un pasillo: se cambian de lugar; de frente: se cruzan; uno detrás del
  otro: lo sigue sin cortarse. En combate, como antes.
- **0.9.36 pathfinder:** recto por una sala abierta, diagonal suave en ángulo, sin zigzag (mobs también).
- **0.9.37 tocarse:** sobre escalera, portal o borde, la usa; sobre un montón, lo recoge; sin nada, busca.
- **0.9.38 / 0.9.44 acción rápida, Seguir, arrojadizas:** botón sobre la barra a la derecha (retrato del enemigo más
  cerca o el ítem más cerca; E); Seguir celeste arriba a la derecha, sólo fuera de combate (X); la arrojadiza del
  acceso rápido apunta sola al más cerca y se lanza tocándolo otra vez. Vertical: que no tapen el log. Controles con
  4 filas.
- **0.9.39 / 0.9.41 combate:** carrusel sólo con los que pelean o se ven; "Fin del combate" al matar al último;
  "Experiencia del combate: +N" al terminar (no durante); guardar a mitad de pelea no la pierde. Un mob que llega a
  mitad se suma al final de la ronda. Una escena lateral no cuenta para el combate de otra.
- **0.9.40 L1:** README (sección Avisos) y Acerca de, en horizontal y vertical (que entre todo).
- **0.9.42 M1d:** pasar el mouse por la profundidad muestra el nombre y las coordenadas de la zona.
- **0.9.43 L2:** la ficha y la lista de personajes siguen mostrando la clase 5e (`class_5e`).
- **0.9.45 mods:** lista a todo el ancho; nombres largos con ".."; tocar un mod abre su menú (descripción, ajustes,
  BORRAR / SEGURO? en los importados); scroll con rueda y flechas.

### Decisiones abiertas (del dueño)
- **N8:** un personaje en el mundo de un host con otro pack: ¿lleva su progreso (y guarda aparte lo de mods que faltan)
  o regla estricta (cada personaje sólo en su pack)? Plan hoy: lo lleva.
- **M1d:** cómo es la brújula o el minimapa del mundo.
- **Por discutir** (`pendientes.md`): plantillas de personaje, pregenerados, creación rápida; el DM y las reglas
  dinámicas.
- **L2 / L3:** carpeta de datos `PixelDND`, clave `~/.pddnd-signing/`, id de la app en Android (cambiarlo la vuelve
  otra app) y prefijo `pdri:`: decidir junto con el nombre nuevo (*Pixelated Experience: Roll Initiative*).
- La decisión vieja "si muere, vuelve con un personaje nuevo" (`decisiones.md`, Multijugador) queda precisada por la
  muerte hardcore con baneo por mundo (C3).

### Siguiente (en este orden, salvo que el dueño diga otra cosa)
1. Lo que falle en la prueba de arriba.
2. Validar el generador propio y borrar el de PD (`level` build, `painters`, `room`, `patch`, `pdri:sewers`).
3. M1d (brújula / minimapa) cuando esté el diseño; después M2 (mapas prefabricados).
4. Planes listos para empezar cuando el dueño lo pida: [P](pasos/7-progresion-5e.md) (personaje 5e),
   [C](pasos/8-personajes-mundos.md) (personajes y mundos), [N6](pasos/9-packs-mods.md) (packs), el resto de
   [L](pasos/10-nombre-avisos.md). C va antes que N6 (los packs guardan personajes y mundos).
5. Lista de mods: ver los importados sin reiniciar (lo único que queda del pedido; N6 rehace esa pantalla).

### Otros
- Rama `pd-core` (0.9.25, `ef30813`): el juego con el core de PD.
- Teléfono: beta con actualizador en canal beta; se actualiza desde las pre-releases de `dev` al pushear.
- Sprites: plan en [`pasos/6-sprites.md`](pasos/6-sprites.md); arte para 5e en [`sprites-5e.md`](sprites-5e.md).
- Repos aparte: editor `../pdri-editor/` y mods `../pdri-mods/` (mod de ejemplo del herrero).

## Pendiente del dueño
- `git push -u origin dev` (hay commits de esta sesión sin pushear; `PROTOCOL` 4: en la LAN todos con 0.9.45). Al
  validar una fase, `merge: dev -> main`.
- Respaldar `~/.pddnd-signing/`.
- ufw: `sudo ufw allow 47810/udp` y `sudo ufw allow 47811/tcp`.
- Opcional: borrar `~/pddnd-android/` y `~/pddnd-backup-20261103-1116/`.
