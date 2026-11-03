# Assets propios

Todo lo de `Assets/` viene del Pixel Dungeon original salvo lo listado aquí (hecho para este proyecto, a pedido del
dueño). Los archivos propios llevan el prefijo `pddnd_`.

| Archivo | Contenido | Origen |
|---|---|---|
| `Assets/Items/pddnd_items.png` | **Sin uso** (decisión del dueño: nada de sprites generados). Era 0 arco corto, 1 flechas, 2 flecha en vuelo. Hoy el arco y las flechas se dibujan con su nombre hasta que haya sprites hechos por una persona | pixel art generado con pygame |

Sprite que falta: el ítem va sin `image` en `data/items.json` y se dibuja su nombre (`gameUi.TextIcon`). Lo mismo si
una hoja propia o de un mod no carga o el índice no existe. No generar sprites: esperar a uno hecho por una persona.
