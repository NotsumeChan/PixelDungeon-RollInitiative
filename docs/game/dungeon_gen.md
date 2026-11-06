# game/src/dungeon_gen.rs

Generador propio de zonas (`"gen": "pdri:dungeon"` en `world.json`; 0.9.32), sin código de PD:
1. Salas rectangulares al azar (interior 4-8 x 4-7, ~1 cada 100 casillas, entre 3 y 16) que no se tocan.
2. Qué se une: un árbol (cada sala nueva con la más cercana ya unida) y ~1 de cada 4 salas con otra más (vueltas).
3. Cada unión: una puerta en la pared que mira a la otra sala y un pasillo por fuera de las salas (BFS en 4
   direcciones, sin tocar el borde del mapa). Si no hay camino, esa unión se saltea.
4. Entrada en la primera sala; salida en la más lejana a pie. Si alguna sala no se alcanza, se reintenta con el rng
   como quedó (`build` devuelve false; `Level::generate` limpia el mapa y vuelve a probar).
5. Agua y pasto: ruido suavizado dos veces (queda si 5 o más del vecindario 3x3). Trampas ocultas desde z = 2.
6. Salidas de `world.json` (M1c, 0.9.34; sin ellas no se gasta rng: el mundo base sale igual). Por cada borde con
   salida, un pasillo recto desde la sala más cercana a ese borde hasta la casilla de al lado del borde (esa casilla
   es la salida; el borde sigue de pared para que ninguna vecina caiga fuera del mapa). Cada portal, en una casilla
   libre de una sala Standard, como `EMPTY_SP` hasta que haya sprite. Quedan en `Level::exits`.
Las salas quedan en `Level::rooms` (Entrada, Salida, el resto Standard) para dónde aparecen mobs, ítems y NPC.
Usa los tiles de siempre. Revisar a mano: `cargo run -q -p game --example level -- <seed> <piso>`.
