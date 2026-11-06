# android/build.sh + AndroidManifest.xml

APK sin Gradle ni Java: `NativeActivity` carga `libapp.so` (`android.app.lib_name = app`).

1. `cargo ndk -t arm64-v8a` compila el crate `app` como cdylib en `target/android/apk/lib/arm64-v8a/`.
2. `aapt2 link` arma el APK con el manifiesto contra el `android.jar` más nuevo del SDK.
3. Se agrega `lib/`, `zipalign -p 4` y `apksigner` con la clave de `PDRI_KEYS` (por defecto
   `~/.pddnd-signing/keystore.properties`). La contraseña pasa por variable de entorno, nunca por argumento.

- Por defecto usa el SDK y el NDK que dejó la prueba de buildozer en `~/.buildozer/android/platform/`; en CI,
  `ANDROID_HOME` y `ANDROID_NDK_LATEST_HOME` del runner.
- Sólo `arm64-v8a` (todos los teléfonos actuales). minSdk 26, targetSdk 35, orientación `fullUser` (gira con el teléfono y respeta el bloqueo de rotación; sin reiniciar la actividad), GLES 3 requerido (wgpu usa Vulkan si
  hay, si no GLES).
- Permisos de red ya declarados para R6 (incluye `CHANGE_WIFI_MULTICAST_STATE` para el MulticastLock).
- Instalar: `adb install -r target/android/pdri.apk`; abrir: `adb shell am start -n
  io.github.notsumechan.pdri/android.app.NativeActivity`; log: `adb logcat -s RustStdoutStderr:* app:*`.
- `hasFragileUserData="true"` (Android 10+): al desinstalar, el sistema ofrece "Conservar datos de la app"
  (desmarcada). Cubre `internal_data_path`, donde van `Local.config`, personajes y partidas. Al reinstalar, los datos
  vuelven si el APK está firmado con la misma clave.
- CI: el job `release` de `.github/workflows/build.yml` publica en cada push a `main` la Release `v<versión>` con
  `pdri-linux-x86_64`, `pdri-windows-x86_64.exe` y `pdri-android.apk`; no repite un tag y falla sin la clave del dueño.
- `MainActivity` también maneja el teclado del sistema: `setKeyboard` (lo llama Rust) y `dispatchKeyEvent`, que pasa
  cada letra a Rust por `nativeChar` (ver `docs/app/lib.md`).
- `PDRI_FEATURES=updater`: compila con el actualizador y agrega al manifiesto (una copia en `target/android/`) el permiso
  `REQUEST_INSTALL_PACKAGES`; la build normal (y la de Play) no lo lleva. `MainActivity.installApk` instala con
  PackageInstaller y su `onCreate` registra el receptor que abre la confirmación de Android.
