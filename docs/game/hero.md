# game/hero.py

El héroe: `Hero.java` + `HeroClass.java` + `Belongings.java` + `Regeneration` + `Hunger`, con reglas de D&D 5e.

## Clases como datos
`HERO_CLASSES[id]`, IDs `pddnd:warrior`, `pddnd:mage`, `pddnd:rogue`, `pddnd:huntress`. **Es el punto de entrada
para addons:** una clase nueva o custom es otra entrada.

| Clase PD | Clase 5e | Dado de golpe | Armas | Armaduras | Inicio |
|---|---|---|---|---|---|
| Warrior | Guerrero | d10 | simples y marciales | todas | short sword; conoce la poción de fuerza |
| Mage | Mago | d6 | dagger, quarterstaff, knuckles | ninguna (la ropa sí) | knuckles; conoce identify |
| Rogue | Pícaro | d8 | simples + short sword, sword, longsword | ligeras | dagger; conoce magic mapping; hambre más lenta |
| Huntress | Explorador | d10 | simples y marciales | ligeras y medias | dagger |

Todos empiezan con armadura de tela (cuenta como ropa) y una ración.
Campos: `dnd_class`, `hit_die`, `weapon_prof` (`simple` / `martial` / IDs), `armor_prof` (`light` / `medium` /
`heavy`), `start_weapon`, `start_armor`, `items`, `known`, `awareness_base`, `hunger_step`, `traits`, `save_prof`
(salvaciones con competencia: Warrior FUE/CON, Mage INT/SAB, Rogue DES/INT, Huntress FUE/DES).

## Características
- `abilities`: `{STR, DEX, CON, INT, WIS, CHA}`, vienen de la creación de personaje (`screens/createScreen.py`).
  Sin personaje se usa el array estándar.
- `mod(ab)`: modificador. `STR` es una propiedad (la usan los requisitos de fuerza de ítems).
- La poción de fuerza suma +1 a FUE (máximo 20).

## Combate 5e
- **CA:** armadura + DES (con tope según la armadura) + nivel de la armadura. Sin armadura: 10 + DES.
- **Ataque:** mod de FUE (o el mayor entre FUE y DES con armas sutiles) + competencia (si es competente con el
  arma) + nivel del arma.
- **Daño:** dados del arma + nivel + el mismo mod. Sin arma: 1 + FUE.
- **Desventaja** si al arma o a la armadura les falta FUE (`str_missing`) o si llevas armadura sin competencia.
  Además el arma pesada resta 1 al ataque por cada punto de FUE que falta. Ventaja contra mobs
  desprevenidos. Ambas se anulan.
- **Velocidad:** la armadura que pide más FUE te hace más lento (regla del original, `/1.3^n`).
- **Sigilo:** `max(0, mod DES)`. Hace más difícil que los mobs te noten.
- **PV:** nivel 1 = dado de golpe máximo + CON (mínimo 1). Cada nivel suma el promedio del dado + CON (mínimo 1).
- **XP:** se mantiene la curva del original (`5 + nivel * 5`). La competencia sube según el nivel.

## Rasgos de clase 5e (`traits`)
IDs de rasgo en la clase; `has_trait(id)` los consulta. Un mod puede dárselos a cualquier clase.

| Rasgo | Clase | Efecto |
|---|---|---|
| `pddnd:second_wind` | Warrior | Acción `second_wind` (tecla F): 1d10 + nivel PV, sin gastar tiempo (acción adicional). `second_wind` = disponible; se recarga al llegar a un piso nuevo hasta que haya descansos |
| `pddnd:sneak_attack` | Rogue | Si ataca con ventaja (neta, tras anular desventajas) y arma sutil: +`sneak_dice()`d6 (1d6 en nivel 1, +1 cada 2 niveles), dobles en crítico. Va en el mismo evento de dado que el arma (`1d4+1d6`) |

Trampas: el Ataque Furtivo depende de la ventaja que calcula `Char.attack`, por eso `roll_damage` recibe `adv`.
En 5e también vale con un aliado al lado del objetivo: llega con el multijugador.

## Pertenencias
`quickslots`: 2 ranuras de acceso rápido (id para apilables, `Item` para equipo); `quick_item(i)` devuelve el ítem actual.
`weapon`, `armor`, `backpack` (máx. 19), `gold`. `collect(item)` apila o devuelve False si está llena.
`proficient_weapon(item)` y `proficient_armor(item)` los usa también la UI.

