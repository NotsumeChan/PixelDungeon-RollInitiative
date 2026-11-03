# Fase 2 — Render del mapa

## Estado
- ✅ Pantalla de título con assets originales (`screens/homeScreen.py`).
- ✅ Resolución virtual con zoom entero (`core/Screen.py`), font original (`core/Font.py`).
- ✅ Tilemap + capa de agua en `screens/gameScreen.py`; R = nivel nuevo.
- ✅ FOV (`game/fov.py`, port de ShadowCaster) + `level.observe` / `visible` / `visited` / `mapped`.
- ✅ Niebla de guerra suavizada (FogOfWar) y cámara que sigue al observador.
- ✅ Observador (adelanto del héroe) que camina con WASD y abre puertas. Lo reemplaza el héroe en la fase 3.
- Pendiente de revisión manual del dueño.
- Corregido: la niebla se corría hasta medio tile hacia los bordes del mapa (se veía detrás de paredes simples).
  `smoothscale` sobre el mapa entero no reparte los píxeles como el filtro LINEAR del original; ahora se arma tile por
  tile (`FogTile`, degradado entre las 4 esquinas) y ya no hace falta `FOG_OFFSET`.

Objetivo: `screens/gameScreen.py` dibuja el nivel de `game/` con los tiles originales, cámara y niebla.

## Assets
- Pisos 1–4: `Assets/Tiles/tiles0.png` (tiles de 16x16) y `Assets/Tiles/water0.png`.
- En el original (`DungeonTilemap.java`, `scenes/GameScene.java`) el índice de tile sale del valor de
  `Terrain` -> las constantes de `game/terrain.py` deben conservar los valores Java. Verificar al portar.
- El agua es una capa aparte (`water0.png`) debajo del tilemap; las celdas WATER del tilemap son transparentes.

## Piezas
| Pieza | Referencia Java | Destino |
|---|---|---|
| Tilemap (cortar `tiles0.png`, dibujar map) | `DungeonTilemap.java` | render |
| Capa de agua | `GameScene` (`water`) | render |
| Cámara centrada en el héroe, zoom entero (pixel art) | `scenes/GameScene.java`, `noosa/Camera` | render |
| FOV | `mechanics/ShadowCaster.java` (153 líneas) | `game/fov.py` |
| Niebla: visto / recordado / no visto | `FogOfWar.java`, `Level.visited/mapped` | render lee `game/` |

## Notas
- Dibujar en un `pg.Surface` y escalar con zoom entero; `core/Screen` ya sube el canvas a una textura OpenGL con filtro NEAREST.
- Cortar los tiles una sola vez al cargar, no por frame.
- Check: abrir el juego con una seed fija y comparar con el ASCII de fase 1.
