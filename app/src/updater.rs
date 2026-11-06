//! Actualizador (D4, feature `updater`): sólo en las builds de GitHub o las compiladas con `--features app/updater`;
//! nunca en Play Store ni Steam (actualizan sus tiendas). Consulta la última Release estable del repo y baja el
//! archivo de esta plataforma: en escritorio reemplaza el ejecutable (la app reinicia), en Android
//! `MainActivity.installApk` lo instala (el sistema pide confirmar). Todo en hilos: la app recibe un `Msg`.
//! Sin la feature, `check` e `install` no hacen nada (`ENABLED` = false).
use std::path::PathBuf;
#[cfg(not(feature = "updater"))]
use std::{path::Path, sync::mpsc::Receiver};

/// Lo que contesta el hilo del actualizador.
#[cfg_attr(not(feature = "updater"), allow(dead_code))]
pub enum Msg {
    /// Hay una versión más nueva (la última Release estable).
    Found(String),
    /// Bajada y lista: el ejecutable nuevo (escritorio) o el APK (Android).
    Ready(PathBuf),
    Failed(String),
}

pub const ENABLED: bool = cfg!(feature = "updater");

/// La beta de esta build (`PDRI_BUILD`, "beta-a"...; la pone la CI en `dev`), si es una.
pub fn build() -> Option<&'static str> {
    option_env!("PDRI_BUILD").filter(|b| !b.is_empty())
}

/// La versión para mostrar: "0.9.26" o "0.9.26 beta-a".
pub fn version_label() -> String {
    build().map_or_else(|| game::VERSION.to_string(), |b| format!("{} {b}", game::VERSION))
}

#[cfg(not(feature = "updater"))]
pub fn check() -> Option<Receiver<Msg>> {
    None
}

#[cfg(not(feature = "updater"))]
pub fn install(_version: &str, _dir: &Path) -> Option<Receiver<Msg>> {
    None
}

#[cfg(not(feature = "updater"))]
pub fn cleanup(_dir: &Path) {}

#[cfg(feature = "updater")]
pub use imp::{check, cleanup, install};

#[cfg(feature = "updater")]
mod imp {
    use super::Msg;
    use serde_json::Value;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{channel, Receiver};
    use std::time::Duration;

    /// El repo de las Releases: el de la CI que compiló (`PDRI_REPO`, así un fork se actualiza del suyo) o el del juego.
    const REPO: &str = match option_env!("PDRI_REPO") {
        Some(r) => r,
        None => "NotsumeChan/PixelDungeon-RollInitiative",
    };
    /// El archivo de esta plataforma en la Release (los nombres del job `release` de `.github/workflows/build.yml`).
    pub(super) const ASSET: &str = if cfg!(target_os = "android") {
        "pdri-android.apk"
    } else if cfg!(windows) {
        "pdri-windows-x86_64.exe"
    } else {
        "pdri-linux-x86_64"
    };
    const MAX_BYTES: u64 = 256 << 20;

    /// Canal beta (testers): compilado con `PDRI_CHANNEL=beta`, también ve las pre-releases de `dev`
    /// (`v<versión>-beta.<n>`). Las builds estables nunca.
    fn beta() -> bool {
        option_env!("PDRI_CHANNEL") == Some("beta")
    }

    /// La versión de esta build para comparar: la del juego y, en una beta, su letra ("0.9.26-beta-a").
    pub(super) fn current() -> String {
        match super::build() {
            Some(b) => format!("{}-{b}", game::VERSION),
            None if beta() => format!("{}-beta", game::VERSION), // beta compilada a mano: antes que cualquier beta-a
            None => game::VERSION.to_string(),
        }
    }

    // ponytail: confía en HTTPS y en el repo (sin firma propia del binario); en Android el sistema además exige la
    // misma clave del APK instalado
    fn get(url: &str) -> Result<ureq::Response, String> {
        ureq::get(url).set("User-Agent", "pdri-updater").timeout(Duration::from_secs(120)).call().map_err(|e| e.to_string())
    }

    /// La Release `v<version>`, o la última estable (`latest` no incluye pre-releases).
    fn release(version: Option<&str>) -> Result<Value, String> {
        let url = match version {
            Some(v) => format!("https://api.github.com/repos/{REPO}/releases/tags/v{v}"),
            None => format!("https://api.github.com/repos/{REPO}/releases/latest"),
        };
        let body = get(&url)?.into_string().map_err(|e| e.to_string())?;
        serde_json::from_str(&body).map_err(|e| e.to_string())
    }

