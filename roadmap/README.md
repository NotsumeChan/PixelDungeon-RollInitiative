# Roadmap

Leer esto primero. Un archivo por tema:

| Archivo | Qué tiene |
|---|---|
| [`decisiones.md`](decisiones.md) | Lo que decidió el dueño |
| [`estado.md`](estado.md) | Lo escrito por fase (R0-R6, N1-N5) y qué probar |
| [`pendientes.md`](pendientes.md) | Lo que falta, sin orden |
| [`versiones.md`](versiones.md) | Regla de versiones e historial |
| [`dnd5e.md`](dnd5e.md) | Reglas 5e y equipo |
| [`sprites-5e.md`](sprites-5e.md) | El arte que hace falta para 5e (personajes, equipo, mobs, interfaz) |
| [`pasos/`](pasos/) | Próximos pasos, uno por archivo |

## Ahora
Rama `dev`, **0.9.41** (`versionCode` 51). Casi todo está ✏️ (sin probar): lo que falle en la prueba del dueño va
primero.

## Próximos pasos
| # | Paso | Etiqueta |
|---|---|---|
| 1 | ✏️ [Releases en GitHub](pasos/1-releases.md) | `#D1` |
| 2 | ✏️ [Red por internet](pasos/2-red-internet.md) | `#D2` |
| 3 | ✏️ [Compatibilidad entre versiones](pasos/3-compatibilidad.md) | `#D3` |
| 4 | ✏️ [Actualizador](pasos/4-updater.md) | `#D4` |
| 5 | ✏️ M1a, M1b, M1c · [Zonas, mapas y editor](pasos/5-mapas-editor.md) | `#M1`… |

| 6 | [Sprites como datos y apariencias por capas](pasos/6-sprites.md) | `#S1`… |
| 7 | [Personaje 5e: creación y progresión](pasos/7-progresion-5e.md) | `#P1`… |
| 8 | [Personajes y mundos (estilo Terraria)](pasos/8-personajes-mundos.md) | `#C1`… |
| 9 | [Paquetes de mods](pasos/9-packs-mods.md) | `#N6`… |
| 10 | ✏️ L1 · [Nombre propio y avisos legales](pasos/10-nombre-avisos.md) | `#L1`… |

Estética: en conversación, sin paso.

