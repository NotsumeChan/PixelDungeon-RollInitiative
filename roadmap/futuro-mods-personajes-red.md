# Futuro — Mods, personajes, partidas en red y crossplay

> Sólo es plan. No se implementa ahora: la prioridad es la fase 3. Sirve para que lo que se haga hoy no cierre
> estas puertas. Es una prueba de concepto: el alcance se recorta cuando llegue el momento.

## Ideas del dueño (resumen)
- **Mods/addons:** personajes nuevos, cambiar el sistema de personajes entero, personajes "custom" basados en
  los 4 originales, clases, subclases, ítems, habilidades, requisitos de nivel y escalados custom.
- **Estilo Modrinth (Minecraft):** primero eliges un *pack* de mods (instancia); dentro de la instancia están
  sus personajes guardados.
- **Estilo tModLoader (Terraria):** al unirte a la partida de otro, se crea sola una instancia con los mods
  mínimos para jugar.
- **Categorías de mods:**
  - **Obligatorio / gameplay:** personajes, sistemas, ítems. Todos los jugadores deben tenerlo.
  - **Cliente / estético:** gráficos, sonido. No afecta al juego; cada uno usa los suyos.
- **Sin tienda de mods:** los mods se consiguen por fuera, y eso está bien.
- **Personajes:** lista de personajes creados, en vez del único guardado del original. Lista de partidas
  disponibles para unirse, primero en red local.
- **Crossplay:** Android, Windows y Linux. iOS y Mac quedan fuera.

## Lo que ya prepara la fase 3
- Las clases (`game/hero.py: HERO_CLASSES`) y los mobs (`game/mobs.py: MOBS`, `BESTIARY`) son **datos en dicts**,
  no una clase de Python por cada uno. Un mod agrega o pisa entradas.
- Los estados alterados (`game/buffs.py: BUFFS`) también son datos: un mod de datos crea estados nuevos combinando
  daño con dados, velocidad, flags y `save_ends`; un mod con código registra comportamientos en `TICKS`.
- Nada es global salvo `Globals`. La partida es un objeto (`Dungeon`) y cada piso tiene su `Scheduler`.
- Toda la aleatoriedad del juego sale de rngs con seed, así que es determinista entre peers.
- `game/` no usa pygame: la lógica corre igual en cualquier plataforma, o sin ventana en un host dedicado.

## Diseño propuesto

### 1. Registros y contenido como datos (paso previo, chico)
- Pasar `HERO_CLASSES`, `MOBS`, `BESTIARY` (y luego ítems, habilidades, subclases) a archivos JSON en `data/`,
  cargados al iniciar en registros (`dict`s).
- Un mod es otra carpeta con la misma estructura. Se cargan en orden y lo último pisa a lo anterior.
- **Custom basado en una clase original:** `{"base": "pddnd:warrior", "HT": 25, ...}`. Se copia la base y se aplican
  los cambios.
- **Escalados y requisitos:** `"per_level": {"HT": 5, "attack": 1}`, `"requires": {"lvl": 5}`. Hoy están fijos
  en `Hero.earn_exp`; pasarían a leerse de la clase.

### 2. Instancias (packs), estilo Modrinth
```
instances/<pack>/
  pack.json        # nombre, lista de mods [{id, version, sha256, side}], versión del juego
  mods/            # .pdmod (zip)
  characters/      # un .json por personaje creado
  saves/           # partidas guardadas
```
- **Inicio:** elegir instancia -> lista de personajes -> crear / elegir personaje -> jugar o unirse.
- Hay una instancia "vanilla" sin mods.
- **Personaje:** clase (clave del registro), nombre, stats/progreso y los mods que necesita. Si falta uno de
  esos mods, el personaje aparece deshabilitado, sin romperse.

### 3. Formato de mod (`.pdmod` = zip)
```
mod.json   # id, name, version, author, side: "required" | "client", depends: [{id, version}]
data/      # JSON que se suma a los registros (clases, mobs, ítems, habilidades...)
assets/    # sprites, sonidos (los "client" sólo traen esto)
code/      # OPCIONAL: Python (ver seguridad)
```
- **`side: required`:** cambia el juego. Todos los peers deben tener exactamente el mismo (`sha256`).
- **`side: client`:** sólo visual o sonido. No se compara ni se envía.

### 4. Unirse a una partida, estilo tModLoader
1. El host anuncia la partida con la lista de mods `required` (id, versión, sha256).
2. El cliente busca cada hash en su caché local de mods (`mods_cache/<sha256>.pdmod`).
3. Los que falten se descargan **del propio host** por la misma conexión P2P (sin tienda ni servidor central).
4. Se crea o reutiliza la instancia temporal `instances/_join_<hash del pack>/` con esos mods más los `client`
   que el jugador quiera.
5. Antes de empezar se compara el hash de los registros cargados. Si difiere, no se entra: evita desyncs.

### 5. Seguridad (decidido)
Python no tiene un sandbox confiable: un mod con código puede hacer cualquier cosa en la máquina.
- **Mods sin código** (sólo JSON y assets, sin nada que se ejecute): se instalan sin pedir permiso, también
  al unirse a una partida. Siempre se muestra un aviso general: los mods vienen de fuentes externas, hay que
  tener cuidado de dónde se descargan y revisarlos.
