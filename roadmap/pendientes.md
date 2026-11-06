# Pendientes

Sin orden. Cuando el dueño elige algo, pasa a `pasos/`.

## Red
- Aplicar el estado sin rearmar la partida (hoy el cliente regenera los pisos en cada estado). Por internet importa:
  hacerlo con D2 o justo después. Luego, mandar diferencias.
- El teléfono como host (MulticastLock). Con la dirección a mano deja de ser urgente.
- Log, dados y diálogos llegan en el idioma del host.

## Combate 5e
- Correr, Destrabarse, Esquivar, ataques de oportunidad, moverse después de actuar.
- `speed_ft` e `initiative` por mob; guardar el combate en curso.
- Oleada de Acción, estilo de combate del Explorador, Percepción para buscar, Sigilo, descansos con dados de golpe.
- Conjuros: concentración, foco, más niveles, conos.
- Rasgos de clase para después: Inspiración bárdica, Sentido divino, Canalizar divinidad, golpe extra, Forma salvaje,
  Castigo divino, invocaciones, metamagia.
- Atribución SRD en el About si se usan textos del SRD.

## Ítems y mobs
- Pociones y pergaminos restantes, varitas, anillos, semillas, bombas; ítems de clase (varita del Mago, anillo del
  Pícaro); identificar con el uso.
- Swarm, Skeleton, Thief, mimic, FLEEING.

## Pulido
- Partículas, proyectiles en vuelo, nubes de gas. Indicador de ocupado que gire.
- Vertical: "cuesta interactuar" (preguntar qué pantalla).
- Audio: sólo cuando se pida.

## NPCs y mods
- Lista de mods (pedido del dueño, 2026-11-06):
  - Sin botón AJUSTES: tocar el mod abre su menú (ajustes, descripción, borrar).
  - Vertical: el botón ON / OFF tapa la descripción.
  - Horizontal: la lista no usa el ancho de la pantalla, sólo se achata.
  - Scroll, ver los importados sin reiniciar, borrar un mod.
- Precios por nivel e identificación; tiradas sociales que esperen al dado (`ponytail` en `game/src/npcs.rs`).
- Más NPC: imp, rey rata, herrero (el fantasma, no por ahora).
- `.pdmod` con versión y sha256, packs con nombre y versión (`instances/<pack>/pack.json`), mods del host al unirse (caché por sha256, se bajan
  del host), mods que necesita cada personaje.

## Controles y visión
- **Movimiento en PC:** sólo mouse o sólo teclado, y línea punteada con el camino (también en el teléfono).
- **Luz y visión 5e:** luz brillante, penumbra y oscuridad por piso o sala; antorchas y linternas con radio;
  visión en la oscuridad; los mobs también ven según la luz. La niebla suma la penumbra como tono intermedio.

## Separarse de Pixel Dungeon
Meta a largo plazo ([`decisiones.md`](decisiones.md#meta-un-juego-propio-2026-11-06)). La rama `pd-core` ya existe
(0.9.25): desde ahí, `dev` se separa.

| Qué | Cómo |
|---|---|
| Mobs | Monstruos del SRD 5.2 (sólo los que están en el SRD) |
| Ítems | Pociones, pergaminos y armas del SRD |
| Generación | Generador propio para las zonas (M1), en vez de las Alcantarillas |
| Código portado | ✅ FOV (symmetric shadowcasting), pathfinder y línea de tiro propios (0.9.28). Falta: turnos fuera de combate (planificador de PD) |
| UI | Layout propio (con la conversación de estética) |
| Arte | Tiles, personajes, ítems y UI hechos por personas, de a uno |
| Fuente | ✅ **Silkscreen** (OFL) en vez de `font1x.png` de PD (0.9.27) |
| Sonido | Propio desde el principio (todavía no hay audio) |
| Cierre | Borrar `Assets/` del original, revisar que no quede nada derivado, nombre nuevo |

## Distribución (sin fecha)
- **Steam:** primero sin SDK. Después, opcional, feature `steam`: amigos, Steam Networking, logros, nube (revisar
  licencia antes).
- **Play Store:** AAB con `bundletool`, política de privacidad, Data safety, clave en Play App Signing, sin `updater`.
- Exportar e importar partidas a un archivo.
- Enlace de donación (GitHub Sponsors o Ko-fi) en el README y en "Acerca de".
- Instalador de Windows con "Borrar partidas guardadas".
- Relay propio o servidor dedicado (sacar `ServerGame` de `app/`).