**Meta a largo plazo:** separarse por completo de Pixel Dungeon (ver [`decisiones.md`](decisiones.md#meta-un-juego-propio-2026-11-06)).
Lo nuevo nace propio; lo de PD se reemplaza de a poco.

## Cómo se trabaja
- Sólo compilar (`cargo clippy --workspace --all-targets`, `cargo ndk -t arm64-v8a check -p app --lib`) y tests de
  red (`cargo test -p app --lib net`). Nada de correr el juego ni simulaciones: el dueño prueba a mano.
- Commits en `dev`, a nombre del dueño, sin coautoría, con la etiqueta del paso. Cada cambio sube la versión.
- Al terminar algo: su "Para probar" en `estado.md` y esta tabla al día.
- Referencias: prototipo en la rama `python-prototipe` (con sus planes viejos en `roadmap/`), Java original en
  `~/Documents/programacion/personal/java/pixel-dungeon/`.

## Estado al cerrar la sesión (2026-11-06)
**Para retomar:** leer esta sección y el plan de M1c en [`pasos/5-mapas-editor.md`](pasos/5-mapas-editor.md). No hay
nada a medio hacer: todo está en commits en `dev` (0.9.41). El teléfono tiene una beta con el actualizador (canal
beta): se actualiza solo desde las pre-releases de `dev`. Notas locales (no del proyecto) en `NOTAS.md`.

- Hecho y sin probar (✏️): D1-D4 (releases, red por internet con el teclado del sistema, compatibilidad, actualizador
  con canal beta: betas `v<versión>-beta-<letra>`) y M1a (tamaño por nivel y zonas `{x, y, z}`, 0.9.26).
- Rama `pd-core` creada en `ef30813` (0.9.25): el juego con el core de PD.
- Teléfono: 0.9.26 beta (con actualizador en canal beta), compilada antes de las zonas: se actualiza desde la
  primera beta que publique la CI al pushear `dev`.
- Fuente: Silkscreen desde 0.9.27 (falta que el dueño la vea: tamaños, que nada se corte).
- Visión, pathfinder y línea de tiro propios desde 0.9.28 (falta que el dueño los pruebe: que los mobs persigan, la
  vista en esquinas y pasillos, disparos y conjuros).
- Configuración por pestañas y BUSCAR AHORA desde 0.9.29 (falta que el dueño la vea, sobre todo en vertical).
- M1b (`world.json`, 0.9.31) y el **generador propio** (`pdri:dungeon`, 0.9.32) hechos, sin probar. Configuración
  por pestañas, sin F11 ni controles en Android e idioma como lista (0.9.33).
- M1c (salidas por el borde y portales, 0.9.34) hecho, sin probar: cómo probarlo en
  [`pasos/5-mapas-editor.md`](pasos/5-mapas-editor.md) (hace falta un mod con dos zonas: ejemplo en `docs/data.md`).
- Turnos fuera de combate propios (0.9.35), sin probar. Para probar en LAN (o hotseat): con un jugador quieto, el
  otro camina sin esperarlo (los mobs también se mueven); los dos quietos = se espera. Caminar hacia el otro lado de
  un aliado quieto en un pasillo: se cambian de lugar. Dos caminando de frente por un pasillo: se cruzan. Uno detrás
  del otro en la misma dirección: el de atrás lo sigue sin cortarse. En combate, todo como antes.
- Pathfinder sin zigzag (0.9.36), sin probar: caminar en recta por una sala abierta y en diagonal suave hacia un
  destino en ángulo; los mobs también.
- Tocarse (0.9.37), sin probar: parado en la escalera (arriba y abajo), en un portal o en el borde abierto, tocarse
  la usa; sobre un montón lo recoge; sin nada abajo, busca.
- Acción rápida, Seguir y apuntar arrojadizas (0.9.38), sin probar. Botones arriba de la barra, a la derecha, cuando
  el héroe espera input: con un mob a la vista, su retrato (tocar = atacarlo; con arco, dispara); sin mobs y con un
  ítem a la vista, su ícono (recogerlo, o "Abrir" un cofre / tumba). Seguir: click lejos, que aparezca un mob o
  encontrar algo oculto corta la caminata, Seguir (celeste, arriba a la derecha, sólo fuera de combate) la retoma;
  al llegar desaparece. Arrojadiza en un acceso rápido:
  tocarlo dice a quién apunta; tocarlo otra vez la lanza al más cerca (tocar una casilla sigue funcionando). En LAN
  también. Revisar en vertical que los botones no tapen el log.
- Combate (0.9.39), sin probar: el carrusel sólo con los que pelean o se ven (sin "?" de mobs lejanos); al matar
  al último, "Fin del combate" en el momento y "Experiencia del combate: +N" (durante la pelea no sube la XP). Un
  mob que llega a mitad de pelea se suma al final de la ronda. Guardar y salir a mitad de una pelea, volver y
  terminarla: el resumen suma también lo ganado antes de guardar (0.9.41).
- L1 (0.9.40), sin probar: leer el README (sección Avisos) y Acerca de en horizontal y en vertical (que entre todo).
- **Siguiente: M1d** (nombre de zona y brújula). Después: borrar el generador de PD cuando el dueño valide el propio,
  y M2 (mapas prefabricados).
- Sprites: plan en [`pasos/6-sprites.md`](pasos/6-sprites.md) y la lista del arte para 5e en
  [`sprites-5e.md`](sprites-5e.md).
- Repos aparte (el dueño los crea en GitHub): editor `../pdri-editor/` y mods `../pdri-mods/` (ahí está el mod de
  ejemplo del herrero).

## Pendiente del dueño
- `git push -u origin dev`; al validar una fase, `merge: dev -> main`.
- Respaldar `~/.pddnd-signing/`.
- ufw: `sudo ufw allow 47810/udp` y `sudo ufw allow 47811/tcp`.
- Opcional: borrar `~/pddnd-android/` y `~/pddnd-backup-20261103-1116/`.
