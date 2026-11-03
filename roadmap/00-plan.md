> **Migración a Rust (2026-11-03):** esta versión en Python es el prototipo de referencia (rama `python-prototipe`).
> Plan: `roadmap/migracion-rust.md`.

# PixelDungeonMazes&Monsters — Plan

Objetivo: juego base de Pixel Dungeon (pisos 1–4, Alcantarillas, sin el jefe Goo del piso 5),
con la misma generación de niveles que el original y multijugador P2P.

Fuente de referencia: `~/Documents/programacion/personal/java/pixel-dungeon` (Java original).
`pixelDungeonPy` no sirve como base: `level/`, `room/`, `patch`, `terrain` están vacíos.
Se porta directo desde Java, solo lo que usan los pisos 1–4.

## Principio de arquitectura

**Lógica separada del render.** Toda la simulación (nivel, actores, combate, turnos) vive en
`game/` sin importar pygame. Motivos:
- se puede revisar sin ventana (`python -m game.level <seed> <depth>`);
- el multijugador sólo sincroniza estado/acciones de `game/`, nunca superficies de pygame;
- determinismo: cada nivel usa su propio `random.Random(seed)`, **nunca** el `random` global.

```
main.py, Local.config, Assets/, Shaders/
core/      ventana OpenGL (Screen, HiRes), Font1x, Button, Config (Local.config), Version
screens/   título, personajes (lista y creación), juego (gameScreen + gameUi), charSprite, particles, diceFx,
           about, mods
game/      lógica pura, sin pygame
  rng, geom, terrain, patch, room, painters, level   generación (Alcantarillas, pisos 1–4)
  fov, pathfinder, ballistica                         vista, caminos, línea de tiro
  actor, char, hero, mobs, dungeon                    turnos, combate 5e, IA, partida y pisos
  items, buffs, spells, traps, dice                   ítems, estados, conjuros, trampas, dados 5e
  lang, registry, save, characters, globals           idiomas, registros JSON, guardado, personajes
data/      contenido en JSON (classes, mobs, bestiary, items, buffs, spells)
net/       multijugador (0.9, ver roadmap/fase5-red.md)
docs/      un .md por archivo de código (+ data.md, assets.md)
roadmap/   este plan + un doc por fase
```

## Fases

### 0. Base que arranca ✅ (detalle en `roadmap/fase0-base.md`)
Bugs detectados en el código actual:
- `core/Config.py`: `Version = Resolutions[...version]` -> KeyError ("0.0.1" no es resolución).
- `screens/homeScreen.py`: anota `pg.Surface` pero importa pygame como `py` -> NameError.
- `core/Button.py`: `self.IsHover` sin `()` (siempre True); `SetPos*` llaman `rect.center(...)` en vez de asignar; `SetHoverImg` pisa `img`.
- `core/AnimationPlayer.py`: parámetro `ammountFrames` duplicado (SyntaxError), usa `pygame.` sin importar, `_splitFrames` sin `self`.
- `core/Player.py`: `LoadIcon` siempre lanza excepción; `Draw` usa `self.color` inexistente.
- No hay entorno: crear `venv` con `pygame-ce` + `moderngl` (los mismos que `pixelDungeonPy/venv`) y un `requirements.txt`.
- No es repo git: `git init` recomendado antes de empezar.

### 1. Generación de niveles (headless) — ver `roadmap/fase1-generacion.md`
Port de `RegularLevel.build()` en el mismo orden:
`initRooms (split recursivo)` -> elegir entrada/salida -> `Graph` distancias -> conectar salas
-> `assignRoomType` (especiales) -> `paint` -> `paintWater`/`paintGrass` (Patch) -> `placeTraps`
-> `decorate` (SewerLevel) -> `createMobs` -> `createItems`.
Primero sólo salas estándar + entrada/salida + túneles/pasajes; las salas especiales se suman después.
Check: volcado ASCII por consola + test "misma seed = mismo nivel" y "entrada alcanza salida".

### 2. Render del mapa — ver `roadmap/fase2-render.md`
Tilemap con `Assets/Tiles/tiles0.png` + `water0.png`, cámara que sigue al héroe, niebla de guerra (FOV).

### 3. Héroe, turnos y mobs — ver `roadmap/fase3-turnos.md`
Planificador por tiempo de `Actor.java`, movimiento por click/teclas, combate cuerpo a cuerpo,
IA básica (SLEEPING/WANDERING/HUNTING), mobs del Bestiary 1–4. Bajar escaleras 1 -> 4.

### 4. Ítems, inventario y UI ✅ — ver `roadmap/fase4-items-ui.md`
Primera pasada hecha: oro, comida, pociones/pergaminos simples, armas y armaduras, mochila y UI original.

### D&D 5e ✅ (0.5–0.7) — ver `roadmap/dnd5e.md`
Creación de personaje con dados y combate 5e sobre la base de PD.

### Futuro cercano: mods, personajes, partidas en red, crossplay
Plan en `roadmap/futuro-mods-personajes-red.md`: packs de mods estilo Modrinth, sincronización estilo
tModLoader, mods obligatorios vs. de cliente, lista de personajes, partidas en LAN y Android/Windows/Linux.

### Opciones
- FPS: 30 (default) / 60 / 120 / sin límite (`clock.tick(0)`). Hoy se fija en `Local.config` (`maxFPS`).

### Audio (al final)
Música (`theme.mp3`, `game.mp3`) y sonidos (`snd_*.mp3`) del original.

### 5. Multijugador (0.9 LAN, 1.0 estable) — ver `roadmap/fase5-red.md`
Host autoritativo (ver decisiones abajo): el host corre la partida y los clientes mandan acciones y dibujan lo que
reciben. stdlib `asyncio` + TCP, mensajes JSON por línea, descubrimiento LAN por UDP. Primero varios héroes y el nuevo
modelo de turnos sin red.

## Decisiones del multijugador (del dueño)
1. **Topología:** host autoritativo. El host simula; los demás mandan acciones y reciben el resultado.
2. **Turnos:** en combate, por **iniciativa 5e** (se tira al empezar y se actúa por orden). Fuera de combate,
   **simultáneo** (todos eligen y el mundo avanza). Cambia el planificador actual (tiempo de PD): diseñarlo en la 0.9.
3. **Pisos:** cada jugador puede estar en un piso distinto: el host simula varios pisos a la vez (ya existen
   `Dungeon.floors`, uno con su planificador por piso).
4. **Muerte:** salvaciones contra la muerte de 5e (a 0 PV cae inconsciente; un aliado puede estabilizarlo o curarlo).
   Si muere, puede volver a entrar a la partida con un personaje nuevo.
5. **Botín y XP:** el botín es de quien lo agarra. La XP se reparte entre los jugadores, con un bonus chico para el
   que da el último golpe.
6. **Conexión:** sólo LAN en la 1.0. Después: internet con relay y la opción de que cada uno hostee su propio
   servidor, sin que sea exclusivo de PC.
7. **Personajes:** cada jugador guarda su personaje (estilo Terraria) y lo lleva a la partida de cualquier host; el
   host guarda la partida.
8. Clases y cuánto D&D: resuelto (`roadmap/dnd5e.md`).