## Acciones (`hero.action`)
`handle(celda)` decide: atacar si hay una criatura visible, `heap` si hay ítems, `descend` si es la salida,
`ascend` si es la entrada (sólo con `stairs=True`, es decir con click: pisar la escalera con
las teclas o al pasar no cambia de piso), si no `move`.

| Acción | Turno |
|---|---|
| `move` | un paso por turno, sólo por celdas ya vistas |
| `attack` | se acerca; al estar al lado ataca una vez (gasta la velocidad del arma) |
| `descend` / `ascend` | click en la salida / entrada: camina hasta ella y baja / sube. Parado encima, click en ella |
| `heap` | camina al montón; lo abre (cofre, tumba, esqueleto) o recoge el ítem de arriba (1) |
| `second_wind` | 0 (ver rasgos) |
| `rest` / `search` | 1 / 2. Buscar encuentra todos los secretos vecinos |
| `use` | `hero.use(item, verbo, objetivo)`: `eat` (3), `drink` / `read` (1), `equip` (arma 1, armadura 2), `unequip` (1), `drop` (0.5). Lo maldito no se puede quitar ni soltar |

- `waiting()`: el planificador se detiene sin acción, durante la animación o si está muerto.
- Se interrumpe si aparece un enemigo nuevo a la vista o si recibe daño.
- Tras cada paso: `observe` y búsqueda pasiva de secretos (probabilidad `awareness`, como el original).

## Regeneration y Hunger
- `Regeneration`: +1 PV cada 10 de tiempo, salvo famélico.
- `Hunger`: +10 cada 10 de tiempo (Rogue cada 12). Hambriento a 260, famélico a 360: no regenera y pierde 1 PV
  con 30% en cada paso; se puede morir de hambre.

## Pendiente (5e)
Rasgos de clase (Oleada de Acción, conjuros, Explorador), habilidades
(Percepción para buscar), descanso corto/largo con dados de golpe, subclases, requisitos de nivel.

## Estados y salvaciones
- `save_bonus(car)` = mod + competencia si `car` está en `save_prof`.
- Paralizado: `waiting()` no frena el planificador; `act()` descarta la acción y pasa el turno.
- `speed()` aplica `buff_speed()` (lisiado = la mitad).

## Búsqueda y nombres
- `search(True)` (Q o la lupa): destello azul en cada casilla visible revisada (`CheckedCell`), "buscando" sobre el
  héroe y, si no hay nada, "Buscas, pero no encuentras nada". El orden del rng no cambió.
- `name` y `dnd_class` por idioma: Guerrero/Warrior, Mago/Mage, Pícaro/Rogue, Cazadora/Huntress (`dnd_class`:
  Guerrero/Fighter, Mago/Wizard, Pícaro/Rogue, Explorador/Ranger). El nombre del héroe se fija al crearlo.

## Cofres, tumbas y restos (`open`)
- Click en un montón cerrado -> acción `("open", celda)` (`HeroAction.OpenChest`): camina hasta una casilla **vecina**
  (o lo abre si ya está al lado o encima), lo abre en 1 de tiempo (`TIME_TO_OPEN`, `Key.TIME_TO_UNLOCK`) y lo de adentro
  queda en el suelo como montón normal. Vacío -> desaparece ("Está vacío").
- Un montón normal se recoge con click: camina encima (`heap`, `PickUp`), como el original.
- Falta: la tumba no invoca espectros y los restos con un ítem maldito no hacen daño (no hay Wraith).

## Animaciones
Emite `("anim", héroe, tipo, celda)` en `d.fx`: `operate` al abrir, buscar, comer y beber; `read` al leer. El ataque lo
emite `Char.attack`.

## Conjuros y Explorador
- `spellcasting` en la clase (Mago: INT, lanzador completo; Cazadora/Explorador: SAB, medio lanzador). Ver
  `docs/game/spells.md`. Estado: `slots` (espacios disponibles por nivel), `free_casts` (usos gratis), `marked`.
- Acción `("cast", id, celda)`: la resuelve `spells.cast`.
- `recover()`: Segundo Aliento, espacios y usos gratis (descanso largo; hoy, piso nuevo).
- Rasgo `pddnd:favored_enemy` (Explorador): Marca del cazador gratis. `roll_damage(crit, adv, target)` suma 1d6
  (dobles en crítico) si el objetivo tiene `pddnd:hunters_mark` y se ataca con arma.

## Armas a distancia
- **Lanzar** (`("throw", ítem, celda)`, `Item.cast` + `MissileWeapon.onThrow`): una unidad hacia una casilla visible;
  se frena donde diga `ballistica`. Si se frena en alguien es un ataque 5e a distancia; si no, cae ahí.
