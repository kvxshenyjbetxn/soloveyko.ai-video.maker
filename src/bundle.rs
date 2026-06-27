use std::path::PathBuf;

#[cfg(target_os = "windows")]
const FFMPEG_NAME: &str = "ffmpeg.exe";
#[cfg(not(target_os = "windows"))]
const FFMPEG_NAME: &str = "ffmpeg";

#[cfg(target_os = "windows")]
const FFPROBE_NAME: &str = "ffprobe.exe";
#[cfg(not(target_os = "windows"))]
const FFPROBE_NAME: &str = "ffprobe";

/// На Windows whisper розпаковується у підпапку whisper.win/ (як whisperx та whisper-amd).
#[cfg(target_os = "windows")]
const WHISPER_WIN_DIR: &str = "whisper.win";
#[cfg(target_os = "windows")]
const WHISPER_WIN_CLI: &str = "whisper-cli.exe";
/// Легасі-назви для тих у кого ще є старіші бінарники.
#[cfg(target_os = "windows")]
const WHISPER_NAME_LEGACY: &str = "main.exe";
#[cfg(target_os = "windows")]
const WHISPER_NAME_LEGACY2: &str = "whisper-whisper.exe";
#[cfg(target_os = "windows")]
const WHISPER_NAME_LEGACY3: &str = "whisper.exe";
/// Не Windows — один файл у bin_dir.
#[cfg(not(target_os = "windows"))]
const WHISPER_NAME: &str = "whisper";

#[cfg(target_os = "windows")]
const FFMPEG_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffmpeg.exe";
#[cfg(not(target_os = "windows"))]
const FFMPEG_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffmpeg";

#[cfg(target_os = "windows")]
const FFPROBE_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffprobe.exe";
#[cfg(not(target_os = "windows"))]
const FFPROBE_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/ffprobe";

/// На Windows — zip-архів з папкою всередині, в якій лежить main.exe.
#[cfg(target_os = "windows")]
const WHISPER_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisper.win.zip";
#[cfg(not(target_os = "windows"))]
const WHISPER_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisper";

#[cfg(target_os = "windows")]
const WHISPERX_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisperx_win.zip";
#[cfg(target_os = "macos")]
const WHISPERX_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisperx_mac.zip";

#[cfg(target_os = "windows")]
const WHISPERX_DIR_NAME: &str = "whisperx_win";
#[cfg(not(target_os = "windows"))]
const WHISPERX_DIR_NAME: &str = "whisperx_mac";

#[cfg(target_os = "windows")]
const WHISPERX_CLI_NAME: &str = "whisperx_cli.exe";
#[cfg(not(target_os = "windows"))]
const WHISPERX_CLI_NAME: &str = "whisperx_cli";

/// Тільки Windows — AMD GPU-оптимізований whisper (zip → папка whisper-amd/).
#[cfg(target_os = "windows")]
const WHISPER_AMD_URL: &str =
    "https://github.com/kvxshenyjbetxn/repo.releases/releases/download/all.bundle/whisper-amd.zip";

const WHISPER_AMD_DIR_NAME: &str = "whisper-amd";

#[cfg(target_os = "windows")]
const WHISPER_AMD_CLI_NAME: &str = "main.exe";
#[cfg(not(target_os = "windows"))]
const WHISPER_AMD_CLI_NAME: &str = "main";

/// Папка для бандлованих бінарників: <UserConfigDir>/Soloveyko.AI-Video.Maker/bin/
pub fn bin_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Soloveyko.AI-Video.Maker")
        .join("bin")
}

