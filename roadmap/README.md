# Roadmap

Leer esto primero. Un archivo por tema:

| Archivo | Qué tiene |
|---|---|
| [`decisiones.md`](decisiones.md) | Lo que decidió el dueño |
| [`estado.md`](estado.md) | Lo escrito por fase (R0-R6, N1-N5) y qué probar |
| [`pendientes.md`](pendientes.md) | Lo que falta, sin orden |
| [`versiones.md`](versiones.md) | Regla de versiones e historial |
| [`dnd5e.md`](dnd5e.md) | Reglas 5e y equipo |
| [`pasos/`](pasos/) | Próximos pasos, uno por archivo |

## Ahora
Rama `dev`, **0.9.33** (`versionCode` 43). Casi todo está ✏️ (sin probar): lo que falle en la prueba del dueño va
primero.

## Próximos pasos
| # | Paso | Etiqueta |
|---|---|---|
| 1 | ✏️ [Releases en GitHub](pasos/1-releases.md) | `#D1` |
| 2 | ✏️ [Red por internet](pasos/2-red-internet.md) | `#D2` |
| 3 | ✏️ [Compatibilidad entre versiones](pasos/3-compatibilidad.md) | `#D3` |
| 4 | ✏️ [Actualizador](pasos/4-updater.md) | `#D4` |
| 5 | ✏️ M1a, M1b · [Zonas, mapas y editor](pasos/5-mapas-editor.md) | `#M1`… |

| 6 | [Sprites como datos y apariencias por capas](pasos/6-sprites.md) | `#S1`… |

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
- Hecho y sin probar (✏️): D1-D4 (releases, red por internet con el teclado del sistema, compatibilidad, actualizador
  con canal beta: betas `v<versión>-beta-<letra>`) y M1a (tamaño por nivel y zonas `{x, y, z}`, 0.9.26).
- Rama `pd-core` creada en `ef30813` (0.9.25): el juego con el core de PD.
- Teléfono: 0.9.26 beta (con actualizador en canal beta), compilada antes de las zonas: se actualiza desde la
  primera beta que publique la CI al pushear `dev`.
- Fuente: Silkscreen desde 0.9.27 (falta que el dueño la vea: tamaños, que nada se corte).
- Visión, pathfinder y línea de tiro propios desde 0.9.28 (falta que el dueño los pruebe: que los mobs persigan, la
  vista en esquinas y pasillos, disparos y conjuros).
- Configuración por pestañas y BUSCAR AHORA desde 0.9.29 (falta que el dueño la vea, sobre todo en vertical).
- M1b (`world.json`, 0.9.31) y el **generador propio** (`pdri:dungeon`, 0.9.32) hechos, sin probar. Siguiente: que el
  dueño los pruebe; después borrar el generador de PD, y seguir con M1c (salidas por el borde y portales) o M2.
- Repos aparte (el dueño los crea en GitHub): editor `../pdri-editor/` y mods `../pdri-mods/` (ahí está el mod de
  ejemplo del herrero).

## Pendiente del dueño
- `git push -u origin dev`; al validar una fase, `merge: dev -> main`.
- Respaldar `~/.pddnd-signing/`.
- ufw: `sudo ufw allow 47810/udp` y `sudo ufw allow 47811/tcp`.
- Opcional: borrar `~/pddnd-android/` y `~/pddnd-backup-20261103-1116/`.
