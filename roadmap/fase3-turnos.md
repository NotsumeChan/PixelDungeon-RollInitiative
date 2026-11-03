# Fase 3 — Héroe, turnos y mobs

## Estado
- ✅ Escrito, pendiente de revisión manual del dueño.
- `game/actor.py` (turnos), `game/pathfinder.py`, `game/char.py` (combate), `game/mobs.py` (datos + IA
  SLEEPING/WANDERING/HUNTING), `game/hero.py` (4 clases como datos, acciones, XP y niveles, búsqueda,
  regeneración), `game/dungeon.py` (partida, pisos 1→4, respawn).
- `screens/gameScreen.py`: click o teclas para moverse y atacar, sprites originales (primer frame), HUD y log.
- Docs: `docs/game/{actor,pathfinder,char,mobs,hero,dungeon}.md`, `docs/screens/gameScreen.md`.
- Pendiente: habilidades especiales de los mobs, FLEEING, números de daño,
  caer por el abismo, hambre, loot (todo eso necesita ítems o buffs).
- Clases y mobs quedaron como datos para los mods futuros: ver `roadmap/futuro-mods-personajes-red.md`.

Objetivo: un héroe que se mueve y pelea, mobs con IA básica, bajar del piso 1 al 4. Un solo
jugador por ahora; el multijugador (fase 5) se monta encima, por eso todo vive en `game/`.

## Planificador de turnos (`actors/Actor.java` -> `game/actor.py`)
- Cada actor tiene `time`. `process()` toma el actor con menor `time` y llama `act()`; repite mientras `act()` devuelva True.
- `spend(t)` suma tiempo (`TICK = 1`). Moverse/atacar = 1 tick por defecto.
- El héroe devuelve False en `act()` cuando espera input -> el loop se detiene hasta la próxima acción.
- En Java el turno espera a que termine la animación del sprite; aquí eso es cosa del render, la lógica no espera.

## Combate (`actors/Char.java`)
- `hit()`: `Random.Float(attackSkill) >= Random.Float(defenseSkill)` (ver modificadores en `Char.hit`).
- Daño = `damageRoll()` - `dr()` del objetivo.

## Héroe (`actors/hero/Hero.java`)
- `HP = HT = 20`, `STR = 10`, `attackSkill = 10`, `defenseSkill = 5`. Clase inicial a definir (ver plan).
- Movimiento: click en celda -> `utils/PathFinder.java` -> `game/pathfinder.py`, un paso por turno.

## Mobs (`actors/mobs/`, `Bestiary.java`)
| Piso | Pesos |
|---|---|
| 1 | Rat |
| 2 | Rat 1, Gnoll 1 |
| 3 | Rat 1, Gnoll 2, Crab 1, Swarm 0.02 |
| 4 | Rat 1, Gnoll 2, Crab 3, Swarm 0.02, Skeleton 0.01, Thief 0.01 |

| Mob | HP | def | daño | atk | dr | EXP | maxLvl |
|---|---|---|---|---|---|---|---|
| Rat | 8 | 3 | 1–5 | 8 | 1 | 1 | 5 |
| Gnoll | 12 | 4 | 2–5 | 11 | 2 | 2 | 8 |
| Crab | 15 | 5 | 3–6 | 12 | 4 | 3 | 9 |
| Swarm | 80 | 5 | 1–4 | 12 | – | – | 10 |
| Skeleton | 25 | 9 | 3–8 | 12 | 5 | 5 | 10 |
| Thief | 20 | 12 | 1–7 | 12 | 3 | 5 | 10 |

(Verificar EXP/dr de Swarm y Rat en sus `.java` al portar.)
Habilidades especiales (Swarm se divide, Skeleton explota, Thief roba, Gnoll suelta oro): después de lo básico.
Cantidad por piso: `2 + depth % 5 + Int(3)`, en salas STANDARD fuera de la vista del héroe.

## IA (`actors/mobs/Mob.java`, 593 líneas)
Estados: SLEEPING, WANDERING (`randomDestination`), HUNTING (persigue si ve al héroe), FLEEING, PASSIVE.
Para fase 3 basta SLEEPING/WANDERING/HUNTING.

## Escaleras
- Pisar EXIT -> generar piso `depth+1` con una seed derivada estable (p. ej. `seed * 1000 + depth`;
  no usar `hash()` de Python, cambia entre procesos y rompería el P2P).
- El piso 4 es el último; el 5 (Goo) queda fuera.

## Checks
- Test: un Rat adyacente al héroe lo ataca en su turno; los actores actúan en orden de `time`.
