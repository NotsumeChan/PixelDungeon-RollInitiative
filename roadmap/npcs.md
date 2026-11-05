# NPCs (pedido del dueño, 2026-11-05)

Meta: NPCs como en D&D. Hablar con ellos, conseguir misiones, objetos y pistas, comprar y vender, y también
persuadirlos, mentirles, intimidarlos, coquetearles o atacarlos, con tiradas de 5e. Todo como **datos**
(`data/*.json`): el juego base y los mods usan el mismo formato.

Estado: plan confirmado por el dueño (2026-11-05, ver "Decisiones del dueño").

## Modelo (propuesta)
- **NPC = mob con diálogo**: en `data/mobs.json`, `"npc": "<id del diálogo>"` y `"disposition"` inicial. Así reusa
  sprite, animaciones, guardado, red y combate. IA nueva `State::Passive`: no caza, deambula cerca de su casilla
  o se queda quieto. Tocarlo = **Hablar** (`Action::Talk(mob)`); en combate no se habla.
- **Disposición** (actitud de 5e: hostil / indiferente / amistoso, como número -2..2): la mueven las tiradas
  sociales y las acciones. Cambia las CD y lo que ofrece el diálogo. Atacarlo lo vuelve hostil (pasa a
  `Hunting`) y baja la disposición de los NPC que lo vean.
- **Diálogos** (`data/dialogs.json`): árbol de nodos. Nodo = `text` + `options`. Opción = `text`, `if`
  (condiciones), `check` (tirada: `{"skill", "dc", "ok", "fail"}`), `do` (efectos), `go` (nodo siguiente o fin).
  - Condiciones: `has` [ítem, n], `gold` n, `flag` [clave, op, valor], `quest` [id, estado], `disposition` n,
    `class` id, `level` n.
  - Efectos: `give` / `take` [ítem, n], `gold` ±n, `flag` [clave, valor], `quest` [id, estado], `disposition` ±n,
    `hostile`, `shop` id, `log` texto (pistas), `map` (revela el piso), `end`.
  - Textos `{"es", "en"}` como el resto del contenido.
- **Pericias 5e** (`data/skills.json`: id -> característica y nombre; Persuasión, Engaño, Intimidación,
  Interpretación, Perspicacia, ...) y `skills` (competencias) en `classes.json`. Tirada = d20 + mod + competencia
  contra la CD, con el dado grande (como las salvaciones). Coquetear = Persuasión o Interpretación, según el
  diálogo.
- **Misiones** (`data/quests.json`): nombre, descripción, objetivos como condiciones (`kill` [mob, n] con el
  contador `kills:<mob>`, `has` ítem, `flag`) y recompensa (efectos). Ventana de misiones en la ficha.
- **Inventario y equipo de los NPC** (pedido del dueño, 2026-11-05): cada NPC lleva mochila, arma y armadura (como el
  héroe): en `mobs.json`, `inventory` ([ítem, cantidad] o sacado del generador) y `weapon` / `armor`. Pelea con su
  equipo, se guarda con el piso y, si muere, lo suelta. Un diálogo puede dar o pedirle cosas a su inventario.
- **Comercio**: la tienda **vende lo que el NPC tiene en su inventario** (así el mercader fijo y el ambulante son lo
  mismo); precio = `price` de `items.json` (nuevo campo), modificado por la disposición. Vender = la mitad y el ítem
  pasa a su inventario. Ventana de tienda con las dos mochilas.
- **Mercader ambulante** (futuro): un NPC con tienda que deambula (`State::Passive` que camina) y puede aparecer en
  cualquier piso; reabastece su inventario cada tanto.
- **Colocación**: `npcs` por piso (como `bestiary.json`), con un rng propio derivado de la seed del piso: no gasta el
  rng del nivel, así los pisos y los mobs siguen saliendo iguales.
- **Red**: `Talk` y `Choose(k)` son acciones como las demás (el host las valida). El diálogo abierto va en el estado
  del héroe: cada jugador ve el suyo y lo importante sale en el log de todos ("X convence a Y").
- **Guardado**: disposición, flags, contadores y misiones.

## Contenido base (del original, con sus sprites)
- **Fabricante de varitas** (`Wandmaker.java`, `wandmaker.png`): el NPC inicial. En el original aparece en las Prisiones
  y pide un objeto a cambio de una varita; acá, en las Alcantarillas, con su misión adaptada.
- **Mercader** (`shopkeeper.png`): en el original aparece en el piso 6; acá, uno en las Alcantarillas.
- Fuera por ahora (decisión del dueño): el Fantasma triste (`Ghost.java`).
- Para después (sprites que ya están en `Assets/Characters/`): blacksmith, imp, ratking, ghost.

