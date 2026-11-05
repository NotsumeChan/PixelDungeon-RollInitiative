# game/src/spells.rs

`game/spells.py`: conjuros de 5e como datos (`data/spells.json`) y los rasgos de clase.

- Números del lanzador: bono de ataque = competencia + mod de `spellcasting.ability`; CD = 8 + bono. Espacios por
  nivel de personaje (lanzador completo: Mago; medio: Explorador, reglas 2024); se recargan al llegar a un piso nuevo
  (`recover`, hasta que haya descansos) y suben con el nivel (`level_up_spells`).
- `cast(héroe, k, celda)` (acción `Cast`): valida el objetivo (enemigo a la vista con línea de tiro, casilla vista,
  uno mismo o un aliado al lado), gasta un uso gratis o el menor espacio que alcance (los niveles de más suben el
  conjuro) y lo resuelve por `kind`: attack (d20 + bono - cobertura vs CA; 20 crítico; trucos x2/x3/x4 en los niveles
  5/11/17; estado al acertar), auto (dardos que aciertan solos), sleep (duerme a los de menos vida dentro del radio
  hasta gastar la reserva), heal (dados + mod), mark (Marca del cazador: quita la anterior). Lo que se ve espera al
  dado (`Pending::Attack` / `Buff` / `Sleep` / `Heal`). `bonus` = no gasta tiempo.
- Con armadura sin competencia no se lanzan conjuros (5e).
- Rasgos: Segundo Aliento (Guerrero, acción `SecondWind`: 1d10 + nivel, una vez por recarga), Ataque Furtivo (Pícaro:
  +1d6 cada 2 niveles con ventaja y arma sutil, en `roll_damage`), Enemigo predilecto (Explorador: Marca del cazador
  gratis 2 + (nivel - 1) / 4 veces; la marca suma +1d6 a los ataques con armas).
- `shot`: la línea de tiro de un héroe (pasa por los aliados: media cobertura, -2).
- Faltan: proyectiles y partículas de los conjuros, concentración, foco.

Clases de 5e (0.9.3): tipos nuevos `save` (salvación contra la CD del lanzador: Llama sagrada, Burla cruel, Rociada
venenosa; `half` = la mitad si se salva) y `buff` (un estado sobre uno mismo: Furia), `amount` en `heal` (curación fija:
Imposición de manos) y la tabla `pact` de espacios (Brujo: pocos, del nivel más alto). Los rasgos con usos (`features`
de la clase: [conjuro, base, cada]) son conjuros de nivel 1 con usos por descanso en vez de espacios; no piden
competencia con la armadura. El +1d6 de las marcas sale del flag `mark` del estado (Marca del cazador, Maleficio).
