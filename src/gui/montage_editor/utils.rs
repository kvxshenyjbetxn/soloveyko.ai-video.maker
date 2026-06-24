use std::path::{Path, PathBuf};
use std::time::Instant;
use super::types::PreviewRenderSettings;

/// Стабільний хеш шляху → частина імені папки кешу.
pub fn path_hash(path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    format!("{:x}", h.finish())
}

/// Версія файлу для кешу кадрів.
/// Коли медіа перегенеровано в той самий шлях, mtime/розмір змінюються,
/// тому редактор не читає старі кадри з попереднього кешу.
fn file_cache_version(path: &Path) -> String {
    let Ok(meta) = std::fs::metadata(path) else {
        return "missing".to_string();
    };
    let modified_ms = meta.modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{}_{}", meta.len(), modified_ms)
}

/// Папка легкого кешу кадрів превʼю з версією якості/роздільності.
/// Версіонування не дає редактору випадково читати старі кадри після апдейту.
pub fn frame_cache_dir(cache_base: &Path, media_path: &Path, settings: PreviewRenderSettings) -> PathBuf {
    cache_base
        .join(".frame_cache")
        .join(format!(
            "{}_{}_{}_{}",
            path_hash(media_path),
            file_cache_version(media_path),
            settings.quality.cache_tag(),
            settings.fps_tag(),
        ))
}

/// Папка чіткого still-кешу. Тут зберігаються тільки точкові кадри,
/// які користувач реально зупиняв на таймлінії.
pub fn sharp_frame_cache_dir(cache_base: &Path, media_path: &Path, settings: PreviewRenderSettings) -> PathBuf {
    cache_base
        .join(".frame_cache")
        .join(format!(
            "{}_{}_{}_{}",
            path_hash(media_path),
            file_cache_version(media_path),
            settings.quality.sharp_cache_tag(),
            settings.fps_tag(),
        ))
}

fn clean_windows_path(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy();
        if s.starts_with(r"\\?\") {
            return PathBuf::from(&s[4..]);
        }
    }
    path.to_path_buf()
}

/// Перевіряє чи є аудіодоріжка у відеофайлі (через ffprobe)
pub fn probe_has_audio(path: &Path) -> bool {
    let clean_path = clean_windows_path(path);
    let mut cmd = std::process::Command::new(crate::bundle::ffprobe_path());
    cmd.args([
        "-v", "error",
        "-select_streams", "a:0",
        "-show_entries", "stream=codec_name",
        "-of", "default=nw=1:nk=1",
    ])
    .arg(&clean_path);
    crate::bundle::set_no_window(&mut cmd);
    cmd.output().ok()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}

/// Отримує тривалість медіафайлу через ffprobe
pub fn probe_duration(path: &Path) -> Option<f32> {
    let clean_path = clean_windows_path(path);
    let mut ffprobe_cmd = std::process::Command::new(crate::bundle::ffprobe_path());
    ffprobe_cmd.args(["-v", "error", "-show_entries", "format=duration",
           "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(&clean_path);
    crate::bundle::set_no_window(&mut ffprobe_cmd);
    let out = ffprobe_cmd.output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<f32>().ok()
}

pub fn uuid_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}-{:x}", t.as_nanos(), rand_u32())
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    Instant::now().hash(&mut h);
    h.finish() as u32
}
