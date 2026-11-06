# 🎲 PixelDungeon-RollInitiative

![Licencia](https://img.shields.io/badge/licencia-GPL%20v3-blue)
![Plataformas](https://img.shields.io/badge/plataformas-Linux%20·%20Windows%20·%20Android-green)
![Rust](https://img.shields.io/badge/hecho%20en-Rust-orange)

Roguelike basado en [Pixel Dungeon](https://github.com/watabou/pixel-dungeon) con reglas de **D&D 5e** y
**multijugador cooperativo**.

> Proyecto de fans, libre y sin fines de lucro. No está afiliado a Watabou ni a Wizards of the Coast.

## ✨ Qué tiene
- Personajes creados con dados y las 12 clases de 5e: CA, salvaciones, iniciativa, conjuros y rasgos.
- Las Alcantarillas (pisos 1-4) con la generación, los monstruos y el arte del original.
- Cooperativo en LAN entre PC y teléfono. Cada uno lleva su personaje a la partida de cualquiera.
- NPCs con diálogos, misiones, tiendas y tiradas sociales.
- Mods: contenido en JSON y scripts Lua, importables desde un zip.
- Todo dibujado por GPU, en Linux, Windows y Android.

## 📥 Descargar
Desde [Releases](../../releases).

## 🛠️ Compilar
```sh
cargo run --release                                   # escritorio
android/build.sh && adb install -r target/android/pdri.apk   # Android (cargo-ndk, SDK, NDK y la clave)
```
Las opciones se guardan en `Local.config`, en la carpeta de datos (`%APPDATA%/PixelDND` o `~/.local/share/PixelDND`). La clave de firma de Android vive fuera del repo, en `~/.pddnd-signing/` (en CI, desde
secretos de GitHub): respaldarla, porque sin ella no se puede actualizar el APK.

## 🗺️ Roadmap
En [`roadmap/`](roadmap/README.md). La meta a largo plazo es convertirse en un juego propio, reemplazando de a poco lo
que viene de Pixel Dungeon. Una rama conservará la versión con el core de PD.

> 🐍 Hay un **prototipo en Python** (0.8) en la rama [`python-prototipe`](../../tree/python-prototipe), con sus planes
> originales. Está congelado y sirvió de referencia para el port.

## 🤝 Contribuir
- Pull requests hacia `dev`. Errores e ideas, en los issues.
- Commits cortos: `feat: …`, `fix: …`, `merge: …`.
- La lógica (`game/`) no depende del render (`app/`). El contenido va en `data/*.json` con IDs `namespace:nombre`.
- Todo azar pasa por el rng con seed (la misma seed da el mismo nivel en todos los jugadores).
- Cada archivo de código tiene su `.md` en `docs/`, con la misma ruta.
- Arte y sonido sólo hechos por personas, con licencia compatible, anotados en [`docs/assets.md`](docs/assets.md).
  Si falta un sprite, el juego dibuja el nombre.
- Al contribuir aceptas que tu aporte se distribuya bajo GPL v3.

## 📜 Créditos y licencias
- **Pixel Dungeon** © 2012-2015 Oleg Dolya ([Watabou](https://github.com/watabou/pixel-dungeon)), GNU GPL v3: el
  diseño, la generación de niveles y los monstruos, y los gráficos, sonidos y fuentes de `Assets/`.
- **Música:** Cube_Code (la de Pixel Dungeon).
- **Nombre:** gracias a [chomusuke-png](https://github.com/chomusuke-png) por la idea de *Roll Initiative*.
- **Reglas de D&D 5e:** este trabajo incluye material del System Reference Document 5.1 ("SRD 5.1") y del System
  Reference Document 5.2 ("SRD 5.2") de Wizards of the Coast LLC, disponibles en
  <https://dnd.wizards.com/resources/systems-reference-document> y <https://www.dndbeyond.com/srd>. El SRD 5.1 y el
  SRD 5.2 están bajo la licencia Creative Commons Attribution 4.0 International,
  <https://creativecommons.org/licenses/by/4.0/legalcode>.
- *Dungeons & Dragons* y *D&D* son marcas de Wizards of the Coast LLC. *Pixel Dungeon* es de Watabou. Se nombran sólo
  para indicar el origen.
- **Dependencias:** [winit](https://github.com/rust-windowing/winit), [wgpu](https://github.com/gfx-rs/wgpu),
  [serde](https://serde.rs) y las demás de los `Cargo.toml`, cada una con su licencia.

Código de este proyecto: **GNU GPL v3**, ver [`LICENSE`](LICENSE).
