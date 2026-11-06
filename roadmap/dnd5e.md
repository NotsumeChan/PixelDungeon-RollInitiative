# D&D 5e — capa de reglas

## Decisiones (del dueño)
- **Características:** 7 tiradas de 4d6 quitando el menor; se asignan 6 a voluntad y sobra 1. También se
  pueden escribir a mano.
- **Combate:** 5e completo. d20 + mod + competencia contra la CA; dados de daño + mod; crítico con 20 natural;
  1 natural falla; ventaja y desventaja; PV por dado de golpe + CON.
- **Mobs:** se convierten los números de PD (`mob_ac`, `mob_attack_bonus` en `game/mobs.py`) para conservar el
  balance del original. El daño mantiene el rango de PD.
- **Clases:** las 4 de PD mapeadas: Guerrero d10, Mago d6, Pícaro d8, Explorador d10, con sus competencias.

## Hecho
- `game/dice.py` (dados, ventaja, modificadores, competencia, tiradas de características).
- `game/char.py` (tirada de ataque 5e con log de dados), `game/hero.py` (CA, ataque, daño, PV, competencias),
  `game/mobs.py` (conversión), `game/items.py` (dados de armas, sutiles, simples/marciales; CA de armaduras).
- `screens/createScreen.py` (creación de personaje), ficha 5e (click en el avatar) en `screens/gameUi.py`.
- Tiradas de salvación (`Char.saving_throw`, `save_prof` por clase) usadas por las trampas (`game/traps.py`) y
  por los estados con `save_ends` (`game/buffs.py`).
- CA de los mobs bajada 2 puntos (`mob_ac` base 8): los primeros pisos eran muy duros.
- Conjuros como datos (`game/spells.py`): Mago (Descarga de fuego, Rayo de escarcha, Proyectil mágico, Dormir) y
  Explorador con reglas 2024 (Enemigo predilecto = Marca del cazador gratis, Curar heridas). Espacios 5e, tecla F.
- Rasgos como datos (`traits`): Segundo Aliento (Guerrero, tecla F) y Ataque Furtivo (Pícaro). Ver `docs/game/hero.md`.

## Mapeo de equipo
| Arma | Dados | Tipo |
|---|---|---|
| dagger | 1d4 | simple, sutil |
| knuckleduster | 1d4 | simple, ataca 2 veces por turno |
| short sword | 1d6 | marcial, sutil |
| quarterstaff | 1d6 | simple |
| spear | 1d8 | simple, lenta (x0.67) |
| mace | 1d6 | simple, rápida (x1.25) |
| sword | 1d8 | marcial |
| longsword / battle axe | 1d10 | marcial |
| war hammer | 2d6 | marcial |
| glaive | 1d12 | marcial |

| Armadura | CA | Tipo |
|---|---|---|
| cloth | 11 + DES | ropa (sin competencia) |
| leather | 12 + DES | ligera |
| mail | 13 + DES (máx. 2) | media |
| scale | 14 + DES (máx. 2) | media |
| plate | 18 | pesada |

Los requisitos de FUE del original se mantienen: arma muy pesada = desventaja; armadura muy pesada = más lento.

- Salvaciones contra la muerte, daño masivo, ventaja y crítico contra caídos, estabilizar (Medicina sin
  competencia todavía), Curar heridas de toque (`docs/game/hero.md`).

## Pendiente
En [`pendientes.md`](pendientes.md#combate-5e).
