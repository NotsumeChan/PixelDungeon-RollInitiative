# game/spells.py

Conjuros de D&D 5e. El original no tiene un sistema así; la línea de tiro es `game/ballistica.py`.
**Son datos:** `SPELLS[id]` + tipos por nombre (`KINDS`). Un mod de datos combina tipos que ya existen; uno con
código registra tipos nuevos en `KINDS` (`fn(héroe, conjuro, objetivo, celda, niveles extra)`).

## Campos
| Campo | Qué hace |
|---|---|
| `name`, `desc` | str o dict por idioma |
| `level` | 0 = truco (sin espacio; dados x2/x3/x4 en los niveles 5/11/17); 1+ = gasta el menor espacio disponible de ese nivel o mayor |
| `kind` | `attack` (tirada de ataque de conjuro contra la CA, 20 crítico, 1 falla; `damage`, `buff` al acertar), `auto` (`darts` dardos que aciertan solos), `sleep` (reserva `pool`, `radius`), `heal` (`damage` + mod al lanzador), `mark` (pone `buff`, quita la marca anterior) |
| `target` | `enemy` (criatura visible con línea de tiro), `cell` (celda visible), `self`, `touch` (uno mismo o un aliado al lado, también caído) |
| `upcast` | por cada nivel de espacio de más: `{"darts": n}`, `{"pool": "2d8"}`, `{"damage": "2d8"}` |
| `bonus` | acción adicional: no gasta tiempo |
| `fx` | efecto del render: `fire`, `frost`, `missile`, `sleep` (proyectil), `mark` (destello) |

## Base
| Conjuro | Clase | Nivel | Efecto |
|---|---|---|---|
| Descarga de fuego | Mago | truco | ataque, 1d10 |
| Rayo de escarcha | Mago | truco | ataque, 1d8 y lisiado 2 turnos |
| Proyectil mágico | Mago | 1 | 3 dardos de 1d4+1 que aciertan solos (+1 por nivel de espacio extra) |
| Dormir | Mago | 1 | 5d8 PV (+2d8): duerme a los mobs de menos vida a 2 casillas (radio ajustable por dato) |
| Marca del cazador | Explorador | 1 | acción adicional; +1d6 a los ataques con armas contra el marcado (estado `pddnd:hunters_mark`) |
| Curar heridas | Explorador | 1 | 2d8 + SAB (+2d8) |

## Reglas
- Lanzador = `HERO_CLASSES[...]["spellcasting"]`: `ability`, `slots` (`full` Mago / `half` Explorador, tablas 5e en
  `SLOT_TABLES`, el medio lanzador con las reglas 2024) y `spells`. Bono = competencia + mod; CD = 8 + bono.
- `FREE_CASTS`: Enemigo predilecto (`pddnd:favored_enemy`) da Marca del cazador gratis 2 veces (+1 cada 4 niveles).
- No se lanza con armadura sin competencia. Al subir de nivel se suman los espacios nuevos (`level_up`); se
  recuperan todos con `Hero.recover()` (hoy, al llegar a un piso nuevo).
- `cast` valida y, si no se puede, avisa sin gastar tiempo. Tira todo en el momento (orden del rng fijo) y el
  resultado espera en `dungeon.pending` a que el dado se vea, como los ataques. Animación: `attack` (zap) o `operate`.
- Ventaja: sólo la del objetivo (desprevenido o paralizado), no las desventajas del arma o la armadura del héroe.

## Falta
Concentración, foco (varita), reacciones (Escudo), conjuros de área en cono, conjuros de los mobs.

## Varios héroes (0.9)
- El objetivo tiene que estar a la vista del que lanza (`hero.fov`), no de cualquier héroe.
- Objetivo `touch` (Curar heridas): uno mismo o un aliado al lado, también caído (lo levanta). La UI pide la casilla.
- Los resultados van por `Dungeon.defer` (no `pending` directo): lanzado en otro piso se aplica en el acto.
- La línea de tiro pasa por los aliados (`Dungeon.shot`); con uno en medio, -2 a la tirada de ataque del conjuro.
