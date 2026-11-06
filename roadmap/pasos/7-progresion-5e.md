# P — Personaje 5e: creación y progresión

Pedido del dueño (2026-11-06): al subir de nivel, los conjuros y rasgos nuevos de cada clase como en 5e; con qué
parte cada personaje (competencias, equipo inicial, oro). Todo configurable: es parte de la "experiencia de rol" y
del camino para separarse de Pixel Dungeon (hoy la progresión es la de PD: XP fija por mob y `max_exp` = 5 + nivel x 5).
Sólo contenido del SRD 5.2. Es grande: cada punto, versiones chicas que se prueben por separado.

## Qué hay hoy
- `data/classes.json`: dado de golpe, competencias (armas, armaduras, salvaciones, habilidades), arma y armadura
  iniciales, ítems, rasgos (`traits`), prioridad de características. Al subir de nivel (`Hero::gain_level`): PV,
  competencia y espacios de conjuro (`level_up_spells`). Sin elección del jugador.
- Rasgos pendientes (lista en `pendientes.md`, "Combate 5e"): Inspiración bárdica, Sentido divino, Canalizar
  divinidad, golpe extra, Forma salvaje, Castigo divino, invocaciones, metamagia.

## P1 — Inicio por clase, como datos
- Equipo inicial con opciones 5e ("A o B") o el oro de la clase en su lugar; oro inicial (fijo o en dados).
- Competencias de habilidad a elegir (N de una lista de la clase) en la creación del personaje.
- `docs/data.md` al día; API de mods estable (sólo sumar campos).

## P2 — Subir de nivel
- Tabla por clase y nivel como datos: rasgos que se ganan, espacios de conjuro, conjuros conocidos / preparados.
- Ventana al subir de nivel con lo que se elige: conjuros nuevos, subclase (nivel 3), mejora de característica o
  dote (nivel 4, 8...). Con PV tirado o fijo según la regla de la partida.
- XP de 5e (por CR del monstruo y tabla de niveles) en vez de la de PD, o por hitos (regla de la partida).

## P3 — Rasgos de clase
- Los pendientes, de a uno por clase, como datos donde se pueda (`traits` + scripts) y en código donde no.

## P4 — Configurable (reglas de la partida)
- En `Rules` y la ventana de reglas: XP por combate o por hitos, PV fijos o tirados, oro o equipo inicial, dotes
  sí / no, nivel inicial. El host decide en la LAN.
