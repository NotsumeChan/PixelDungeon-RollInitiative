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
Rama `dev`, **0.9.25** (`versionCode` 35). Casi todo está ✏️ (sin probar): lo que falle en la prueba del dueño va
primero.

## Próximos pasos
| # | Paso | Etiqueta |
|---|---|---|
| 1 | ✏️ [Releases en GitHub](pasos/1-releases.md) | `#D1` |
| 2 | ✏️ [Red por internet](pasos/2-red-internet.md) | `#D2` |
| 3 | ✏️ [Compatibilidad entre versiones](pasos/3-compatibilidad.md) | `#D3` |
| 4 | ✏️ [Actualizador](pasos/4-updater.md) | `#D4` |
| 5 | [Zonas, mapas y editor](pasos/5-mapas-editor.md) | `#M1`… |

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

## Pendiente del dueño
- `git push -u origin dev`; al validar una fase, `merge: dev -> main`.
- Respaldar `~/.pddnd-signing/`.
- ufw: `sudo ufw allow 47810/udp` y `sudo ufw allow 47811/tcp`.
- Opcional: borrar `~/pddnd-android/` y `~/pddnd-backup-20261103-1116/`.
