# screens/homeScreen.py

Pantalla de título: port de `scenes/TitleScene.java` + `ui/Archs.java` + `effects/BannerSprites.java`.

## Qué dibuja (en orden)
1. **Arcos** (`arcs1.png` atrás, `arcs2.png` adelante): texturas repetidas que suben a 20 y 40 px/s.
   `DrawTiled` replica el centrado horizontal de `Archs.layout`.
2. **Título**: `banners.png`, recorte `(0, 0, 128, 70)`.
3. **Letreros**: `banners.png`, recorte `(0, 161, 128, 57)`, sumados con blend aditivo. Su alpha es `sin(-t)` y
   queda invisible mientras es negativo, igual que en Java, así que aparecen unos 3 s después de abrir.
4. **Dashboard**: 3 botones de 48x48: Play, Mods y About. Rankings y Badges del original se sacaron.
   Íconos de `dashboard.png` (0 Play, 1 About; Mods usa el 2, el de Rankings).
5. **Versión** abajo a la derecha, en gris `0x888888` (de `core/Version.py`).

## Pendiente
- Play abre la lista de personajes (`screens/rosterScreen.py`); Mods abre `screens/modsScreen.py`.
- About abre `screens/aboutScreen.py`. Rankings y Badges todavía no tienen pantalla (`action = None`).
- Antorchas (`Fireball` junto al título), botón de opciones y de salir.
- Fade in.
- Música (`theme.mp3`): al final, junto con el resto del audio.
- Layout vertical (retrato): no aplica en PC.

## Configuración
- `prefs`: ícono `Icons.PREFS` (`icons.png (30, 0, 16, 16)`) arriba a la izquierda, como `TitleScene`: abre la
  ventana de configuración (`gameUi.DrawSettings`). Mientras está abierta, los clicks van a ella; Esc la cierra.
- `BuildButtons()`: los textos de los botones salen del idioma (Jugar / Play, Acerca de / About) y se rearman si cambia.