    /// La versión de una Release y la URL del archivo de esta plataforma.
    pub(super) fn pick(r: &Value) -> Option<(String, String)> {
        let v = r["tag_name"].as_str()?.trim_start_matches('v').to_string();
        let a = r["assets"].as_array()?.iter().find(|a| a["name"] == ASSET)?;
        Some((v, a["browser_download_url"].as_str()?.to_string()))
    }

    /// La versión más nueva que se ofrece: la última estable o, en el canal beta, la más nueva de las últimas Releases
    /// (pre-releases incluidas), con el archivo de esta plataforma.
    fn newest() -> Result<Option<String>, String> {
        if !beta() {
            return Ok(pick(&release(None)?).map(|(v, _)| v));
        }
        let body = get(&format!("https://api.github.com/repos/{REPO}/releases?per_page=20"))?.into_string().map_err(|e| e.to_string())?;
        let list: Vec<Value> = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        Ok(list.iter().filter_map(pick).map(|(v, _)| v).reduce(|a, b| if crate::net::newer(&b, &a) { b } else { a }))
    }

    /// Al abrir: si hay una versión más nueva que esta build, `Found`. Sin red o sin Releases, no contesta nada.
    pub fn check() -> Option<Receiver<Msg>> {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            if let Ok(Some(v)) = newest()
                && crate::net::newer(&v, &current())
            {
                let _ = tx.send(Msg::Found(v));
            }
        });
        Some(rx)
    }

    /// Baja la versión `version` (la ofrecida, o la del host al unirse) y la deja lista (`Ready`) o `Failed`.
    /// ponytail: la del host es su versión sin número de beta; si el host es una beta, esa Release estable puede no
    /// existir todavía (Failed)
    pub fn install(version: &str, dir: &Path) -> Option<Receiver<Msg>> {
        let (tx, rx) = channel();
        let (version, dir) = (version.to_string(), dir.to_path_buf());
        std::thread::spawn(move || {
            let _ = tx.send(download(&version, &dir).map_or_else(Msg::Failed, Msg::Ready));
        });
        Some(rx)
    }

    fn download(version: &str, dir: &Path) -> Result<PathBuf, String> {
        let r = release(Some(version))?;
        let (_, url) = pick(&r).ok_or_else(|| format!("la Release v{version} no tiene {ASSET}"))?;
        let mut bytes = Vec::new();
        get(&url)?.into_reader().take(MAX_BYTES).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        #[cfg(target_os = "android")]
        {
            let apk = dir.join("update.apk");
            std::fs::write(&apk, &bytes).map_err(|e| e.to_string())?;
            Ok(apk)
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = dir;
            replace_exe(&bytes).map_err(|e| e.to_string())
        }
    }

    /// Escritorio: el ejecutable nuevo en lugar del que corre. En Windows el que corre no se puede pisar pero sí
    /// renombrar (`.old`, se borra al próximo arranque); en Linux se reemplaza directo.
    #[cfg(not(target_os = "android"))]
    fn replace_exe(bytes: &[u8]) -> std::io::Result<PathBuf> {
        let exe = std::env::current_exe()?;
        let new = exe.with_extension("new");
        std::fs::write(&new, bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755))?;
        }
        #[cfg(windows)]
        {
            let old = exe.with_extension("old");
            let _ = std::fs::remove_file(&old);
            std::fs::rename(&exe, &old)?;
        }
        std::fs::rename(&new, &exe)?;
        Ok(exe)
    }

    /// Al arrancar: lo que quedó de una actualización anterior.
    pub fn cleanup(dir: &Path) {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::fs::remove_file(exe.with_extension("old"));
        }
        let _ = std::fs::remove_file(dir.join("update.apk"));
    }
}

#[cfg(all(test, feature = "updater"))]
mod tests {
    use super::imp::{pick, ASSET};
    use serde_json::json;

    #[test]
    fn archivo_de_la_release() {
        let r = json!({"tag_name": "v0.9.30", "assets": [
            {"name": "otro", "browser_download_url": "https://x"},
            {"name": ASSET, "browser_download_url": "https://u"}]});
        assert_eq!(pick(&r), Some(("0.9.30".to_string(), "https://u".to_string())));
        assert_eq!(pick(&json!({"tag_name": "v1.0.0", "assets": []})), None);
        assert_eq!(pick(&json!({"message": "Not Found"})), None);
    }
}
