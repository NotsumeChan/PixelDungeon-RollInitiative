# screens/createScreen.py

Creación de personaje estilo D&D 5e. Se entra desde la lista de personajes (botón NUEVO).

## Pantalla
0. **Nombre:** click en el recuadro y escribir (máx. 16). Vacío = nombre de la clase.
1. **Clase:** las 4 de `HERO_CLASSES`, con avatar, nombre, clase 5e y dado de golpe. Click para elegir.
2. **Características:** FUE DES CON INT SAB CAR, con valor y modificador.
3. **Drag and drop:** arrastrar una tirada a una característica la asigna; arrastrar entre características las
   intercambia; arrastrar una característica afuera la quita. Un click sin mover sigue funcionando como antes.
4. **Tiradas:** botón TIRAR = 7 tiradas de 4d6 quitando el menor (`dice.roll_ability_pool`).
   - Click en una tirada y luego en una característica para asignarla. Si esa tirada ya estaba en otra
     característica, se mueve. Sobra una; las usadas se ven grises.
   - Al elegir una tirada se ven sus 4 dados, con el descartado entre corchetes.
4. **Escribir a mano:** click en una característica (sin tirada elegida) y escribir el número (1–20).
   Backspace borra; Enter o Tab pasa a la siguiente. Escribir pisa lo que venía de una tirada.
5. **Botones:** TIRAR, LIMPIAR, EMPEZAR (sólo con las 6 características válidas). Esc vuelve al título.

## Al empezar
Guarda el personaje en disco (`game/characters.py`), lo pone en `Globals["character"]`, crea la partida (`gameScreen.NewGame`) y entra
al juego **después** de cerrar su propio loop, para no anidar pantallas.

- Las tiradas usan un rng nuevo con seed al azar. La creación es local; lo determinista empieza con la partida.
- Usa la UI inmediata de `gameUi` (`Hot`, `Hotspot`, `Click`, `DrawButton`).
- Pendiente: nombre del personaje, lista de personajes guardados (ver `roadmap/futuro-mods-personajes-red.md`).
