# S — Sprites como datos y apariencias por capas

Decisiones del dueño (2026-11-06): teñido para los colores; tamaño de cuadro libre por hoja con un máximo; cantidad
de cuadros libre por animación. Todo como datos, igual para el juego base y los mods. Después de M1c / M2 o cuando el
dueño lo pida.

**Hoy:** los mobs ya traen sprite y animaciones en `mobs.json` y las 4 apariencias de PD están en `skins.json`, pero
los PNG van compilados en el binario (`SHEETS` en `app/src/sprites.rs`): un mod no puede traer imágenes.

## S1 — Hojas cargadas al arrancar (también de mods)
- El atlas se arma al iniciar con las hojas del juego y las de los mods activos (`<mod>/sprites/*.png`), no con
  `include_bytes`. `registry::is_mod_file` y el import de zip aceptan `sprites/*.png`.
- Cada hoja con su descripción (`sprites.json`, registro como los demás): `"pdri:rat": {"png": "rat.png",
  "frame": [16, 15], "anims": {"idle": {"frames": [0, 0, 1], "fps": 2}, "run": {...}, "attack": {...},
  "die": {...}, "operate": {...}}}`. Cuadros por animación y fps libres.
- **Máximo por cuadro: 64x64** (`MAX_FRAME`; se puede cambiar). Una hoja más grande o mal descrita no carga: se
  dibuja el nombre, como hoy sin sprite.
- `mobs.json` y `skins.json` pasan a nombrar la hoja por id (`"sprite": "pdri:rat"`). Las de PD siguen igual hasta S4.

## S2 — Apariencias por capas (teñidas)
- Capas en orden: cuerpo → ropa base → pelo (→ armadura y arma en S3). Todas las capas de un cuerpo comparten tamaño
  de cuadro y animaciones (el cuerpo manda; una capa que no coincide no carga).
- Cada capa en escala de grises; se dibuja multiplicada por su color (la GPU ya tiñe cada sprite). Partes sin color
  (detalles, ojos) van en una capa sin teñir.
- Datos (`appearance.json`): partes por ranura (`body`, `outfit`, `hair`...) con su hoja, si se tiñe y su paleta
  sugerida (o color libre). Los mods suman partes y paletas.
- El personaje guarda su apariencia: `{"body": id, "skin": color, "hair": id, "hair_color": color, "outfit": id,
  "outfit_color": color}`. Viaja en el `hello` de la red (sube `PROTOCOL`) y se dibuja igual en todos.
- Creación de personaje: un selector por ranura y por color, con vista previa animada.
- Las apariencias enteras de hoy (los 4 héroes de PD) siguen como una sola capa sin teñir.

## S3 — Equipo visible
`"look"` en `items.json`: la armadura y el arma suman su capa (teñible) sobre el cuerpo.

## S4 — Arte
Cuerpo base, peinados, ropas y equipo, hechos por personas (`docs/assets.md`), con la grilla de S2. Recién ahí se
reemplazan los héroes de PD (parte de separarse de PD).
