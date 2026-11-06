# app/src/map.rs

El piso como instancias, port de `DrawLevel` / `BuildFog` de `screens/gameScreen.py`:

- `water`: `water0.png` en mosaico debajo de los tiles, bajando `WATER_SPEED` (5 px/s), recortado al mapa (la fila
  partida se dibuja con un quad más corto y su recorte de uv). Los tiles de agua (48..63) son transparentes.
- `tiles`: un tile por celda; el valor de terreno es el índice en `tiles0.png` (16 por fila). 1024 instancias por
  frame: para la GPU no es nada. ponytail: si algún día pesa, pasar el mapa a un buffer estático que se toca sólo
  cuando cambia una celda.
- `fog_corners` + `fog_quad`: FogOfWar.java. Un color por esquina de celda (visible si las 4 celdas que la tocan se
  ven, recordada si se vieron, revelada por magia, si no negra; el borde del mapa siempre negro) en una textura de
  33x33 con filtro **LINEAR**; el quad cubre el mapa con cada esquina en el centro de su texel, así la GPU interpola
  entre esquinas igual que el original. Reemplaza el armado tile por tile del prototipo.

M1a: el agua, los tiles y la niebla usan el tamaño del nivel. La textura de la niebla es de `FOG_SIZE` = 129 (zonas de
hasta 128x128) y el quad toma sólo la esquina del mapa.
