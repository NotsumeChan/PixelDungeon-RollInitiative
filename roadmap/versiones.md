# Versiones

**Rust** (rama `main` / `dev`): la versión vive en el `Cargo.toml` del workspace (`game::VERSION` la lee de ahí) y en
`android/AndroidManifest.xml` (`versionName`, y `versionCode` sube de a uno o más, nunca baja). Regla: la versión de
Rust es la del prototipo cuyo contenido ya está portado completo. Al cerrar un paso del roadmap que la cambie, subirla
en los dos lugares y en la tabla de Rust.

| Versión (Rust) | Pasos | Contenido |
|---|---|---|
| 0.1.0 | R0, R1 | Workspace, ventana y GPU, APK firmado, CI; generación de las Alcantarillas idéntica al prototipo |
| 0.2.0 | R2 | Render del mapa: tiles, agua, niebla, cámara, zoom, orientación vertical |
| 0.3.0 | R3 | Héroe, turnos y rondas 5e por iniciativa, mobs con IA, combate 5e, salvaciones contra la muerte, pisos 1-4 |
| 0.4.0 | R4, R5a | Personajes animados, dados animados, UI original (estado, iniciativa, barra, examinar, ficha, menú); ítems, mochila, accesos rápidos, pociones y pergaminos, hambre |
| 0.5.0 | R5 | Creación de personaje con dados (4d6, 7 tiradas, asignar o escribir), lista de personajes y guardado; ya estaban el combate 5e y la ficha |

(Las 0.2 y 0.3 no se marcaron en su momento: el `Cargo.toml` pasó de 0.1.0 a 0.4.0.)

**Prototipo en Python** (rama `python-prototipe`): la versión vivía en `core/Version.py`. **1.0 = multijugador P2P
listo.**

| Versión | Contenido |
|---|---|
| 0.1.0 | Base que arranca (fase 0) + generación de niveles de las Alcantarillas (fase 1) |
| 0.2.0 | Render: tiles, agua, campo de visión, niebla de guerra, cámara (fase 2) |
| 0.3.0 | Héroe, turnos, combate, mobs con IA, pisos 1–4 (fase 3) |
| 0.4.0 | Ítems, inventario, hambre, UI original, pantalla About (fase 4) |
| 0.5.0 | Capa D&D 5e: creación de personaje con dados, combate 5e, ficha |
| 0.6.0 | Mejoras: animación de dados, lista de personajes, drag and drop, pantalla completa, mochila, legibilidad, requisitos de FUE |
| 0.6.1 | El daño espera al dado, log de tiradas como opción de desarrollador, maximizar ajusta el contenido, arrastrar el mapa, Mods en el título, 2 accesos rápidos |
| 0.6.2 | Lista de personajes: JUGAR y BORRAR en gris si no hay personajes |
| 0.7.0 | Rasgos de clase y conjuros (Segundo Aliento, Ataque Furtivo, Mago, Explorador), salvaciones, trampas, estados como datos, armas a distancia (arrojadizas, bumerán, arco), subir de piso, menú y configuración (idioma, FPS, pantalla completa), examinar, animaciones de personajes y efectos, CA de mobs más baja, arreglos de niebla y de dt |
| 0.8.0 | Contenido en JSON (`data/`, con `"base"`), guardado de partidas por personaje, decisiones del multijugador, tooltips, accesos rápidos en orden, zoom de cámara nítido (rueda / pellizco), escala de la interfaz, vista recalculada al abrir puertas |

## Plan hacia la 1.0
| Versión | Objetivo |
|---|---|
| 0.7 | ✅ Rasgos de clase 5e, salvaciones, estados. La segunda pasada de ítems pasó a pendientes (`roadmap/mejoras.md`) |
| 0.8 | ✅ Guardado de partidas por personaje, registros en JSON (base para mods) |
| 0.9 | ⏳ Red: descubrimiento LAN + partida para 2 jugadores (plan: `roadmap/fase5-red.md`) |
| **1.0** | **Multijugador P2P estable** |
| 1.x | Mods (`.pdmod`, instancias, sincronización estilo tModLoader), mapas por zonas, Android, internet (relay y servidores propios, no sólo PC) |
