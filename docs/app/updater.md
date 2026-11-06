# app/src/updater.rs

Actualizador (D4). Sólo con la feature `updater` (`ENABLED`): la prenden las builds de la CI (Releases de GitHub) y
quien compila con `--features app/updater`. Nunca en Play Store ni Steam: actualizan sus tiendas y Play no acepta el
permiso de instalar APKs. Sin la feature, `check` e `install` devuelven `None` y la UI no muestra nada.

- **Al abrir** (`check`, si `[general] check_updates` no es `false`; opción "Actualizaciones" en Configuración): en un
  hilo, la última Release (`releases/latest`, sólo estables) y, si es más nueva que esta build (`net::newer`),
  `Msg::Found`. El título muestra "Versión X disponible" con ACTUALIZAR y LUEGO (`Ui::draw_update`).
- **Actualizar** (`install`): baja la Release `v<versión>` y su archivo de esta plataforma (`ASSET`, los nombres del
  job `release`). Escritorio: reemplaza el ejecutable (Windows renombra el que corre a `.old`, que `cleanup` borra al
  próximo arranque) y la app se reinicia con el nuevo. Android: `update.apk` en la carpeta de datos y
  `MainActivity.installApk` (PackageInstaller; Android pide confirmar y exige la misma clave).
- **Al unirse (D3):** si el host tiene un protocolo distinto y una versión más nueva, Unirse muestra ACTUALIZAR, que baja
  **la versión del host**, no la última.
- **Repo:** `PDRI_REPO` al compilar (la CI pone el suyo: un fork se actualiza de sus Releases); si no, el del juego.
- **Canal beta (testers):** compilado con `PDRI_CHANNEL=beta`, también ve las pre-releases (`releases?per_page=20`, la
  más nueva). La CI publica una en cada push a `dev` (`v<versión>-beta-<letra>`: a, b, ... z, aa; la letra vuelve a `a`
  con cada versión; job `beta-name` la calcula antes de compilar, `beta` publica, se guardan 3) y esas builds llevan
  su letra (`PDRI_BUILD` = `beta-a`, `current()`; el título muestra "v 0.9.26 beta-a"), así cada build nueva de dev
  se ofrece aunque la versión no cambie. `net::newer`: una beta va antes que su estable. Compilar una a mano:
  `PDRI_CHANNEL=beta cargo build --release --features app/updater` (Android:
  `PDRI_CHANNEL=beta PDRI_FEATURES=updater android/build.sh`).
- ponytail: confía en HTTPS y en el repo (sin firma propia del binario).
