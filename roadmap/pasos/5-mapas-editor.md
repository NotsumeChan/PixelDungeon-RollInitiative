# M — Zonas, mapas prefabricados y editor

Va después de D4. Es grande: al llegar, partir cada punto en pasos chicos que se prueben por separado.

## M1 — Mundo por zonas
Decisiones en [`decisiones.md`](../decisiones.md#mundo-por-zonas-2026-11-06). Cada paso, una versión que se prueba.
- ✏️ **M1a — Coordenadas y tamaño variable** (0.9.26, falta la prueba del dueño: jugar como siempre, bajar y subir, guardar y volver; una partida vieja no aparece): la clave de los pisos pasa de `depth` a `Zone {x, y, z}` y cada nivel
  tiene su ancho y alto (fuera las constantes `WIDTH`/`HEIGHT`/`LENGTH`). La seed de cada zona sale de sus
  coordenadas. Las partidas viejas no se cargan; sube `PROTOCOL`. El juego se ve igual: 4 zonas (0,0,1)..(0,0,4).
- ✏️ **M1b — `world.json`** (0.9.31, falta la prueba del dueño: bajar y subir, el nombre de la zona al entrar, el fin del piso 4): el mundo como datos (zonas, nombre, mapa generado o prefabricado, conexiones). El base
  reproduce las Alcantarillas de hoy. `docs/data.md` y la API de mods estable.
- **M1c — Salidas con dirección:** por el borde del mapa (norte, sur, este, oeste) y casillas especiales (puertas,
  portales), además de las escaleras.
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
