# game/src/buffs.rs

`game/buffs.py` (actors/buffs/*.java): estados alterados como datos (`data/buffs.json`) + un `Buff` genérico, y las
tiradas de salvación de 5e.

- Cada estado vive en su personaje (`Char::buffs`) y actúa en el planificador como `ActorId::Buff(uid)` una vez por
  turno (en combate, al final de cada ronda): su `tick` (poison: restantes / 3 + 1; bleeding: baja al azar entre la
  mitad y el total; burning: 1-5, se apaga en el agua o al azar), el daño con dados (`damage`), la duración y la
  salvación para librarse (`save_ends` contra su `dc`). Los del héroe viajan con él entre zonas.
- `add_buff` (Buff.affect + prolong: si ya lo tiene se queda con lo mayor), `detach_buff`, `has_flag`
  ("paralysed": no actúa, le atacan con ventaja, falla sola las salvaciones de FUE y DES), `buff_speed` (Lisiado =
  la mitad, también los pies por turno en combate).
- `saving_throw(personaje, característica, CD, motivo)`: d20 + bono (héroe: mod + competencia de su clase; mob:
  `saves` de data/mobs.json o precisión / 4), sin éxito ni fallo automático con 20 o 1. Lo que se ve va al dado
  grande con borde verde / rojo.
- `source_name`: el nombre de un origen de daño (personaje, estado, hambre, trampa) para el log.
