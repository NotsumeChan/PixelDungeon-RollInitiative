# app/src/mods.rs

Importar mods desde un zip. `pick(datos)`: en escritorio, el diálogo de archivos del sistema (`rfd`, filtro zip) y la
instalación en el acto; en Android llama por JNI a `MainActivity.pickZip` (`android/java`), que abre el selector de
documentos y copia el zip elegido a `<datos>/mod_import.zip` (y el nombre a `mod_import.name`); `poll` lo levanta
desde la pantalla de título. `import_zip`: sólo `<registro>.json` y `script.lua` (`registry::is_mod_file`), en la raíz
o dentro de una carpeta; `enclosed_name` descarta rutas con `..`; tope `MAX_BYTES`. `installed`: las carpetas de
`<datos>/mods`. Los mods se cargan al arrancar: después de importar hay que reiniciar.
