# L — Nombre propio y avisos legales

Decisiones del dueño (2026-11-06), en [`decisiones.md`](../decisiones.md#nombre-y-avisos-2026-11-06). Parte del
cierre de la separación de Pixel Dungeon (`pendientes.md`). Sólo plan.

## Nombres
- Juego: **Pixelated Experience: Roll Initiative**. Editor: **Pixelated Editor: Roll Imagination**.
- Se cambian cuando no quede nada de PD (`Assets/`, código, nombre). Hasta entonces, el de hoy.

## Cómo se habla de las reglas
- Nunca decir que el juego está basado en D&D ni nombrar *Dungeons & Dragons* / *D&D* para describirlo.
- Sí: "altamente compatible con la 5.ª edición de juegos de rol" ("highly compatible with 5th edition roleplaying").
- Las referencias a Wizards of the Coast salen de todo el proyecto **salvo la atribución del SRD**: mientras se use
  material del SRD 5.1 / 5.2 (reglas, monstruos, conjuros, textos), su licencia CC-BY-4.0 obliga a dar el crédito tal
  como pide (nombra a Wizards of the Coast LLC). Revisar el texto de la licencia antes de publicar (no somos abogados).

## ✏️ L1 — Textos (0.9.40, falta que el dueño lo revise: README y Acerca de, en horizontal y vertical)
- README, Acerca de (en el juego), textos de la UI y de `data/`: sacar "D&D" / "Dungeons & Dragons"; usar "5.ª
  edición" / "5e" donde haga falta. ~25 archivos hoy (README, `ui.rs`, `lib.rs`, `dice.rs`, `spells.rs`, `hero.rs`,
  docs, roadmap).
- Avisos claros en el README y en Acerca de: proyecto no afiliado (a Watabou mientras quede algo de PD; a Wizards of
  the Coast), compatible con la 5.ª edición, la atribución del SRD (CC-BY-4.0) y la licencia GPL v3.

## L2 — Nombres internos (✏️ en parte, 0.9.43: `class_5e` y `reglas5e.md`)
- ✏️ `data/classes.json`: el campo `dnd_class` pasa a `class_5e`; la API de mods es estable: el viejo se sigue
  leyendo como alias (`docs/mods.md`).
- ✏️ `roadmap/dnd5e.md` -> `reglas5e.md`. Falta (afecta instalaciones, decidir junto con el nombre): carpeta de datos `PixelDND`, clave `~/.pddnd-signing/`, CI y `android/build.sh`.
- Ojo Android: cambiar el id de la app (package) la vuelve otra app (no actualiza la instalada, otra clave de datos):
  decidirlo una sola vez, junto con el nombre nuevo.

## L3 — El cambio de nombre
- Título, ventana, APK, releases, README, repos (juego y `pdri-editor`), prefijo de ids `pdri:` (se queda: es interno
  y lo usan los mods; decidir si se cambia).
