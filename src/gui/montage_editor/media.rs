use super::types::{ClipKind, PreviewRenderSettings};
use super::utils::{
    acquire_preview_extraction, frame_cache_dir, probe_duration_and_audio, sharp_frame_cache_dir,
    uuid_str,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ─── Медіа-файл у пулі ───────────────────────────────────────────────────────

fn save_preview_jpeg(img: &image::DynamicImage, out: &Path, quality: u8) -> image::ImageResult<()> {
    let rgb = img.to_rgb8();
    let file = std::fs::File::create(out)?;
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
    encoder.encode_image(&rgb)
}

#[derive(Clone)]
pub struct MediaItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub duration_secs: f32,
    pub kind: ClipKind,
    /// Папка де зберігаються легкі scrub-кадри: .frame_cache/{path_hash}_{version}/
    pub cache_dir: PathBuf,
    /// Папка для чітких still-кадрів, які генеруються на вимогу.
    pub sharp_cache_dir: PathBuf,
    /// true = прев'ю для медіа готове; для відео кадри витягуються ліниво на вимогу
    pub extraction_complete: Arc<AtomicBool>,
    /// true = відеофайл містить вбудовану аудіодоріжку
    pub has_audio: bool,
    /// Результат фонового ffprobe (duration_secs, has_audio, duration_verified).
    probe_result: Arc<Mutex<Option<(f32, bool, bool)>>>,
    duration_verified: bool,
    duration_probe_attempted_sync: bool,
}

impl MediaItem {
    /// Створює медіа-елемент.
    /// Легкі кадри для превʼю генеруються ліниво, тільки коли вони реально потрібні UI.
    /// `cache_base` — базова папка задачі (save_path).
    pub fn new(path: PathBuf, cache_base: &Path, preview: PreviewRenderSettings) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_image = matches!(
            ext.as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"
        );
        let is_video = matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm");
        let is_audio = matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac");

        let kind = if is_video {
            ClipKind::Video
        } else if is_audio {
            ClipKind::Audio
        } else {
            ClipKind::Image
        };

        // Тривалість та наявність аудіо зондуємо у фоновому потоці,
        // щоб не блокувати UI-трід ffprobe-викликами.
        let probe_result: Arc<Mutex<Option<(f32, bool, bool)>>> = Arc::new(Mutex::new(None));
        let (duration_secs, has_audio) = if is_image {
            // Для зображень — відразу встановлюємо значення без зонду
            *probe_result.lock().unwrap() = Some((5.0, false, true));
            (5.0, false)
        } else if is_video || is_audio {
            let probe_arc_c = Arc::clone(&probe_result);
            let path_c = path.clone();
            let is_vid = is_video;
            std::thread::spawn(move || {
                let (dur, has_audio) = probe_duration_and_audio(&path_c);
                let duration_verified = dur.is_some();
                let duration = dur.unwrap_or(10.0);
                let audio = is_vid && has_audio;
                if let Ok(mut g) = probe_arc_c.lock() {
                    *g = Some((duration, audio, duration_verified));
                }
            });
            // Тимчасові значення до завершення зонду
            (10.0, false)
        } else {
            *probe_result.lock().unwrap() = Some((5.0, false, true));
            (5.0, false)
        };

        // Стабільні папки кешу на основі хешу шляху + версії якості превʼю.
        let cache_dir = frame_cache_dir(cache_base, &path, preview);
        let sharp_cache_dir = sharp_frame_cache_dir(cache_base, &path, preview);
        let extraction_complete = Arc::new(AtomicBool::new(false));

        if cache_dir.join(".complete").exists() || cache_dir.join("000001.jpg").exists() {
            // Вже є готовий прев'ю-кадр з попередньої сесії.
            extraction_complete.store(true, Ordering::Relaxed);
        } else if is_image {
            let path_clone = path.clone();
            let dir = cache_dir.clone();
            let flag = extraction_complete.clone();
            let width = preview.quality.scrub_width();
            let quality = preview.quality.jpeg_quality();
            std::thread::spawn(move || {
                let _permit = acquire_preview_extraction();
                std::fs::create_dir_all(&dir).ok();
                let out = dir.join("000001.jpg");

                for delay_ms in [0u64, 250, 800, 2000] {
                    if delay_ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    }
                    if out.exists() {
                        flag.store(true, Ordering::Relaxed);
                        return;
                    }
                    if let Ok(bytes) = std::fs::read(&path_clone) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let thumb = img.thumbnail(width, width * 2);
                            if save_preview_jpeg(&thumb, &out, quality).is_ok() {
                                let _ = std::fs::write(dir.join(".complete"), b"1");
                                flag.store(true, Ordering::Relaxed);
                                return;
                            }
                        }
                    }
                }
            });
        } else if is_video {
            // Відео не проганяємо всі разом при відкритті проекту.
            // Активний кліп сам запускає пріоритетний chunk-preload біля playhead у FrameCache.
            extraction_complete.store(false, Ordering::Relaxed);
        } else {
            // Аудіо: вилучення не потрібне
            extraction_complete.store(true, Ordering::Relaxed);
        }

        Self {
            id: uuid_str(),
            path,
            name,
            duration_secs,
            kind,
            cache_dir,
            sharp_cache_dir,
            extraction_complete,
            has_audio,
            probe_result,
            duration_verified: is_image || (!is_video && !is_audio),
            duration_probe_attempted_sync: false,
        }
    }

    /// Оновлює duration_secs та has_audio якщо фоновий ffprobe-зонд завершився.
    /// Повертає true якщо дані змінились — сигнал що UI треба перемалювати.
    pub fn refresh_probe(&mut self) -> bool {
        if let Ok(mut guard) = self.probe_result.try_lock() {
            if let Some((dur, audio, duration_verified)) = guard.take() {
                self.duration_secs = dur;
                self.has_audio = audio;
                self.duration_verified = duration_verified;
                return true;
            }
        }
        false
    }

    /// Гарантує точну тривалість активного відео перед розрахунком бумерангу.
    pub fn ensure_duration_verified(&mut self) -> bool {
        let mut changed = self.refresh_probe();
        if self.duration_verified
            || self.duration_probe_attempted_sync
            || !matches!(self.kind, ClipKind::Video)
        {
            return changed;
        }

        self.duration_probe_attempted_sync = true;
        let (duration, has_audio) = probe_duration_and_audio(&self.path);
        if let Some(duration) = duration {
            changed |= (self.duration_secs - duration).abs() > 0.001 || self.has_audio != has_audio;
            self.duration_secs = duration;
            self.has_audio = has_audio;
            self.duration_verified = true;
        }
        changed
    }

    pub fn is_extraction_complete(&self) -> bool {
        if self.extraction_complete.load(Ordering::Relaxed) {
            return true;
        }
        // Перший кадр вже є — можна показувати (витягування ще йде, але не блокуємо UI)
        if self.cache_dir.join("000001.jpg").exists() {
            return true;
        }
        // Fallback після перезапуску: перевіряємо маркер на диску
        if self.cache_dir.join(".complete").exists() {
            self.extraction_complete.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}