- **Mods con código** (`code/` o cualquier forma de ejecutar código): se pueden instalar igual, pero con una
  advertencia fuerte y confirmación explícita. El aviso dice que el mod trae modificaciones de código, que puede
  cambiar el gameplay de forma considerable y que hay que tener mucho cuidado al instalarlo. También muestra el
  mod, el autor y quién lo envía (al unirse a una partida).
- **Detectar código:** el cargador clasifica el `.pdmod` mirando su contenido real (archivos `.py`, `.pyc`,
  binarios, cualquier cosa fuera de `data/` + `assets/` + `mod.json`), no lo que declare el `mod.json`.
- El contenido JSON se valida contra un esquema al cargar. Un mod de datos nunca puede apuntar a código
  arbitrario: sólo a efectos ya existentes en el juego, por nombre.

### 6. Partidas en red local
- **Descubrimiento:** el host emite un broadcast UDP en la LAN cada ~1 s (stdlib `socket`) con nombre, jugadores,
  versión y hash del pack. El cliente escucha y arma la lista de partidas.
- **Conexión:** TCP (`asyncio`) con mensajes JSON. Encaja con la fase 5 (P2P).
- Internet (NAT traversal o relay) queda para después.

### 7. Crossplay Android / Windows / Linux
- **Lógica:** Python puro y determinista. `random.Random` usa el mismo algoritmo en todas las plataformas y los
  floats son IEEE 754. No usar `hash()` de Python ni el orden de los `set`.
- **Render — riesgo principal:** `moderngl` pide OpenGL 3.3 de escritorio y Android tiene OpenGL ES, así que
  probablemente no funcione allí. Por suerte todo se dibuja en una Surface de pygame y `moderngl` sólo la estira
  a la ventana. En Android, `Screen.Update` puede usar `pygame.transform.scale` + `display.flip`. Es un cambio
  chico y aislado en `core/Screen.py`.
- **Empaquetado Android:** python-for-android / buildozer. Verificar que haya receta de `pygame-ce` (o usar `pygame`).
- **Input:** el original es táctil. El click ya funciona como toque; faltan botones en pantalla para
  esperar, buscar e inventario.
- **Rutas:** carpeta de datos por plataforma (instancias, caché de mods, personajes), no rutas relativas al código.
- **Mods:** sólo datos o Python puro, sin extensiones nativas, para que el mismo `.pdmod` sirva en las 3 plataformas.

### 8. Dungeons prefabricadas (tipo de mod)
- Un tipo de `.pdmod` que trae un **layout fijo** (mapa diseñado a mano) en vez de usar el generador.
- El layout define terreno, salas, entradas y salidas, y opcionalmente mobs e ítems clave en posiciones fijas.
- Los ítems **no clave** siguen saliendo al azar del Generator, salvo que el mapa diga lo contrario (por ítem o
  por zona: `"random_items": false`, o listas fijas).
- Encaja con lo que ya hay: un `Level` que en vez de `build()` carga el layout; `create_items` respeta lo fijo.

### 9. Mapa no sólo vertical: coordenadas, orientación y zonas
- Hoy el mundo es una pila de pisos (bajar = depth + 1). Propuesta: un **mundo de zonas conectadas** en un grafo,
  con coordenadas (por ejemplo `x, y, z`, donde z = profundidad) y salidas en varias direcciones
  (norte/sur/este/oeste, arriba/abajo).
- Mostrar coordenadas y orientación en la UI (brújula / minimapa del mundo) para ubicarse.
- **Nombres de zona:** cada zona puede tener un nombre (puesto por el diseñador del mapa o por el generador) que
  se muestra al entrar y en la barra de estado.
- Impacto: `Dungeon.descend()` pasa a ser "ir a la zona conectada por esta salida", con zonas cacheadas (volver a
  una ya visitada), y la seed de cada zona sale de sus coordenadas en vez de `seed*1000+depth`.

### Licencia y créditos
- El código y los gráficos del original son GPL v3: si el proyecto se distribuye, también es GPL v3 (LICENSE y
  código fuente disponibles). Los mods con código que deriven del juego podrían heredar la GPL; los de sólo
  datos son más discutibles. Revisar antes de publicar.
- D&D 5e: si se usan reglas o textos del SRD 5.1, van con atribución (CC-BY-4.0) en el About. "Dungeons & Dragons"
  es marca de Wizards of the Coast: no usarla como nombre del juego ni dar a entender afiliación.

## Orden sugerido
1. Terminar la fase 3, luego la 4 (ítems).
2. Registros en JSON (punto 1): chico, y destraba los custom.
3. Personajes múltiples y guardado por personaje.
4. Fase 5 P2P + descubrimiento LAN (punto 6).
5. Instancias y formato `.pdmod` (puntos 2 y 3).
6. Sincronización estilo tModLoader (punto 4) con los avisos de seguridad (punto 5).
7. Port a Android (punto 7).

## Decisiones (del dueño)
- Los personajes viajan entre partidas de distintos hosts (como en Terraria): cada jugador guarda su personaje y el
  host guarda la partida. Ver las demás decisiones del multijugador en `roadmap/00-plan.md`.
- Registros en JSON (punto 1): hecho en la 0.8 (`data/*.json`, `game/registry.py`, con `"base"`).

## Tal vez (más adelante)
- **Editor:** una aplicación aparte para crear mapas (dungeons prefabricadas, zonas y sus nombres) y mods no
  complejos: clases, personajes custom, ítems, ajustes de stats. Generaría `.pdmod` de sólo datos, que son los que
  se instalan sin la advertencia de código.
