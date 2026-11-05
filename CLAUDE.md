# PixelDungeon-RollInitiative

Roguelike basado en Pixel Dungeon (pisos 1–4, Alcantarillas) con reglas de D&D 5e y multijugador cooperativo.
En **Rust** (rama `main`). Referencias para portar:
- Prototipo en Python (0.8): rama `python-prototipe` — la referencia directa del port.
- Java original: `~/Documents/programacion/personal/java/pixel-dungeon/src/com/watabou/`.
Plan y fases: `roadmap/migracion-rust.md` (leer primero: tiene el estado de cada fase, lo siguiente y lo
pendiente del dueño).

## Arranque
```bash
cargo run --release      # escritorio; los assets van dentro del binario (include_bytes!)
android/build.sh         # APK firmado -> target/android/pdri.apk (necesita cargo-ndk, SDK, NDK, la clave)
adb install -r target/android/pdri.apk
cargo run -q -p game --example level -- <seed> <piso>   # volcado ASCII de un piso (comparar con el prototipo)
```

## Estructura
- `game/` lógica pura: **nunca** depende de wgpu, winit ni nada de render. Corre headless (host, herramientas).
- `app/` ventana (winit), render (wgpu), UI, pantallas, red. Lee el estado de `game`, nunca al revés.
  `gpu.rs` (dos pasadas: sprites -> textura virtual -> pantalla), `atlas.rs`, `font.rs`, shaders `.wgsl`.
  `net.rs` red LAN (std, hilos y canales): UDP 47810 para encontrar partidas, TCP 47811 para jugar.
- `android/` manifiesto y `build.sh` (sin Gradle). `.github/workflows/build.yml` CI: Linux, Windows, APK.
- `data/` contenido en JSON · `Assets/` del original (GPL v3) · `roadmap/` plan y decisiones · `docs/` un `.md` por
  archivo de código, misma ruta (`docs/app/gpu.md` <-> `app/src/gpu.rs`).

## Reglas
- Todo el dibujo por GPU, en lotes: sprites como instancias de un atlas, 1-2 draw calls por frame. Nada de dibujar
  pixel a pixel en la CPU. Meta: 200+ FPS en la peor máquina.
- Se dibuja en resolución virtual (~320x180, pixeles del juego) y se escala con factor entero y NEAREST.
- Todo lo que cambia con el tiempo se multiplica por el delta (segundos). Nunca "x por frame".
- Contenido como datos en `data/*.json` (campos en `docs/data.md`): se cambia el JSON, no el código. Base de los mods.
- IDs de contenido de texto `namespace:nombre`; el juego base usa `pdri:`. Nada de IDs numéricos.
- Textos de contenido: `str` o `{"es": ..., "en": ...}`. No reemplazar un idioma por otro: agregar el que falte.
- 5e manda sobre los stats de PD. Toda tirada usa el rng con seed del juego; el azar puramente visual puede no.
- Determinismo (misma seed = mismo nivel en todos los peers): nada de `HashMap`/`HashSet` cuyo orden decida un
  resultado (`Vec`, `BTreeMap`). Ojo: `/` de Rust trunca hacia 0 (como Java); `//` de Python redondea hacia abajo.
- Arte: sólo el del original o hecho por personas, anotado en `docs/assets.md`. Nunca generar sprites: sin sprite,
  se dibuja el nombre.
- Audio al final: no agregarlo hasta que se pida.
- Sin tests de juego: el dueño prueba a mano. Tests automáticos sólo de red (`app/src/net.rs`:
  `cargo test -p app --lib net`). No ejecutar el juego, ni simulaciones,
  bots o comparaciones de partidas (aunque sean descartables): sólo compilar (escritorio y `cargo ndk ... check`).
- Trabajo diario en la rama `dev`; `merge: dev -> main` al cerrar cada fase, cuando el dueño la validó. El dueño pushea.
- Al cerrar lo que cambie la versión, subirla en `Cargo.toml` y `android/AndroidManifest.xml` (regla en
  `roadmap/versiones.md`). **Cada cambio, aunque sea chico, sube el último número** (0.9.0 -> 0.9.1 -> 0.9.2...),
  con su fila en la tabla de `versiones.md` y su commit (pedido del dueño, 2026-11-05). Un tema = una versión: más
  arreglos del mismo tema van en la misma versión, no en una nueva. `Local.config` cambia al probar opciones: no commitearlo.
- Commits cortos, a nombre del dueño, sin líneas de coautoría: `feat: …`, `fix: …`, `merge: …`, terminando con
  `#` y el paso del roadmap al que pertenecen (`feat: atlas de sprites #R0`, `fix: niebla #R2`).
- La clave de firma nunca entra al repo (`~/.pddnd-signing/`).
- Al terminar algo de una fase, actualizar `roadmap/migracion-rust.md`. Al crear o tocar un archivo de código,
  crear/actualizar su doc en `docs/`.
- Proyecto derivado de Pixel Dungeon (GPL v3): todo se distribuye como GPL v3.
