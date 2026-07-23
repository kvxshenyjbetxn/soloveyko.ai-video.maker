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

/// Лімітер важких ffmpeg-екстракцій превʼю.
/// Дозволяє повернути плавний playback, але без масового старту десятків ffmpeg,
/// який раніше морозив увесь ПК на великих проектах.
struct PreviewExtractionLimiter {
    active: Mutex<usize>,
    condvar: Condvar,
}

impl PreviewExtractionLimiter {
    fn get() -> &'static Self {
        static LIMITER: OnceLock<PreviewExtractionLimiter> = OnceLock::new();
        LIMITER.get_or_init(|| PreviewExtractionLimiter {
            active: Mutex::new(0),
            condvar: Condvar::new(),
        })
    }

    fn acquire(&self) -> PreviewExtractionPermit<'_> {
        let mut active = self.active.lock().unwrap();
        while *active >= 2 {
            active = self.condvar.wait(active).unwrap();
        }
        *active += 1;
        PreviewExtractionPermit { limiter: self }
    }

    fn release(&self) {
        let mut active = self.active.lock().unwrap();
        if *active > 0 {
            *active -= 1;
        }
        self.condvar.notify_one();
    }
}

pub struct PreviewExtractionPermit<'a> {
    limiter: &'a PreviewExtractionLimiter,
}

impl Drop for PreviewExtractionPermit<'_> {
    fn drop(&mut self) {
        self.limiter.release();
    }
}

pub fn acquire_preview_extraction() -> PreviewExtractionPermit<'static> {
    PreviewExtractionLimiter::get().acquire()
}

fn probe_media_info(path: &Path) -> (Option<f32>, bool) {
    let _permit = ProbeLimiter::get().acquire();
    let clean_path = clean_windows_path(path);
    let mut cmd = std::process::Command::new(crate::bundle::ffprobe_path());
    cmd.args([
        "-v",
        "error",
        "-show_entries",
        "format=duration:stream=codec_type,duration",
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

    let parse_duration = |value: &serde_json::Value| {
        value
            .as_str()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .or_else(|| value.as_f64().map(|duration| duration as f32))
            .filter(|duration| duration.is_finite() && *duration > 0.0)
    };
    let streams = json
        .get("streams")
        .and_then(|streams| streams.as_array())
        .map(Vec::as_slice)
        .unwrap_or_default();
    let video_duration = streams
        .iter()
        .filter(|stream| stream.get("codec_type").and_then(|value| value.as_str()) == Some("video"))
        .filter_map(|stream| stream.get("duration").and_then(parse_duration))
        .max_by(|a, b| a.total_cmp(b));
    let format_duration = json
        .get("format")
        .and_then(|format| format.get("duration"))
        .and_then(parse_duration);
    let stream_duration = streams
        .iter()
        .filter_map(|stream| stream.get("duration").and_then(parse_duration))
        .max_by(|a, b| a.total_cmp(b));
    // Для відеофайлів контейнер може бути довшим через аудіодоріжку.
    // Точка розвороту повинна збігатися з кінцем саме відеопотоку.
    let duration = video_duration.or(format_duration).or(stream_duration);

    let has_audio = streams
        .iter()
        .any(|stream| stream.get("codec_type").and_then(|value| value.as_str()) == Some("audio"));

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
