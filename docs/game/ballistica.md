# game/ballistica.py

`mechanics/Ballistica.java`: `cast(nivel, desde, hasta, find_char)` devuelve la celda donde se detiene un proyectil
(Bresenham con el error inicial `dA // 2`, como el Java). Se frena en la primera criatura o en lo que bloquea la vista;
contra una pared o algo no transitable queda en la celda anterior. Si devuelve `hasta`, hay línea de tiro.
Lo usan los conjuros; servirá para armas arrojadizas y arcos.