/// Перевіряє, чи є бінарник у системному PATH (без запуску процесу).
/// Шлях до бандлованого ffmpeg.
pub fn ffmpeg_path() -> String {
    let local = bin_dir().join(FFMPEG_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    FFMPEG_NAME.to_string()
}

/// Шлях до бандлованого ffprobe.
#[allow(dead_code)]
pub fn ffprobe_path() -> String {
    let local = bin_dir().join(FFPROBE_NAME);
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    FFPROBE_NAME.to_string()
}

/// Шлях до whisper.
/// Windows: bin/whisper.win/whisper-cli.exe → bin/whisper.win/main.exe → bin/whisper-whisper.exe → bin/whisper.exe → PATH.
/// Non-Windows: bin/whisper → PATH.
pub fn whisper_path() -> String {
    let dir = bin_dir();
    #[cfg(target_os = "windows")]
    {
        let folder_bin = dir.join(WHISPER_WIN_DIR).join(WHISPER_WIN_CLI);
        if folder_bin.exists() {
            return folder_bin.to_string_lossy().into_owned();
        }
        let legacy1 = dir.join(WHISPER_WIN_DIR).join(WHISPER_NAME_LEGACY);
        if legacy1.exists() {
            return legacy1.to_string_lossy().into_owned();
        }
        let legacy2 = dir.join(WHISPER_NAME_LEGACY2);
        if legacy2.exists() {
            return legacy2.to_string_lossy().into_owned();
        }
        let legacy3 = dir.join(WHISPER_NAME_LEGACY3);
        if legacy3.exists() {
            return legacy3.to_string_lossy().into_owned();
        }
        return WHISPER_WIN_CLI.to_string();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let local = dir.join(WHISPER_NAME);
        if local.exists() {
            return local.to_string_lossy().into_owned();
        }
        WHISPER_NAME.to_string()
    }
}

/// Перевіряє, чи є whisper у локальному bin_dir.
pub fn whisper_local_exists() -> bool {
    let dir = bin_dir();
    #[cfg(target_os = "windows")]
    {
        dir.join(WHISPER_WIN_DIR).join(WHISPER_WIN_CLI).exists()
            || dir.join(WHISPER_WIN_DIR).join(WHISPER_NAME_LEGACY).exists()
            || dir.join(WHISPER_NAME_LEGACY2).exists()
            || dir.join(WHISPER_NAME_LEGACY3).exists()
    }
    #[cfg(not(target_os = "windows"))]
    {
        dir.join(WHISPER_NAME).exists()
    }
}

/// Перевіряє, чи є папка whisperx у локальному bin_dir.
pub fn whisperx_local_exists() -> bool {
    bin_dir().join(WHISPERX_DIR_NAME).is_dir()
}

/// Повний шлях до виконуваного файлу whisperx_cli (платформозалежно).
pub fn whisperx_cmd_path() -> PathBuf {
    bin_dir().join(WHISPERX_DIR_NAME).join(WHISPERX_CLI_NAME)
}

/// Перевіряє, чи є папка whisper-amd у локальному bin_dir.
pub fn whisper_amd_local_exists() -> bool {
    bin_dir().join(WHISPER_AMD_DIR_NAME).is_dir()
}

/// Повний шлях до виконуваного файлу whisper-amd (платформозалежно).
pub fn whisper_amd_cmd_path() -> PathBuf {
    bin_dir()
        .join(WHISPER_AMD_DIR_NAME)
        .join(WHISPER_AMD_CLI_NAME)
}

/// Завантажує whisper-amd у bin_dir (розпаковує папку з zip).
/// Тільки Windows.
#[cfg(target_os = "windows")]
pub fn download_whisper_amd(on_progress: impl FnMut(String)) -> Result<(), String> {
    let mut on_progress = on_progress;
    let dir = bin_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

    if whisper_amd_local_exists() {
        return Ok(());
    }

    let bytes = download_to_bytes(WHISPER_AMD_URL, "whisper-amd", &mut on_progress)?;
    extract_folder_from_zip(&bytes, &dir)?;
    Ok(())
}

/// Завантажує whisperx у bin_dir (розпаковує папку з zip).
/// Підтримується на macOS та Windows.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn download_whisperx(on_progress: impl FnMut(String)) -> Result<(), String> {
    let mut on_progress = on_progress;
    let dir = bin_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

    if whisperx_local_exists() {
        return Ok(());
    }

    let bytes = download_to_bytes(WHISPERX_URL, "whisperx", &mut on_progress)?;
    extract_folder_from_zip(&bytes, &dir)?;
    Ok(())
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
        "tiny" => 75.0,
        "base" => 148.0,
        "small" => 488.0,
        "medium" => 1500.0,
        "large-v3" => 3100.0,
        "large-v3-turbo" => 1600.0,
        _ => 0.0,
    }
}

