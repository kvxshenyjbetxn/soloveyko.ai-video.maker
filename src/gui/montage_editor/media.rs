use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use super::types::{ClipKind, PreviewRenderSettings};
use super::utils::{frame_cache_dir, probe_duration, probe_has_audio, sharp_frame_cache_dir, uuid_str};

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
    /// true = всі легкі кадри вже витягнуто на диск
    pub extraction_complete: Arc<AtomicBool>,
    /// true = відеофайл містить вбудовану аудіодоріжку
    pub has_audio: bool,
    /// Результат фонового ffprobe (duration_secs, has_audio); None = ще виконується
    probe_result: Arc<Mutex<Option<(f32, bool)>>>,
}

impl MediaItem {
    /// Створює медіа-елемент і запускає фонове витягування кадрів на диск.
    /// `cache_base` — базова папка задачі (save_path).
    pub fn new(path: PathBuf, cache_base: &Path, preview: PreviewRenderSettings) -> Self {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_image = matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp");
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
        let probe_result: Arc<Mutex<Option<(f32, bool)>>> = Arc::new(Mutex::new(None));
        let (duration_secs, has_audio) = if is_image {
            // Для зображень — відразу встановлюємо значення без зонду
            *probe_result.lock().unwrap() = Some((5.0, false));
            (5.0, false)
        } else if is_video || is_audio {
            let probe_arc_c = Arc::clone(&probe_result);
            let path_c = path.clone();
            let is_vid = is_video;
            std::thread::spawn(move || {
                let dur = probe_duration(&path_c).unwrap_or(10.0);
                let audio = is_vid && probe_has_audio(&path_c);
                if let Ok(mut g) = probe_arc_c.lock() {
                    *g = Some((dur, audio));
                }
            });
            // Тимчасові значення до завершення зонду
            (10.0, false)
        } else {
            *probe_result.lock().unwrap() = Some((5.0, false));
            (5.0, false)
        };

        // Стабільні папки кешу на основі хешу шляху + версії якості превʼю.
        let cache_dir = frame_cache_dir(cache_base, &path, preview);
        let sharp_cache_dir = sharp_frame_cache_dir(cache_base, &path, preview);
        let extraction_complete = Arc::new(AtomicBool::new(false));

        if cache_dir.join(".complete").exists() {
            // Вже витягнуто в попередній сесії
            extraction_complete.store(true, Ordering::Relaxed);
        } else if is_image {
            // Зображення: декодуємо через image crate (без ffmpeg)
            let path_clone = path.clone();
            let dir = cache_dir.clone();
            let flag = extraction_complete.clone();
            std::thread::spawn(move || {
                std::fs::create_dir_all(&dir).ok();
                let out = dir.join("000001.jpg");
                // Retry з затримками: файл може бути ще не повністю записаний пайплайном
                // Використовуємо read+load_from_memory (як галерея) — стабільніше ніж open()
                for delay_ms in [0u64, 400, 1200, 3500] {
                    if delay_ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    }
                    if out.exists() { break; }
                    if let Ok(bytes) = std::fs::read(&path_clone) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let width = preview.quality.scrub_width();
                            let thumb = img.thumbnail(width, width * 2);
                            if save_preview_jpeg(&thumb, &out, preview.quality.jpeg_quality()).is_ok() {
                                std::fs::write(dir.join(".complete"), b"1").ok();
                                break;
                            }
                        }
                    }
                }
                flag.store(true, Ordering::Relaxed);
            });
        } else if is_video {
            // Відео: витягуємо всі кадри через ffmpeg
            let path_str = path.to_string_lossy().to_string();
            let dir = cache_dir.clone();
            let flag = extraction_complete.clone();
            let scrub_w = preview.quality.scrub_width();
            let qscale = preview.quality.ffmpeg_qscale();
            let fps_val = preview.fps;
            std::thread::spawn(move || {
                std::fs::create_dir_all(&dir).ok();

                // Крок 1: Швидко витягуємо перший кадр щоб UI одразу мав що показати.
                let first_frame = dir.join("000001.jpg");
                if !first_frame.exists() {
                    let mut quick = std::process::Command::new(crate::bundle::ffmpeg_path());
                    quick.args([
                        "-y", "-v", "error", "-threads", "1",
                        "-i", &path_str,
                        "-vframes", "1",
                        "-vf", &format!("scale={}:-2", scrub_w),
                        "-q:v", qscale,
                        first_frame.to_str().unwrap_or(""),
                    ]);
                    crate::bundle::set_no_window(&mut quick);
                    let _ = crate::api::ffmpeg::run_tracked(&mut quick);
                }

                // Крок 2: Повна екстракція всіх кадрів для скрабінгу.
                let out_pattern = dir.join("%06d.jpg");
                let Some(out_str) = out_pattern.to_str() else {
                    flag.store(true, Ordering::Relaxed);
                    return;
                };
                let mut ffmpeg_preview = std::process::Command::new(crate::bundle::ffmpeg_path());
                ffmpeg_preview.args([
                    "-y", "-v", "error", "-threads", "1",
                    "-i", &path_str,
                    "-vf", &format!("scale={}:-2,fps={}", scrub_w, fps_val),
                    "-q:v", qscale,
                    out_str,
                ]);
                crate::bundle::set_no_window(&mut ffmpeg_preview);
                let status = crate::api::ffmpeg::run_tracked(&mut ffmpeg_preview);
                if matches!(status, Ok(s) if s.success()) {
                    std::fs::write(dir.join(".complete"), b"1").ok();
                }
                // Незалежно від результату — перший кадр вже є з кроку 1
                flag.store(true, Ordering::Relaxed);
            });
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
        }
    }

    /// Оновлює duration_secs та has_audio якщо фоновий ffprobe-зонд завершився.
    /// Повертає true якщо дані змінились — сигнал що UI треба перемалювати.
    pub fn refresh_probe(&mut self) -> bool {
        if let Ok(mut guard) = self.probe_result.try_lock() {
            if let Some((dur, audio)) = guard.take() {
                self.duration_secs = dur;
                self.has_audio = audio;
                return true;
            }
        }
        false
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
