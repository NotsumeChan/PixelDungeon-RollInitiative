# game/src/hero.rs

`game/hero.py` (Hero.java + HeroClass.java + Regeneration.java), con las reglas de 5e. La clase sale de
`data/classes.json`: dado de golpe, competencias, arma y armadura iniciales, `awareness_base`, `speed_ft`.

- Pericias: `proficient_skill` (las `skill_prof` de la clase); el bono lo da `Dungeon::skill_bonus` (`npcs.rs`).
  Estabilizar usa Medicina con su competencia.
- PV: nivel 1 = dado de golpe al máximo + CON; por nivel, promedio + CON (mínimo 1; `gain_level`, que usan la XP y el
  personaje portátil). XP para subir: `5 + nivel * 5`.
- Ataque: d20 + mod (FUE, o la mejor de FUE/DES con arma sutil) + competencia si la clase la tiene. Desventaja con
  arma o armadura que pide más FUE, o armadura sin competencia. CA = `armor_ac`.
- `Action` (Hero.curAction), se repite turno a turno: `Move`, `Attack`, `Help` (estabilizar a un aliado caído),
  `Rest` (1 de tiempo), `Search` (2), `Descend` / `Ascend` (hay que pedirlo con click; pisar la escalera no basta).
  `handle(i, celda, stairs)` elige la acción según lo que hay en la celda. `Leave(celda)` (M1c): una salida del
  borde con sólo pisarla (también con el teclado), un portal sólo con click / toque (`stairs`).
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

R5a (Belongings.java, Hunger.java):
- Mochila de 19 (`BACKPACK_SIZE`), oro aparte, kit inicial de la clase (`items`, conocido) y 2 accesos rápidos (por id;
  el primero de la clase sale de `quick`). `Slot` dice dónde está un ítem (arma, armadura o mochila); `verbs` lo que
  se puede hacer (la primera es la del acceso rápido).
- Acciones: `PickUp` (parado encima), `Open` (cofre / tumba / restos desde al lado; lo de adentro queda como montón),
  `Use(slot, verbo, objetivo)`: comer (baja el hambre), beber / leer (identifica el tipo y aplica su efecto),
  equipar (la armadura tarda el doble; lo maldito no se saca), quitar, soltar.
- Efectos por nombre (`effect` en data/items.json): heal, strength, experience, identify, upgrade, remove_curse,
  magic_mapping, teleport. `target_ok`: qué acepta un pergamino con objetivo (`unknown`, `upgradable`).
- Hambre (`hunger_act`, viaja con el héroe): +10 cada 10 de tiempo (Pícaro cada 12); hambriento a 260, famélico a
  360: no regenera y pierde 1 PV con 30% cada vez (se puede morir de hambre).
- El nivel del arma suma al ataque y a los dados; el de la armadura, a la CA.

R5b: paralizado no actúa (sus turnos pasan solos); con hambre o un estado como origen, el log y "Has muerto (...)" lo nombran. Al morir sale del planificador con regeneración, hambre y estados.

R5c: `second_wind`, `slots`, `free_casts`, `marked`; acciones `Cast(k, celda)` y `SecondWind`; los números del lanzador y los rasgos están en spells.rs.

R5d, armas a distancia (`hero.py`: `_throw`, `_shoot`, `_ranged_attack`, `_after_shot`, `_can_shoot`):
- `Verb::Throw` (LANZAR) es el primero de las arrojadizas en la mochila (el acceso rápido lanza). `Action::Throw(slot,
  celda)`: valida (casilla a la vista, alcance largo, línea de tiro con `Dungeon::shot`), saca una unidad y, si se
  frena en un mob, es un ataque a distancia; contra un aliado o el suelo cae ahí. Tarda `TIME_TO_THROW` x `dly`.
- Arco: `Action::Attack` con arco equipado, flechas (`ammo`, la pila con el id de `ammo` del arco), el enemigo a la
  vista, dentro del alcance largo y con línea de tiro = dispara desde donde está (`shoot`); si no, se acerca y golpea
  cuerpo a cuerpo como un garrote (1d4).
- Fuera de combate, lanzarle o dispararle a un mob inicia el combate (como atacar cuerpo a cuerpo); el tiro va en el
  turno del héroe.
- Durante el ataque, `ranged` = el arma (la arrojadiza o el arco) y `cover` = la cobertura (resta a la tirada):
  `wep()` la usa para el modificador (arco = DES, sutil = la mejor de FUE / DES), competencia, FUE que falta y daño.
  Desventaja (`Dungeon::advantage`) más allá del alcance normal o con un mob vivo al lado. Ataque Furtivo también a
  distancia.
- Al verse la tirada (`Pending::Land`, `apply_land`): el estado del arma si acertó (`on_hit` = [estado, duración, nivel,
  CD]: dardos incendiarios, de curare...), el bumerán vuelve a la mochila (o a los pies si está llena), lo que falla
  cae en la casilla donde se frenó y lo que acierta se pierde.
- Sin proyectil en vuelo todavía (pendiente de pulido, con las partículas).

Apariencia: `Hero::skin` (id de `data/skins.json`), aparte de la clase; se guarda y viaja en el `hello` de la LAN.
`Dungeon::hero_sprite` da la hoja: la de la skin, o la de igual id que su clase (guardados viejos), o la primera.

Rasgos de 5e por datos de la clase: `unarmored_ac` (Defensa sin armadura: sin armadura o con ropa, 10 + DES + esa
característica si es mejor; Bárbaro CON, Monje SAB), el rasgo `pdri:martial_arts` (Monje: sin armas o con armas simples
ataca con FUE o DES, la mayor, y el golpe sin armas hace 1d4). La Furia (estado con flags `rage`: +2 al daño cuerpo a
cuerpo, y `resist`: la mitad del daño de los ataques de los mobs).
