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
const WHISPER_NAME: &str = "whisper.exe";
#[cfg(not(target_os = "windows"))]
const WHISPER_NAME: &str = "whisper";

#[cfg(target_os = "windows")]
const FFMPEG_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffmpeg.exe";
#[cfg(not(target_os = "windows"))]
const FFMPEG_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffmpeg";

#[cfg(target_os = "windows")]
const FFPROBE_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffprobe.exe";
#[cfg(not(target_os = "windows"))]
const FFPROBE_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffprobe";

/// На Windows — zip-архів з папкою всередині, в якій лежить main.exe.
#[cfg(target_os = "windows")]
const WHISPER_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisper.win.zip";
#[cfg(not(target_os = "windows"))]
const WHISPER_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisper";

/// На macOS — zip-архів з папкою всередині. Windows поки не підтримується.
#[cfg(target_os = "macos")]
const WHISPERX_URL: &str = "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisperx_mac.zip";

const WHISPERX_DIR_NAME: &str = "whisperx_mac";

/// Папка для бандлованих бінарників: <UserConfigDir>/Soloveyko.AI-Video.Maker/bin/
pub fn bin_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Soloveyko.AI-Video.Maker")
        .join("bin")
}

/// Шлях до ffmpeg: спочатку bin_dir, потім системний PATH.
pub fn ffmpeg_path() -> String {
    let local = bin_dir().join(FFMPEG_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    FFMPEG_NAME.to_string()
}

/// Шлях до ffprobe: спочатку bin_dir, потім системний PATH.
#[allow(dead_code)]
pub fn ffprobe_path() -> String {
    let local = bin_dir().join(FFPROBE_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    FFPROBE_NAME.to_string()
}

/// Шлях до whisper: спочатку bin_dir, потім системний PATH.
pub fn whisper_path() -> String {
    let local = bin_dir().join(WHISPER_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    WHISPER_NAME.to_string()
}

/// Перевіряє, чи є whisper у локальному bin_dir.
pub fn whisper_local_exists() -> bool {
    bin_dir().join(WHISPER_NAME).exists()
}

/// Перевіряє, чи є папка whisperx у локальному bin_dir.
pub fn whisperx_local_exists() -> bool {
    bin_dir().join(WHISPERX_DIR_NAME).is_dir()
}

/// Завантажує whisperx у bin_dir (розпаковує папку з zip).
/// Підтримується лише macOS. На Windows повертає помилку.
pub fn download_whisperx(mut on_progress: impl FnMut(String)) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = on_progress;
        return Err("WhisperX автоматичне завантаження підтримується лише на macOS".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let dir = bin_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

        if whisperx_local_exists() {
            return Ok(());
        }

        let bytes = download_to_bytes(WHISPERX_URL, "whisperx", &mut on_progress)?;
        extract_folder_from_zip(&bytes, &dir)?;
        Ok(())
    }
}

/// Папка для ggml-моделей whisper.cpp: <bin_dir>/models/
pub fn models_dir() -> PathBuf {
    bin_dir().join("models")
}

/// Повертає ім'я файлу ggml-моделі за її назвою (наприклад, "base" → "ggml-base.bin").
pub fn whisper_model_filename(model: &str) -> String {
    format!("ggml-{}.bin", model)
}

/// Повертає повний шлях до файлу ggml-моделі.
pub fn whisper_model_path(model: &str) -> PathBuf {
    models_dir().join(whisper_model_filename(model))
}

/// Перевіряє, чи завантажена дана модель у локальну папку models.
pub fn whisper_model_exists(model: &str) -> bool {
    whisper_model_path(model).exists()
}

/// Приблизний розмір ggml-моделі у MB (для відображення перед завантаженням).
pub fn whisper_model_size_mb(model: &str) -> f64 {
    match model {
        "tiny"           => 75.0,
        "base"           => 148.0,
        "small"          => 488.0,
        "medium"         => 1500.0,
        "large-v3"       => 3100.0,
        "large-v3-turbo" => 1600.0,
        _                => 0.0,
    }
}

/// Завантажує ggml-модель whisper.cpp із HuggingFace у папку bin/models.
pub fn download_whisper_model(model: &str, mut on_progress: impl FnMut(String)) -> Result<(), String> {
    let dir = models_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Не вдалося створити папку models: {}", e))?;

    let filename = whisper_model_filename(model);
    let dest = dir.join(&filename);

    if dest.exists() {
        return Ok(());
    }

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    );
    download_file(&url, &dest, model, &mut on_progress)?;
    Ok(())
}

/// Завантажує ffmpeg і ffprobe у bin_dir (пропускає вже наявні).
/// `on_progress` викликається з рядком прогресу, наприклад `"ffmpeg (7.2 / 76.0 MB, 9%)"`.
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

/// Завантажує whisper у bin_dir (пропускає якщо вже є).
/// На Windows — розпаковує zip і витягує main.exe.
pub fn download_whisper(mut on_progress: impl FnMut(String)) -> Result<(), String> {
    let dir = bin_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

    let dest = dir.join(WHISPER_NAME);
    if dest.exists() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let bytes = download_to_bytes(WHISPER_URL, "whisper", &mut on_progress)?;
        extract_main_exe_from_zip(&bytes, &dest)?;
    }

    #[cfg(not(target_os = "windows"))]
    download_file(WHISPER_URL, &dest, "whisper", &mut on_progress)?;

    Ok(())
}

