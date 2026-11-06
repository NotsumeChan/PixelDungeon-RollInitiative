# Decisiones del dueño

No se vuelven a discutir salvo que el dueño lo pida.

## Meta: un juego propio (2026-11-06)
- A largo plazo, **separarse por completo de Pixel Dungeon**: mobs e ítems del SRD 5.2 (CC-BY 4.0), generación,
  código y UI propios, arte y sonido hechos por personas. Caro y lento, sin apuro: lo nuevo nace propio y lo viejo se
  reemplaza de a poco. Pasos en [`pendientes.md`](pendientes.md#separarse-de-pixel-dungeon).
- **Nombre:** sigue siendo PixelDungeon-RollInitiative mientras quede algo de PD. El nuevo se elige al separarse
  (no "Roll Initiative" solo: es marca registrada).
- **Rama `pd-core`:** creada el 2026-11-06 en `ef30813` (0.9.25, después de D1-D4): el juego con el core de PD, para
  quien le interese. Desde ahí, `dev` empieza a separarse (camino al editor).
- Libre y gratis en todas las plataformas, y el agradecimiento a Pixel Dungeon queda en los créditos aunque ya no sea obligatorio.
- **Fuente:** al separarse, Silkscreen (OFL) en vez de la de PD (2026-11-06). Hecho en 0.9.27.
- **Música:** la de PD (Cube_Code) no va en el juego: será un mod opcional instalable cuando haya audio; se le
  agradece en los créditos (2026-11-06).
- Mientras quede algo de PD, el juego es GPL v3; al separarse, la licencia del código nuevo se decide entonces.

## Distribución y red (2026-11-06)
- **Releases:** cada merge a `main` publica `v<versión>` con Linux, Windows y el APK firmado. → [D1](pasos/1-releases.md)
- **Datos al desinstalar:** Android con `hasFragileUserData` (casilla "Conservar datos"). Linux y Windows ya los
  conservan. Instalador de Windows, después.
- **Internet:** dirección a mano (sirve para playit.gg, ngrok, Tailscale; ZeroTier ya anda) + contraseña opcional.
  Relay y servidor dedicado, después. → [D2](pasos/2-red-internet.md)
- **Compatibilidad:** por versión de protocolo + hash del contenido. El cliente actualiza a la versión **del host**,
  no a la última. Android no baja de versión. → [D3](pasos/3-compatibilidad.md)
- **Actualizador:** feature `updater`, sólo builds de GitHub o propias (nunca Play ni Steam). Avisa y descarga, sólo
  estables, apagable en Configuración (prendido por defecto). Canal beta sólo para testers (2026-11-06): builds
  compiladas con `PDRI_CHANNEL=beta`, que reciben las pre-releases que la CI publica desde `dev`. → [D4](pasos/4-updater.md)
- **Steam:** primero sin SDK (como Shattered PD). Amigos y logros: a futuro, sin implementar (revisar licencia).
- **Play Store:** sólo publicar, sin Play Games; la clave del dueño en Play App Signing.
- **Orden:** releases → red → compatibilidad → actualizador → mapas y editor.

## Mapas y editor (2026-11-06)
Editor como aplicación aparte (sólo PC), en **su propio repo** (`../pdri-editor/`), que usa `game/` y `app/` de
este como dependencias.
**Precio:** el editor es GPL (usa `game/` y `app/`): código siempre público; la versión compilada a 1-2 USD en
itch.io primero ("paga lo que quieras", mínimo 1), Steam después si vale el alta (100 USD). Se paga comodidad y apoyo,
no acceso: cualquiera puede compilarlo o compartirlo. Lo que se crea con él es gratis para los jugadores. Primero mapas prefabricados, con el mundo por zonas desde el inicio. Después
del actualizador. → [M](pasos/5-mapas-editor.md)

## Mundo por zonas (2026-11-06)
- **Tamaño variable** por zona (ancho y alto propios), desde M1a.
- **Salidas:** las dos formas. Por el borde del mapa (caminar hasta el borde pasa a la zona vecina, para exteriores) y
  casillas especiales como las escaleras (puertas, portales, para interiores). El editor elige.
- **Mundo sólo de datos:** qué zonas existen y cómo se conectan sale exactamente de `world.json` (base o de un mod).
  El mapa de cada zona sí puede ser prefabricado o generado con la seed (p. ej. las Alcantarillas), según diga el JSON.
- **Guardados:** las partidas viejas no se cargan (los personajes sí quedan). Sube `PROTOCOL`.

## Sprites y apariencias (2026-11-06)
Sprites como datos, también desde mods; apariencias por capas teñidas (piel, pelo, ropa); tamaño de cuadro libre
por hoja con un máximo (64x64); cuadros y velocidad libres por animación. Plan en [S](pasos/6-sprites.md).

## NPCs (2026-11-05)
1. Contenido inicial: Fabricante de varitas y Mercader; el Fantasma, no.
2. Misiones, flags y actitud son de la partida (compartidas en la LAN); las recompensas, del héroe.
3. Tiradas sociales con pericias 5e.
4. Atacar a un NPC lo vuelve hostil y los que lo ven te tratan peor.

## Mods
- Sin tienda: se consiguen por fuera. Packs estilo Modrinth; al unirse, los mods del host estilo tModLoader.
- Obligatorios (se compara el hash) y de cliente (gráficos, sonido).
- Mods con código (Lua): advertencia fuerte y confirmación. Sólo datos: aviso general.
- Plataformas: Android, Windows, Linux. Sin iOS ni Mac.
- **Repo de mods** (2026-11-06): los mods que hace el equipo van en un tercer repo (`../pdri-mods/`), GPL v3. En el
  juego sólo quedan los incluidos (`data/mods/`, como el mercader ambulante).
- **API de mods estable** (2026-11-06): lo que usan los modders (formatos de `data/` y `mod.json` en `docs/data.md` y
  `docs/mods.md`, y el `ctx` de Lua) cambia lo mínimo y sólo sumando: campos y funciones nuevas, nunca quitar ni
  cambiar el sentido de lo que hay. Si alguna vez hay que romper algo: `api` en `mod.json`, el juego sigue cargando
  la versión anterior y el cambio queda anotado en `docs/mods.md`.

## Personajes y mundos (2026-11-06)
- **Personaje separado del mundo** (estilo Terraria): el personaje lleva su identidad y su progreso (nivel, XP,
  inventario, equipo, el uso de armas y armaduras); el mundo, su mapa, mobs, misiones, reglas y dónde quedó cada
  personaje en él. Plan: [`pasos/8-personajes-mundos.md`](pasos/8-personajes-mundos.md).
- **Lista de mundos aparte**: se elige personaje y después mundo.
- **Muerte siempre hardcore**: el personaje muere para siempre y queda baneado de ese mundo; para seguir ahí, otro
  personaje. Los DM suelen hacer excepciones: eso va por las reglas del mundo y la idea del DM (abierta).
- **Reglas del mundo**: al generar un mundo o usar uno prefabricado (one-shot, historia larga) se eligen modificadores
  generales; pueden cambiar a lo largo de la partida, pero sólo saliendo, cambiando la configuración y volviendo a
  entrar.
- Duplicar y editar personajes (nombre, apariencia).
- **Mods**: primero packs como perfiles locales (estilo Modrinth); la descarga desde el host, después. Plan:
  [`pasos/9-packs-mods.md`](pasos/9-packs-mods.md).

## Multijugador (2026-11-04)
1. Host autoritativo.
2. Combate por iniciativa 5e; fuera de combate, simultáneo.
3. Cada jugador puede estar en un piso distinto.
4. Salvaciones contra la muerte; si muere, vuelve con un personaje nuevo.
5. El botín es de quien lo agarra; XP repartida con bonus al último golpe.
6. Cada jugador guarda su personaje (estilo Terraria); el host guarda la partida.
7. Reglas de la partida opcionales (daño estando caído, "mitad si supera").

## Proyecto (2026-11-03)
- **Rust** en vez de Python: todo por GPU (wgpu + winit, sin SDL), 200+ FPS, Linux, Windows y Android. El prototipo
  queda congelado en `python-prototipe`.
- Dos crates: `game/` (lógica sin render) y `app/` (ventana, render, UI, red con `std::net` e hilos).
- Determinismo: `Rng` compatible con CPython, nada de `HashMap` que decida resultados.
- Pruebas a mano; tests automáticos sólo de red. Guardados en formato nuevo, sin migración.
- APK firmado con clave propia fuera del repo (`~/.pddnd-signing/`). Licencia GPL v3.
- 5e manda sobre los stats de PD ([`dnd5e.md`](dnd5e.md)).
