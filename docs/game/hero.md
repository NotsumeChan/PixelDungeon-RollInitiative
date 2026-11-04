# game/src/hero.rs

`game/hero.py` (Hero.java + HeroClass.java + Regeneration.java), con las reglas de 5e. La clase sale de
`data/classes.json`: dado de golpe, competencias, arma y armadura iniciales, `awareness_base`, `speed_ft`.

- PV: nivel 1 = dado de golpe al máximo + CON; por nivel, promedio + CON (mínimo 1). XP para subir: `5 + nivel * 5`.
- Ataque: d20 + mod (FUE, o la mejor de FUE/DES con arma sutil) + competencia si la clase la tiene. Desventaja con
  arma o armadura que pide más FUE, o armadura sin competencia. CA = `armor_ac`.
- `Action` (Hero.curAction), se repite turno a turno: `Move`, `Attack`, `Help` (estabilizar a un aliado caído),
  `Rest` (1 de tiempo), `Search` (2), `Descend` / `Ascend` (hay que pedirlo con click; pisar la escalera no basta).
  `handle(i, celda, stairs)` elige la acción según lo que hay en la celda.
- Sólo camina por celdas ya vistas; un mob nuevo a la vista corta la acción (`check_visible_mobs`). Tras cada paso,
  búsqueda pasiva con probabilidad `awareness`.
- `animating`: tras un paso el planificador espera a que el render termine el movimiento (lo baja la app).
- A 0 PV cae inconsciente (salvo daño masivo). Salvación contra la muerte al inicio de cada turno (10+ éxito, 20 =
  vuelve con 1 PV, 1 = dos fallos); daño estando caído = 1 fallo, 2 si el atacante está al lado. 3 éxitos = estable,
  3 fallos = muere. La salvación y la Medicina (`stabilize`, d20 + SAB vs 10) van por `pending`.
- `regen_act`: +1 PV cada 10 de tiempo (caído, sólo si está estable).
- Faltan (R5): hambre, inventario, rasgos de clase, conjuros, armas a distancia, creación de personaje.

`save_bonus(característica)`: mod + competencia si la clase la tiene (`save_prof`); la usa la ficha y las salvaciones
de R5.