/// Розпаковує main.exe з zip-архіву (шукає в будь-якій підпапці).
#[cfg(target_os = "windows")]
fn extract_main_exe_from_zip(bytes: &[u8], dest: &PathBuf) -> Result<(), String> {
    use std::io::Read;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Помилка відкриття zip: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Помилка читання запису zip: {}", e))?;

        let name = entry.name().to_string();
        if name.ends_with("main.exe") {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)
                .map_err(|e| format!("Помилка розпакування main.exe: {}", e))?;
            std::fs::write(dest, &contents)
                .map_err(|e| format!("Помилка запису whisper.exe: {}", e))?;
            return Ok(());
        }
    }

    Err("main.exe не знайдено в архіві".to_string())
}

/// Завантажує URL у пам'ять з відображенням прогресу.
fn download_to_bytes(
    url: &str,
    label: &str,
    on_progress: &mut impl FnMut(String),
) -> Result<Vec<u8>, String> {
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

    Ok(buf)
}

/// Розпаковує всю папку з zip-архіву у вказану папку призначення.
/// Бере першу папку верхнього рівня в архіві та рекурсивно витягує її вміст.
fn extract_folder_from_zip(bytes: &[u8], dest_parent: &PathBuf) -> Result<(), String> {
    use std::io::Read;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Помилка відкриття zip: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Помилка читання запису zip: {}", e))?;

        let entry_name = entry.name().to_string();

        // Пропускаємо записи з небезпечними шляхами
        if entry_name.contains("..") {
            continue;
        }

        let out_path = dest_parent.join(&entry_name);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|e| format!("Помилка створення папки {}: {}", entry_name, e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Помилка створення папки: {}", e))?;
            }
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)
                .map_err(|e| format!("Помилка читання файлу {}: {}", entry_name, e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let unix_mode = entry.unix_mode().unwrap_or(0o644);

                // Unix mode 0o120000 = символьне посилання (symlink).
                // PyInstaller-бандли містять симлінки всередині _internal/ — їх треба
                // відновлювати як справжні симлінки, інакше dylib виглядатиме як текстовий файл.
                if unix_mode & 0o170000 == 0o120000 {
                    let target = std::str::from_utf8(&contents)
                        .map_err(|e| format!("Некоректна ціль симлінку {}: {}", entry_name, e))?
                        .trim_end_matches('\0');
                    // Якщо симлінк вже існує — видаляємо перед створенням
                    if out_path.exists() || out_path.symlink_metadata().is_ok() {
                        let _ = std::fs::remove_file(&out_path);
                    }
                    std::os::unix::fs::symlink(target, &out_path)
                        .map_err(|e| format!("Помилка створення симлінку {}: {}", entry_name, e))?;
                } else {
                    std::fs::write(&out_path, &contents)
                        .map_err(|e| format!("Помилка запису {}: {}", entry_name, e))?;
                    if unix_mode & 0o111 != 0 {
                        let _ = std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(0o755));
                    }
                }
            }

            #[cfg(not(unix))]
            {
                std::fs::write(&out_path, &contents)
                    .map_err(|e| format!("Помилка запису {}: {}", entry_name, e))?;
            }
        }
    }

    Ok(())
}

/// Завантажує файл за URL і зберігає на диск.
fn download_file(
    url: &str,
    dest: &PathBuf,
    label: &str,
    on_progress: &mut impl FnMut(String),
) -> Result<(), String> {
    let buf = download_to_bytes(url, label, on_progress)?;

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
