# game/src/mobs.rs

`game/mobs.py`: un `Mob` genérico con las stats de `data/mobs.json`; `pick_mob` elige por los pesos de
`data/bestiary.json`.

- Conversión PD -> 5e: `mob_ac` = 8 + (defensa + dr) / 2; `mob_attack_bonus` = precisión / 2. El daño conserva el
  rango del original (`NormalIntRange`), dos tiradas con crítico.
- IA (Mob.java): SLEEPING (lo nota con `Int(distancia + sigilo) == 0`, tarda 1 en despertar), WANDERING (va a un
  destino al azar; si nota al héroe pasa a cazar sin gastar tiempo), HUNTING (ataca si está al lado, si no lo
  persigue; si lo pierde, vuelve a deambular). El daño lo despierta y lo alerta.
- Al morir: XP repartida entre los héroes vivos de la zona (redondeo hacia arriba) +1 al del último golpe; en combate se
  junta y se da al terminar (`combat_exp`, ver `hero.md`). El rng del
  loot se gasta igual que el prototipo, pero no cae nada hasta que haya ítems (R5).
- Faltan (R5): habilidades especiales, FLEEING, loot.

R5b: paralizado no actúa ni ve; los estados multiplican su velocidad; la alarma los manda a la trampa (Mob.beckon).
