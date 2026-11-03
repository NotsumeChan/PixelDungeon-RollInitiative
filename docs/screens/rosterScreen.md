# screens/rosterScreen.py

Lista de personajes: se abre con **Play** en el título (antes Play iba directo a la partida).

- Muestra los personajes guardados (`game/characters.py`): avatar, nombre, clase 5e y dado de golpe, o
  "en partida - piso N, nivel N" si tiene una partida viva en memoria, y sus 6 características.
- Rueda del mouse para desplazarse (4 visibles). Click para elegir.
- Sin personajes (o sin uno elegido), JUGAR y BORRAR se ven grises y no responden.
- **JUGAR:** sigue la partida en memoria de ese personaje o empieza una nueva.
- **NUEVO:** abre la creación (`createScreen.Create`).
- **BORRAR:** pide confirmación (el botón pasa a CONFIRMAR); borra del archivo y su partida en memoria.
- Esc: volver al título.
- Un personaje cuya clase no existe (por ejemplo, de un mod que no está cargado) se muestra pero no se puede
  dibujar ni jugar.
- La pantalla siguiente (juego o creación) se abre **después** de cerrar su loop, para no anidar pantallas.
- Pendiente: las partidas no se guardan en disco; al cerrar el juego se pierden (los personajes no).

## Partidas guardadas
`Run(char)`: la partida en memoria o, la primera vez, la guardada en disco (`save.load_run`); se cachea en
`Globals["runs"]` (también el None, para no leer el disco en cada frame). Borrar un personaje borra su guardado.
