# M — Zonas, mapas prefabricados y editor

Va después de D4. Es grande: al llegar, partir cada punto en pasos chicos que se prueben por separado.

## M1 — Mundo por zonas
Decisiones en [`decisiones.md`](../decisiones.md#mundo-por-zonas-2026-11-06). Cada paso, una versión que se prueba.
- ✏️ **M1a — Coordenadas y tamaño variable** (0.9.26, falta la prueba del dueño: jugar como siempre, bajar y subir, guardar y volver; una partida vieja no aparece): la clave de los pisos pasa de `depth` a `Zone {x, y, z}` y cada nivel
  tiene su ancho y alto (fuera las constantes `WIDTH`/`HEIGHT`/`LENGTH`). La seed de cada zona sale de sus
  coordenadas. Las partidas viejas no se cargan; sube `PROTOCOL`. El juego se ve igual: 4 zonas (0,0,1)..(0,0,4).
- ✏️ **M1b — `world.json`** (0.9.31, falta la prueba del dueño: bajar y subir, el nombre de la zona al entrar, el fin del piso 4): el mundo como datos (zonas, nombre, mapa generado o prefabricado, conexiones). El base
  reproduce las Alcantarillas de hoy. `docs/data.md` y la API de mods estable.
- **M1c — Salidas con dirección** (lo siguiente). Plan:
  - `world.json`: además de `up` / `down`, cada zona puede tener `north`, `south`, `east`, `west` (salida por el
    borde) y `portals`: `[{"id": "cripta", "to": "x,y,z"}]` (casilla especial: puerta, portal).
  - **Borde:** el generador deja una abertura en ese borde (un pasillo desde la sala más cercana hasta el borde, de
    1 casilla). Pisar la casilla del borde pasa a la zona vecina, sin confirmar (como caminar), y se aparece en la
    abertura del borde opuesto de la otra zona (`north` <-> `south`, `east` <-> `west`). Si la otra zona no tiene esa
    abertura, se aparece en su entrada.
  - **Portal:** el generador lo pone en una sala Standard (tile de puerta o portal; sprite en `roadmap/sprites-5e.md`).
    Se usa con click / toque, como las escaleras, y se aparece en el portal de la otra zona con el mismo `id` (o en
    su entrada).
  - `Level`: `exits: Vec<(celda, Exit)>` (`Exit::Edge(dir)` / `Exit::Portal(id)`), armado por el generador a partir
    de la entrada de `world.json`. `build_flag_maps` deja transitables las casillas de salida del borde.
  - `Dungeon`: `edge_exit(i, dir)` y `portal(i, id)` como `descend` / `ascend`; `go_to` recibe dónde aparecer.
    `Hero::handle`: pisar una salida del borde = cambiar de zona; click en un portal = usarlo.
  - Red: las acciones nuevas validadas como las escaleras; el estado ya va por zona (no cambia `PROTOCOL` salvo que
    cambie el formato). El mundo base no cambia (sigue siendo una pila); un mod o el editor arman mundos con bordes.
  - Para probar: un mod chico con dos zonas lado a lado (`0,0,1` con `east`, `1,0,1` con `west`) y un portal.
- **M1d — Nombre de zona y brújula:** al entrar y en la barra de estado; minimapa o brújula del mundo.

## M2 — Mapas prefabricados
- Layout hecho a mano en vez del generador: terreno, salas, salidas, y mobs, ítems clave y NPC fijos.
- El resto de los ítems, al azar salvo que el mapa diga otra cosa.
- Como datos dentro de un mod (`docs/data.md`).

## M3 — Editor
- Repo aparte: `../pdri-editor/` (su roadmap está ahí). Sólo PC; usa `game/` y el render de `app/` de este repo como
  dependencias. Lo que pida de este repo (render reusable, formato de mapas) se hace acá con su etiqueta `#M3`.
- Pintar terreno, poner salas, salidas, mobs, ítems y NPC; nombrar y conectar zonas.
- Exporta un mod de sólo datos. Más adelante: formularios de clases e ítems.