- **Arco**: con el arco equipado y flechas en la mochila, atacar a un enemigo a la vista y con línea de tiro dispara
  desde donde estás (`_can_shoot` / `_shoot`). Sin flechas, se acerca y golpea con el arco como garrote (1d4).
- `_ranged_attack` = `Hero.shoot`: pone `self.ranged` mientras dura el ataque; `_wep()` es esa arma (o la equipada).
  Modificador: arco = DES; arrojadiza sutil = el mayor de FUE y DES; si no, FUE. Competencia por `kind`/ID.
- Desventaja a distancia (5e): enemigo al lado o objetivo más allá del alcance normal. Ataque Furtivo vale con
  armas a distancia; la Marca del cazador también.
- `_after_shot`: cuando el dado ya se vio, aplica `on_hit`, y el proyectil cae (falló), se pierde (acertó) o vuelve
  (bumerán; con mochila llena, cae a tus pies).
- Equipo inicial: Guerrero y Pícaro 8 dardos (acceso rápido), Cazadora el bumerán (acceso rápido); el Mago es
  competente con dardos (5e). `items` acepta `(id, cantidad)`; `quick` = id del primer acceso rápido.

## Varios héroes (0.9)
- `depth`: piso del héroe. `fov`: su propia vista (la calcula `Dungeon.observe`). `handle`, perseguir, disparar,
  lanzar, conjuros, buscar y `check_visible_mobs` usan `fov`, no `level.visible` (que es la unión de todos).
- `ticks = [Regeneration, Hunger]`: viajan con el héroe entre pisos, como sus estados (antes había uno por piso).
- `descend` / `ascend` llaman a `dungeon.descend(self)` / `ascend(self)`.
- `die`: sale del planificador con sus `ticks` y estados, así el piso sigue para los demás. Sin héroes vivos no se
  procesa nada. Las salvaciones contra la muerte (5e) son el paso 2.
- Click sobre otro héroe: caído = estabilizar; en pie = caminar hacia él. No hay fuego amigo (ver "Salvaciones contra
  la muerte" y `Dungeon.shot`).

## Salvaciones contra la muerte (5e, 0.9 paso 2)
- `dead`, `saves = [éxitos, fallos]`, `stable`. `is_alive()` = no muerto; `is_downed()` = vivo a 0 PV (caído).
- `damage`: a 0 PV cae inconsciente (se reinician las salvaciones). Daño masivo: si lo que sobra llega a los PV
  máximos, muere en el acto. Caído, cada daño es un fallo (dos si el atacante está al lado) y pierde lo estable.
- `act` caído: no hace nada (`waiting` no lo espera); si no está estable tira `_death_save`: d20, 10+ éxito, 1 = dos
  fallos, 20 = vuelve con 1 PV. 3 éxitos = estable, 3 fallos = muere. Se aplica al mostrarse el dado (`Dungeon.defer`).
- `heal(n)`: toda curación pasa por aquí (pociones, Segundo Aliento, Curar heridas, regeneración) y levanta al caído.
  La regeneración no cura a un caído salvo que esté estable (así despierta solo con el tiempo, como en 5e).
- Subir de nivel estando caído suma PV máximos pero no lo levanta.
- Click sobre un aliado: caído -> acción `("help", aliado)`: se acerca y tira Medicina (`_stabilize`: d20 + SAB vs CD
  10, una acción); en pie -> camina hacia él. Ya no hay ataque a aliados.
- `die`: `dead = True`; queda de espectador. Volver con un personaje nuevo llega con la red (pasos 5-6).
- Daño estando caído: además del texto flotante, el log dice qué lo causó y cuántos fallos lleva
  ("recibe daño estando caído (veneno): 1 fallo (2/3)"). Si el daño viene de un estado y la regla `dot_death_saves`
  está apagada, no cuenta. El dado de la salvación contra la muerte dice "Muerte CD10".
- A distancia: la línea pasa por los aliados (`Dungeon.shot`); con uno en medio, `cover` = 2 resta a la tirada y el
  log avisa "media cobertura".

## Combate 5e (0.9)
- `base_speed_ft` = `speed_ft` de la clase (30 por defecto). `speed_ft()`: armadura `heavy` sin la FUE que pide = -10 ft.
- `initiative_bonus()` = mod de DES. `get_closer` respeta `can_step()` (ver `docs/game/char.md`).
- Esperar (Espacio) en tu turno = terminar el turno (gasta la acción).
