# M — Zonas, mapas prefabricados y editor

Va después de D4. Es grande: al llegar, partir cada punto en pasos chicos que se prueben por separado.

## M1 — Mundo por zonas
- De pila de pisos a **zonas conectadas** en un grafo, con coordenadas `x, y, z` y salidas en varias direcciones.
- `descend()` pasa a "ir a la zona de esta salida"; las visitadas se guardan como hoy los pisos.
- La seed de cada zona sale de sus coordenadas (cuidar determinismo y guardados).
- Nombre de zona al entrar y en la barra; brújula o minimapa.

## M2 — Mapas prefabricados
- Layout hecho a mano en vez del generador: terreno, salas, salidas, y mobs, ítems clave y NPC fijos.
- El resto de los ítems, al azar salvo que el mapa diga otra cosa.
- Como datos dentro de un mod (`docs/data.md`).

## M3 — Editor
- Repo aparte: `../pdri-editor/` (su roadmap está ahí). Sólo PC; usa `game/` y el render de `app/` de este repo como
  dependencias. Lo que pida de este repo (render reusable, formato de mapas) se hace acá con su etiqueta `#M3`.
- Pintar terreno, poner salas, salidas, mobs, ítems y NPC; nombrar y conectar zonas.
- Exporta un mod de sólo datos. Más adelante: formularios de clases e ítems.
