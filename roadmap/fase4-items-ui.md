# Fase 4 — Ítems, inventario y UI

## Estado
- ✅ Escrito, pendiente de revisión manual del dueño.

## Qué entró (primera pasada)
- **Ítems** (`game/items.py`): oro, comida, 3 pociones, 5 pergaminos, las 11 armas cuerpo a cuerpo y las 5
  armaduras. Hay identificación (colores y runas al azar por partida), niveles ±, maldiciones y montones en el
  suelo (heap, cofre, esqueleto, tumba).
- **Generación por piso** (`game/dungeon.py`): 3+ ítems al azar, una comida garantizada, cuotas de poción de
  fuerza y pergamino de mejora (`posNeeded` / `souNeeded`), tumbas del cementerio, loot de Gnoll (oro) y
  Crab (carne).
- **Héroe** (`game/hero.py`): arma y armadura reales (daño, precisión, demora, bloqueo, penalización por falta de
  STR), mochila de 19, oro, kit inicial de cada clase, hambre (hambriento 260, famélico 360) y acciones:
  recoger o abrir, comer, beber, leer (con objetivo), equipar, quitar, soltar.
- **UI** (`screens/gameUi.py`): barra de estado, barra de herramientas, mochila y ventana de ítem, con los
  assets originales. Tecla I = mochila.
- IDs de texto con prefijo `pddnd:` en todo el contenido (ítems, mobs, clases).

## Qué falta (segunda pasada)
- Pociones con buffs o gases (toxic gas, paralytic, liquid flame, levitation, mind vision, purity, invisibility,
  frost), pergaminos restantes (recharging, challenge, terror, lullaby, psionic blast, mirror image, enchantment).
- Varitas, anillos, semillas y plantas, bombas. Lanzar ya existe (armas arrojadizas, `Hero._throw`) y los estados
  también (`game/buffs.py`).
- Ítems de clase: varita de misil mágico (Mage), anillo de sombras (Rogue). (Dardos y bumerán: hechos.)
- Mimic, mystery meat con efectos, Skeleton que suelta armas, Thief que roba.
- Equipo que se identifica con el uso en vez de al equiparlo.
- (Ranura rápida, íconos de estados, examinar y números de daño: hechos.)
