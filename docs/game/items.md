# game/src/items.rs

`game/items.py` (items/*.java + Generator.java + Heap.java + ItemStatusHandler). El contenido es `data/items.json`.

- `Item`: id, datos, cantidad, nivel, maldito y qué se sabe (nivel, maldición). `split` saca n de una pila.
- `weapon_dice` (dados + nivel: "1d8+1"), `armor_ac` (sin armadura 10 + DES; con armadura `ac` + DES hasta
  `dex_max` + nivel).
- `Generator` (estado por partida, se reinicia por zona): categoría por pesos que se dividen a la mitad al salir;
  equipo con 40% de tener nivel (la mitad negativos y malditos) y, de dos, el de FUE más cercana a la del héroe;
  arrojadizas y flechas en montón (`qty`). Mismas tiradas que el prototipo.
- `Heap` (`HeapKind`: montón, cofre, restos, tumba): el de arriba es `items[0]`; los apilables se juntan.
- Apariencias sin identificar (`POTION_LOOKS`, `SCROLL_LOOKS`): `Dungeon::new` las mezcla con el rng de la partida
  (lo primero que lo gasta, como el prototipo).
- En `Dungeon` (lo que sabe el jugador): `is_known`, `item_name` (poción "turquesa", pergamino "KAUNAN", nivel si se
  conoce), `item_image` (índice de items.png o `Image::Text` sin sprite: se dibuja el nombre), `heap_image`,
  `item_info` (la ventana del ítem, con los números de 5e).
