# net/actions.py

El input del jugador como datos: celdas e índices de ítems (`hero.all_items()`), nunca objetos. El cliente arma el
mensaje (`handle`, `simple`, `use`, `throw`, `cast`) y el host lo pasa por `apply(hero, msg)`, que valida (celda en
rango, ítem que existe, verbo conocido, conjuro registrado, Segundo Aliento sólo con el rasgo) y pone `hero.action`
igual que la UI local. Lo inválido se ignora. Después el juego vuelve a validar al actuar (p. ej. `spells.cast`).
Verbos = los de `gameUi.Verbs`: si se agrega uno allá, agregarlo en `VERBS`.