/// Завантажує ggml-модель whisper.cpp із HuggingFace у папку bin/models.
pub fn download_whisper_model(
    model: &str,
    mut on_progress: impl FnMut(String),
) -> Result<(), String> {
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
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

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
/// Windows: розпаковує zip → папка whisper.win/ з whisper-cli.exe всередині (як whisperx/whisper-amd).
/// Non-Windows: завантажує одиночний бінарник.
pub fn download_whisper(mut on_progress: impl FnMut(String)) -> Result<(), String> {
    let dir = bin_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не вдалося створити папку bin: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let target = dir.join(WHISPER_WIN_DIR).join(WHISPER_WIN_CLI);
        if target.exists() {
            return Ok(());
        }
        let bytes = download_to_bytes(WHISPER_URL, "whisper", &mut on_progress)?;
        extract_folder_from_zip(&bytes, &dir)?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let dest = dir.join(WHISPER_NAME);
        if dest.exists() {
            return Ok(());
        }
        download_file(WHISPER_URL, &dest, "whisper", &mut on_progress)?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
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
            format!(
                "{} ({:.1} / {:.1} MB, {}%)",
                label, downloaded_mb, total_mb, pct
            )
        } else {
            format!("{} ({:.1} MB)", label, downloaded_mb)
        };
        on_progress(progress_str);
    }

    Ok(buf)
}

/// Розпаковує всю папку з zip-архіву у вказану папку призначення.
/// Бере першу папку верхнього рівня в архіві та рекурсивно витягує її вміст.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn extract_folder_from_zip(bytes: &[u8], dest_parent: &PathBuf) -> Result<(), String> {
    use std::io::Read;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Помилка відкриття zip: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
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
            entry
                .read_to_end(&mut contents)
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
                        let _ = std::fs::set_permissions(
                            &out_path,
                            std::fs::Permissions::from_mode(0o755),
                        );
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

    std::fs::write(dest, &buf).map_err(|e| format!("Помилка запису: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod помилка: {}", e))?;
    }

    Ok(())
}

/// Приховує консольне вікно дочірнього процесу на Windows.
/// На macOS та Linux — no-op.
pub fn set_no_window(cmd: &mut std::process::Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    #[cfg(not(target_os = "windows"))]
    let _ = cmd;
}

/// На Windows шукає npm .cmd файл у типових директоріях (APPDATA\npm\).
/// Повертає повний шлях до .cmd файлу, якщо знайдено.
#[cfg(target_os = "windows")]
pub fn find_npm_cmd_windows(name: &str) -> Option<String> {
    let cmd_name = format!("{}.cmd", name);
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = format!("{}\\npm\\{}", appdata, cmd_name);
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    if let Ok(localdata) = std::env::var("LOCALAPPDATA") {
        let p = format!("{}\\npm\\{}", localdata, cmd_name);
        if std::path::Path::new(&p).exists() {
            return Some(p);
        }
    }
    None
}

