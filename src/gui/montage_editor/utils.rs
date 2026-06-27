use super::types::PreviewRenderSettings;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Instant;

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
    let modified_ms = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{}_{}", meta.len(), modified_ms)
}

/// Папка легкого кешу кадрів превʼю з версією якості/роздільності.
/// Версіонування не дає редактору випадково читати старі кадри після апдейту.
pub fn frame_cache_dir(
    cache_base: &Path,
    media_path: &Path,
    settings: PreviewRenderSettings,
) -> PathBuf {
    cache_base.join(".frame_cache").join(format!(
        "{}_{}_{}_{}",
        path_hash(media_path),
        file_cache_version(media_path),
        settings.quality.cache_tag(),
        settings.fps_tag(),
    ))
}

/// Папка чіткого still-кешу. Тут зберігаються тільки точкові кадри,
/// які користувач реально зупиняв на таймлінії.
pub fn sharp_frame_cache_dir(
    cache_base: &Path,
    media_path: &Path,
    settings: PreviewRenderSettings,
) -> PathBuf {
    cache_base.join(".frame_cache").join(format!(
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

/// Лімітер одночасних ffprobe-процесів, щоб відкриття редактора не створювало
/// десятки важких probe одночасно і не морозило систему.
struct ProbeLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
}

impl ProbeLimiter {
    fn get() -> &'static Self {
        static LIMITER: OnceLock<ProbeLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| ProbeLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
        })
    }

    fn acquire(&self) -> ProbePermit<'_> {
        let mut active = self.active.lock().unwrap();
        while *active >= 2 {
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        ProbePermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }
}

struct ProbePermit<'a> {
    limiter: &'a ProbeLimiter,
}

impl Drop for ProbePermit<'_> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

fn probe_media_info(path: &Path) -> (Option<f32>, bool) {
    let _permit = ProbeLimiter::get().acquire();
    let clean_path = clean_windows_path(path);
    let mut cmd = std::process::Command::new(crate::bundle::ffprobe_path());
    cmd.args([
        "-v",
        "error",
        "-show_entries",
        "format=duration:stream=codec_type",
        "-of",
        "json",
    ])
    .arg(&clean_path);
    crate::bundle::set_no_window(&mut cmd);

    let Ok(out) = cmd.output() else {
        return (None, false);
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return (None, false);
    };

    let duration = json
        .get("format")
        .and_then(|v| v.get("duration"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.trim().parse::<f32>().ok());

    let has_audio = json
        .get("streams")
        .and_then(|v| v.as_array())
        .map(|streams| {
            streams
                .iter()
                .any(|stream| stream.get("codec_type").and_then(|v| v.as_str()) == Some("audio"))
        })
        .unwrap_or(false);

    (duration, has_audio)
}

/// Отримує тривалість медіафайлу через ffprobe
pub fn probe_duration(path: &Path) -> Option<f32> {
    probe_media_info(path).0
}

/// Одним викликом отримує duration та ознаку audio, щоб не ганяти ffprobe двічі.
pub fn probe_duration_and_audio(path: &Path) -> (Option<f32>, bool) {
    probe_media_info(path)
}

pub fn uuid_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}-{:x}", t.as_nanos(), rand_u32())
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    Instant::now().hash(&mut h);
    h.finish() as u32
}