## Estado
- **N1** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: NPCs tranquilos, Hablar, diálogos como datos con
  condiciones y efectos, flags de la partida, ventana de diálogo, guardado y red (`docs/game/npcs.md`). Contenido: el
  Mercader en el piso 1 (vende una ración por 10 de oro y el mapa del piso por 30, un rumor) y el Fabricante de
  varitas en el piso 2 (consejos, el ingrediente que busca, regala una poción de curación una vez por partida). Los
  dos se pueden atacar (opción [Atacar], o pegarles). Para probar: tocar al NPC (se acerca y habla), cada opción, que
  las de oro no aparezcan sin oro, el regalo una sola vez (también tras guardar y volver), Esc / Atrás cierra,
  atacarlo (empieza el combate con él sorprendido), que no corte la caminata ni entre al combate de otros mobs, y en
  la LAN que cada uno vea su propio diálogo.

- **N2** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: las 18 pericias de 5e como datos
  (`data/skills.json`) y sus competencias por clase (`skill_prof`: Guerrero Atletismo e Intimidación, Mago Arcanos y
  Perspicacia, Pícaro Engaño, Persuasión, Sigilo y Percepción, Explorador Percepción, Supervivencia y Perspicacia), en
  la ficha; opciones con tirada (`check`) que van a un nodo u otro según el resultado, con el dado grande y el
  resultado en el log de todos; actitud de cada NPC (de hostil a muy amistoso, se ve en la ventana, mueve la CD 2 por
  punto); atacar a un NPC hace desconfiar a los que lo vieron. Medicina ya suma la competencia. Contenido: el
  Fabricante (observarlo con Perspicacia, mentirle con Engaño, coquetearle con Interpretación) y el Mercader
  (regatear con Persuasión, intimidarlo; si es amistoso, un regalo). Cada intento, una vez por partida. Para probar:
  cada tirada con una clase con y sin competencia (el bono del botón cambia), que mentir mal lo vuelva desconfiado y
  suba la CD de lo demás, que regatear bien habilite "¿Algo especial para un amigo?", pegarle a uno delante del otro
  (en pisos distintos no se ven: probarlo con un mod o más adelante), y el dado y el log en la LAN.

- **N3** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: misiones como datos (`data/quests.json`) con objetivos
  de matar (contador por mob) o de tener un ítem, estado de la partida (compartido en la LAN), en la ficha con su
  progreso, y aceptar / revisar / entregar desde los diálogos. Contenido: el Fabricante de varitas (piso 2) pide polvo de
  cadáver, que aparece en unos restos de esqueleto de su piso al aceptar; lo paga con un pergamino de mejora y una
  poción de curación (y se vuelve más amistoso). El Mercader (piso 1) paga 40 de oro por 5 ratas. Para probar: aceptar
  cada una, ver el progreso en la ficha (las ratas cuentan desde que se acepta), abrir los restos y recoger el polvo,
  entregar; guardar y volver a entrar a mitad; en la LAN, que la misión sea del grupo.

- **N4** ✏️ escrito (2026-11-05), pendiente de prueba del dueño: inventario y equipo de los NPC como datos (fijo y al
  azar del generador), con el que pelean y que sueltan al morir; precios en `items.json`; tienda desde el inventario
  (comprar de a una unidad, vender de a una; lo vendido queda en su inventario; la actitud mueve los precios); `give` y
  `take` usan su inventario. El Mercader vende raciones, pociones de curación, pergaminos de identificar, dardos,
  armadura de cuero y algo al azar (2 pociones, 2 pergaminos, 1 arma); pelea con espada corta y armadura de cuero. El
  Fabricante de varitas lleva un bastón. Para probar: "Veamos qué tienes" abre la tienda, comprar con y sin oro, vender
  algo y volver a comprarlo, regatear bien (más barato) o intimidarlo (más caro), la mochila llena, matarlo y ver que
  suelta su mercancía, y en la LAN que cada uno compre de la misma tienda. Pendiente: el mercader ambulante (que
  deambula, aparece en cualquier piso y repone su inventario).

## Pasos (cada uno se prueba a mano)
1. **N1 motor**: NPC pasivo, Hablar, diálogos como datos con condiciones y efectos, flags, ventana de diálogo, red.
2. **N2 social**: pericias y tiradas, disposición, atacar a un NPC.
3. **N3 misiones**: contadores, recompensas, ventana de misiones; el Fabricante de varitas.
4. **N4 inventario y comercio**: inventario y equipo de los NPC, precios, tienda desde el inventario, comprar y
   vender; el Mercader; después, el mercader ambulante.
5. **N5 mods**: el formato en `docs/data.md` y un mod de ejemplo con un NPC propio. Hecho (0.9.12): `docs/mods.md` y `mods/ejemplo-herrero`.
   Probar: copiar `mods/ejemplo-herrero` a `<datos>/mods/` y bajar al piso 3; en la LAN, los dos con el mod (si no, el hash no coincide).

## Decisiones del dueño (2026-11-05)
1. Contenido inicial: el Fabricante de varitas y el Mercader; el Fantasma, por ahora no.
2. Misiones, flags y disposición son **de la partida** (el mundo): en la LAN el grupo las comparte; las recompensas
   quedan en el héroe que las recibe.
3. Tiradas sociales con **pericias 5e** (d20 + característica + competencia de la clase).
4. Atacar a un NPC lo **vuelve hostil** (pelea o huye) y los NPC que lo ven te tratan peor; se lo puede matar.
