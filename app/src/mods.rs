//! Importar mods desde un zip (`docs/mods.md`): el selector de archivos de cada plataforma (escritorio: `rfd`;
//! Android: `MainActivity.pickZip`, que copia el zip elegido a `<datos>/mod_import.zip`) y la descompresión en
//! `<datos>/mods/<mod>/`. Los mods se cargan al arrancar: después de importar hay que reiniciar.
use std::io::{Cursor, Read};
use std::path::Path;

/// Tope de lo que se descomprime (un zip bomba no llena el teléfono).
const MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Las carpetas de `<datos>/mods`, en orden (el de carga).
pub fn installed(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir.join("mods"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// Sólo letras, números, `-` y `_` (el nombre de la carpeta del mod).
fn clean(s: &str) -> String {
    let s: String = s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    if s.is_empty() { "mod".into() } else { s }
}

/// Descomprime un mod: los archivos de registro (`<registro>.json`) y `script.lua`, en la raíz del zip o dentro de una
/// carpeta (que da el nombre; si no, `stem`, el nombre del zip). Lo demás se ignora; rutas con `..` se descartan.
pub fn import_zip(dir: &Path, data: &[u8], stem: &str) -> Result<String, String> {
    let mut z = zip::ZipArchive::new(Cursor::new(data)).map_err(|e| e.to_string())?;
    let (mut files, mut total) = (Vec::new(), 0u64);
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| e.to_string())?;
        let Some(p) = f.enclosed_name().filter(|_| f.is_file()) else { continue };
        let parts: Vec<String> = p.iter().map(|c| c.to_string_lossy().into_owned()).collect();
        let (top, name) = match parts.as_slice() {
            [n] => (None, n.clone()),
            [t, n] => (Some(t.clone()), n.clone()),
            _ => continue,
        };
        if !game::registry::is_mod_file(&name) {
            continue;
        }
        let mut buf = Vec::new();
        f.by_ref().take(MAX_BYTES + 1 - total).read_to_end(&mut buf).map_err(|e| e.to_string())?;
        total += buf.len() as u64;
        if total > MAX_BYTES {
            return Err(game::tr!("el mod es demasiado grande", "the mod is too big").into());
        }
        files.push((top, name, buf));
    }
    if files.is_empty() {
        return Err(game::tr!("el zip no tiene archivos de mod (.json de registros o script.lua)", "the zip has no mod files (registry .json or script.lua)").into());
    }
    let top = files[0].0.clone();
    let name = clean(match &top {
        Some(t) if files.iter().all(|f| f.0.as_ref() == Some(t)) => t,
        _ => stem,
    });
    let out = dir.join("mods").join(&name);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for (_, file, buf) in files {
        std::fs::write(out.join(file), buf).map_err(|e| e.to_string())?;
    }
    Ok(name)
}

/// Escritorio: el diálogo de archivos del sistema; `None` si se cancela.
#[cfg(not(target_os = "android"))]
pub fn pick(dir: &Path) -> Option<Result<String, String>> {
    let path = rfd::FileDialog::new().add_filter("zip", &["zip"]).pick_file()?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    Some(std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| import_zip(dir, &b, &stem)))
}

/// Android: abre el selector de documentos (`MainActivity.pickZip`); el resultado llega después por `poll`.
#[cfg(target_os = "android")]
pub fn pick(_dir: &Path) -> Option<Result<String, String>> {
    let r = (|| -> Result<(), String> {
        let app = crate::ANDROID_APP.get().ok_or("sin actividad")?;
        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let act = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr().cast()) };
        env.call_method(&act, "pickZip", "()V", &[]).map_err(|e| e.to_string())?;
        Ok(())
    })();
    r.err().map(Err)
}

/// Android: el zip que dejó `MainActivity` (y su nombre en `mod_import.name`), si ya llegó.
#[cfg(target_os = "android")]
pub fn poll(dir: &Path) -> Option<Result<String, String>> {
    let zip = dir.join("mod_import.zip");
    let data = std::fs::read(&zip).ok()?;
    let name = std::fs::read_to_string(dir.join("mod_import.name")).unwrap_or_default();
    let _ = std::fs::remove_file(&zip);
    let _ = std::fs::remove_file(dir.join("mod_import.name"));
    let stem = name.trim().trim_end_matches(".zip").to_string();
    Some(import_zip(dir, &data, &stem))
}
