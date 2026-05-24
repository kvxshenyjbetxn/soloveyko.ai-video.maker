use std::path::PathBuf;

#[cfg(target_os = "windows")]
const FFMPEG_NAME: &str = "ffmpeg.exe";
#[cfg(not(target_os = "windows"))]
const FFMPEG_NAME: &str = "ffmpeg";

#[cfg(target_os = "windows")]
const FFPROBE_NAME: &str = "ffprobe.exe";
#[cfg(not(target_os = "windows"))]
const FFPROBE_NAME: &str = "ffprobe";

#[cfg(target_os = "windows")]
const FFMPEG_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffmpeg.exe";
#[cfg(not(target_os = "windows"))]
const FFMPEG_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffmpeg";

#[cfg(target_os = "windows")]
const FFPROBE_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffprobe.exe";
#[cfg(not(target_os = "windows"))]
const FFPROBE_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffprobe";

/// Папка для бандлованих бінарників: <UserConfigDir>/Soloveyko.AI-Video.Maker/bin/
pub fn bin_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Soloveyko.AI-Video.Maker")
        .join("bin")
}

/// Шлях до ffmpeg: спочатку ~/bin/, потім системний PATH.
pub fn ffmpeg_path() -> String {
    let local = bin_dir().join(FFMPEG_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    FFMPEG_NAME.to_string()
}

/// Шлях до ffprobe: спочатку ~/bin/, потім системний PATH.
#[allow(dead_code)]
pub fn ffprobe_path() -> String {
    let local = bin_dir().join(FFPROBE_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    FFPROBE_NAME.to_string()
}

/// Завантажує ffmpeg і ffprobe у ~/bin/.
/// `on_progress` викликається з рядком прогресу, наприклад `"ffmpeg (7.2 / 76.0 MB)"`.
pub fn download_all(mut on_progress: impl FnMut(String)) -> Result<(), String> {
    let dir = bin_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

    let ffmpeg_dest = dir.join(FFMPEG_NAME);
    if !ffmpeg_dest.exists() {
        download_file(FFMPEG_URL, &ffmpeg_dest, "ffmpeg", &mut on_progress)?;
    }

    let ffprobe_dest = dir.join(FFPROBE_NAME);
    if !ffprobe_dest.exists() {
        download_file(FFPROBE_URL, &ffprobe_dest, "ffprobe", &mut on_progress)?;
    }

    Ok(())
}

fn download_file(
    url: &str,
    dest: &PathBuf,
    label: &str,
    on_progress: &mut impl FnMut(String),
) -> Result<(), String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP помилка: {}", e))?;

    let total_bytes = response
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok());

    let mut reader = response.into_reader();
    let mut buf = Vec::new();
    let mut chunk = [0u8; 65536]; // 64 KB чанки

    loop {
        let n = std::io::Read::read(&mut reader, &mut chunk)
            .map_err(|e| format!("Помилка читання: {}", e))?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);

        let downloaded_mb = buf.len() as f64 / 1_048_576.0;
        let progress_str = if let Some(total) = total_bytes {
            let total_mb = total as f64 / 1_048_576.0;
            let pct = (buf.len() as f64 / total as f64 * 100.0) as u32;
            format!("{} ({:.1} / {:.1} MB, {}%)", label, downloaded_mb, total_mb, pct)
        } else {
            format!("{} ({:.1} MB)", label, downloaded_mb)
        };
        on_progress(progress_str);
    }

    std::fs::write(dest, &buf)
        .map_err(|e| format!("Помилка запису: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod помилка: {}", e))?;
    }

    Ok(())
}