/// На Windows шукає node.js скрипт npm-пакету та повертає (node_exe, script_path).
/// Читає pi.cmd щоб знайти реальний JS файл — надійніше ніж запуск через cmd /C.
#[cfg(target_os = "windows")]
pub fn find_npm_node_script_windows(cmd_name: &str) -> Option<(String, String)> {
    let cmd_path = find_npm_cmd_windows(cmd_name)?;

    // Читаємо .cmd файл і витягуємо шлях до JS скрипту зі рядка виду:
    // "%_prog%"  "%dp0%\node_modules\pkg\dist\cli.js" %*
    let content = std::fs::read_to_string(&cmd_path).ok()?;
    let script_path = content
        .lines()
        .find(|l| l.contains("node_modules") && l.contains(".js"))?
        // витягуємо шлях між другою парою лапок
        .split('"')
        .find(|s| s.contains("node_modules") && s.ends_with(".js"))?
        // %dp0% розгортаємо у реальну директорію npm
        .replace(
            "%dp0%",
            &std::path::Path::new(&cmd_path).parent()?.to_string_lossy(),
        );

    if !std::path::Path::new(&script_path).exists() {
        return None;
    }

    // Шукаємо node.exe — спочатку поряд з npm, потім Program Files
    let npm_dir = std::path::Path::new(&cmd_path).parent()?;
    let node_candidates = [
        npm_dir.join("node.exe").to_string_lossy().into_owned(),
        "C:\\Program Files\\nodejs\\node.exe".to_string(),
        "C:\\Program Files (x86)\\nodejs\\node.exe".to_string(),
    ];
    let node_exe = node_candidates
        .iter()
        .find(|p| std::path::Path::new(p.as_str()).exists())
        .cloned()
        .unwrap_or_else(|| "node".to_string()); // fallback: шукає у PATH

    Some((node_exe, script_path))
}

/// Шукає бінарник у типових місцях macOS (PATH з терміналу недоступний у .app).
#[cfg(target_os = "macos")]
pub fn find_binary_macos(name: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("/usr/local/bin/{}", name),
        format!("/opt/homebrew/bin/{}", name),
        format!("{}/.local/bin/{}", home, name),
        format!("{}/.npm-global/bin/{}", home, name),
        format!("/usr/bin/{}", name),
    ];
    candidates
        .iter()
        .find(|p| std::path::Path::new(p.as_str()).exists())
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

/// Розширений PATH для macOS .app — додає типові директорії де живуть node, python тощо.
#[cfg(target_os = "macos")]
pub fn macos_extended_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let system_path = std::env::var("PATH").unwrap_or_default();
    let extra = [
        "/usr/local/bin",
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        &format!("{}/.local/bin", home),
        &format!("{}/.npm-global/bin", home),
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ];
    let mut parts: Vec<&str> = extra.iter().map(|s| *s).collect();
    if !system_path.is_empty() {
        parts.push(&system_path);
    }
    parts.join(":")
}

/// Створює std::process::Command для CLI-інструменту (claude, gemini тощо)
/// з урахуванням специфіки Windows (cmd /C) та macOS (розширений PATH та пошук бінарника).
pub fn new_cli_command(name: &str) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("cmd");
        if name == "codex" {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let codex_candidate = format!(
                    "{}\\Programs\\OpenAI\\Codex\\bin\\codex.exe",
                    local_app_data
                );
                if std::path::Path::new(&codex_candidate).exists() {
                    cmd.args(&["/C", &codex_candidate]);
                    set_no_window(&mut cmd);
                    return cmd;
                }
            }
        }
        cmd.args(&["/C", name]);
        set_no_window(&mut cmd);
        cmd
    }

    #[cfg(target_os = "macos")]
    {
        let candidate = find_binary_macos(name);
        let mut cmd = std::process::Command::new(&candidate);
        cmd.env("PATH", macos_extended_path());
        cmd
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new(name)
    }
}

/// Створює std::process::Command для CLI-інструменту (claude, gemini тощо)
/// БЕЗ використання cmd /C на Windows, щоб уникнути проблем із передачею довгих аргументів,
/// але з пошуком бінарника та встановленням розширеного PATH на macOS.
pub fn new_direct_cli_command(name: &str) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new(name);
        set_no_window(&mut cmd);
        cmd
    }

    #[cfg(target_os = "macos")]
    {
        let candidate = find_binary_macos(name);
        let mut cmd = std::process::Command::new(&candidate);
        cmd.env("PATH", macos_extended_path());
        cmd
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new(name)
    }
}
