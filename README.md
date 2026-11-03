# PixelDungeonD&D

Roguelike de mazmorras en pixel art basado en [Pixel Dungeon](https://github.com/watabou/pixel-dungeon) de Watabou,
llevado a las reglas de **Dungeons & Dragons 5e** y con **multijugador cooperativo**.

Bajas por las Alcantarillas con un personaje creado con dados: características, CA, competencias, tiradas de
salvación, iniciativa, conjuros y rasgos de clase de 5e, sobre la mazmorra, los monstruos y el arte del original.

> Proyecto de fans, libre y sin fines de lucro. No está afiliado ni respaldado por Watabou ni por Wizards of the Coast.

## Estado

- **`main`**: el juego en **Rust**, en migración (plan: [`roadmap/migracion-rust.md`](roadmap/migracion-rust.md)).
  Mientras empieza la fase R0, esta rama todavía tiene el código del prototipo.
- **`python-prototipe`**: el prototipo jugable en Python (0.8), congelado como referencia del port: pisos 1-4,
  creación de personaje con dados, combate 5e por iniciativa (movimiento + acción), Guerrero, Mago, Pícaro y
  Explorador, salvaciones contra la muerte, inventario, guardado y modo de 2 jugadores en la misma PC.

## Meta

- **Una experiencia D&D 5e sobre Pixel Dungeon**: cuando chocan, **5e manda** sobre los números de PD.
- **Cooperativo**: host autoritativo; en la 1.0, partidas en LAN. Cada jugador guarda su personaje y lo lleva a la
  partida de cualquier host. Después: internet (relay y servidores propios).
- **Todo por GPU**: 200+ FPS incluso en la peor máquina.
- **Linux, Windows y Android** desde la misma base.
- **Contenido como datos** (`data/*.json`): clases, mobs, ítems, estados y conjuros se cambian sin tocar código.
  Es la base de los mods.

## Decisiones del proyecto

- **Libre**: GPL v3, igual que el original. Todo lo derivado se distribuye con su código.
- **Arte hecho por personas**: sólo se usan los assets del original o assets hechos por personas, con su autor y
  licencia anotados en [`docs/assets.md`](docs/assets.md). Nada de arte generado: si un sprite falta, el juego dibuja
  el nombre del objeto hasta que alguien lo haga.
- **No hablar en nombre de nadie**: no es un juego oficial de Pixel Dungeon ni de D&D. El contenido propio usa IDs
  `pddnd:` (nunca `pd:`), y los textos y marcas de terceros se atribuyen.
- **Crédito a cada autor** (ver abajo).
- **Reglas 5e por defecto**: las variantes son *reglas de la partida* opcionales que elige el host, nunca sorpresas.

## Compilar

### Rust (`main`)

Se completa al cerrar la fase R0. Lo previsto:

```sh
rustup toolchain install stable
cargo run --release            # escritorio (Linux / Windows)
```

Android: `rustup target add aarch64-linux-android`, Android NDK y la firma de la sección siguiente.

### Prototipo en Python (`python-prototipe`)

Requiere Python 3.14 y una GPU con OpenGL 3.3.

```sh
git switch python-prototipe
python -m venv venv
source venv/bin/activate          # fish: source venv/bin/activate.fish · Windows: venv\Scripts\activate
pip install -r requirements.txt   # pygame-ce, moderngl
python main.py                    # siempre desde la raíz (las rutas de Assets/ son relativas)
```

Opciones en `Local.config` (resolución, FPS, idioma; `[dev]` para desarrollo).

### Firma de Android

La clave de firma **nunca** va al repositorio (`.gitignore` ignora `*.jks`, `*.keystore` y `keystore.properties`).
Vive fuera, en `~/.pddnd-signing/`:

```properties
# ~/.pddnd-signing/keystore.properties
storeFile=/ruta/a/release.jks
storePassword=...
keyAlias=pddnd
keyPassword=...
```

En CI se carga desde secretos de GitHub. Si se pierde la clave no se pueden publicar actualizaciones del mismo APK:
hay que respaldarla.

## Contribuir

1. Lee [`roadmap/00-plan.md`](roadmap/00-plan.md) y [`roadmap/migracion-rust.md`](roadmap/migracion-rust.md).
2. Ramas: el trabajo diario va en `dev`; al cerrar una fase, `dev` se une a `main`. `python-prototipe` sólo recibe
   arreglos que hagan falta como referencia.
3. Commits cortos con su tipo: `feat: …`, `fix: …`, `merge: …`.
4. Reglas del código:
   - La lógica del juego no depende del render ni de la ventana; el render lee el estado del juego, no al revés.
   - Contenido nuevo en `data/*.json`, con IDs `namespace:nombre` y textos `{"es": …, "en": …}`.
   - Todo lo que cambia con el tiempo se multiplica por el delta: el juego se ve igual a cualquier FPS.
   - Todo azar de juego pasa por un rng con seed, y nada cuyo orden sea aleatorio (como un `HashMap`) decide un
     resultado: la misma seed da el mismo nivel en todos los jugadores.
   - Cada archivo de código tiene su `.md` en `docs/` (misma ruta): qué hace, de dónde viene, decisiones y trampas.
5. Arte y sonido: sólo hechos por personas, con licencia compatible con GPL v3, anotados en `docs/assets.md`.
6. Pruebas: el juego se prueba a mano. Sólo la red tiene pruebas automáticas.
7. Al contribuir aceptas que tu aporte se distribuya bajo GPL v3.

Errores e ideas: en los issues de GitHub. Cambios: pull request hacia `dev`.

## Créditos y licencias

- **Pixel Dungeon** © 2012-2015 Oleg Dolya ([Watabou](https://github.com/watabou/pixel-dungeon)), GNU GPL v3.
  Base del juego: diseño, generación de niveles y monstruos (portados de su código), y los gráficos, sonidos y
  fuentes de `Assets/`.
- **Música**: Cube_Code (la de Pixel Dungeon).
- **Reglas de D&D 5e**: este trabajo incluye material del System Reference Document 5.1 ("SRD 5.1") y del System
  Reference Document 5.2 ("SRD 5.2") de Wizards of the Coast LLC, disponibles en
  <https://dnd.wizards.com/resources/systems-reference-document> y <https://www.dndbeyond.com/srd>. El SRD 5.1 y el
  SRD 5.2 están bajo la licencia Creative Commons Attribution 4.0 International,
  <https://creativecommons.org/licenses/by/4.0/legalcode>.
- *Dungeons & Dragons* y *D&D* son marcas de Wizards of the Coast LLC. *Pixel Dungeon* es de Watabou. Se nombran sólo
  para indicar el origen; este proyecto no está afiliado a ninguno de los dos.
- **Prototipo en Python**: [pygame-ce](https://github.com/pygame-community/pygame-ce) (LGPL 2.1),
  [moderngl](https://github.com/moderngl/moderngl) y [glcontext](https://github.com/moderngl/glcontext) (MIT).
- **Rust**: [winit](https://github.com/rust-windowing/winit), [wgpu](https://github.com/gfx-rs/wgpu) y
  [serde](https://serde.rs) (MIT / Apache-2.0). La lista final queda en este README al cerrar R0.

Código de este proyecto: **GNU GPL v3**, ver [`LICENSE`](LICENSE).
