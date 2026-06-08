use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use super::types::{ClipKind, PREVIEW_FPS, PREVIEW_WIDTH};
use super::utils::{path_hash, probe_duration, uuid_str};

// ─── Медіа-файл у пулі ───────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MediaItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub duration_secs: f32,
    pub kind: ClipKind,
    /// Папка де зберігаються превью-кадри: .frame_cache/{path_hash}/
    pub cache_dir: PathBuf,
    /// true = всі кадри вже витягнуто на диск
    pub extraction_complete: Arc<AtomicBool>,
}

impl MediaItem {
    /// Створює медіа-елемент і запускає фонове витягування кадрів на диск.
    /// `cache_base` — базова папка задачі (save_path).
    pub fn new(path: PathBuf, cache_base: &Path) -> Self {
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

        let duration_secs = if is_image {
            5.0
        } else if is_video || is_audio {
            probe_duration(&path).unwrap_or(10.0)
        } else {
            5.0
        };

        // Стабільна папка кешу на основі хешу шляху
        let cache_dir = cache_base.join(".frame_cache").join(path_hash(&path));
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
                if let Ok(img) = image::open(&path_clone) {
                    let thumb = img.thumbnail(PREVIEW_WIDTH, PREVIEW_WIDTH * 2);
                    let out = dir.join("000001.jpg");
                    if thumb.save(&out).is_ok() {
                        std::fs::write(dir.join(".complete"), b"1").ok();
                        flag.store(true, Ordering::Relaxed);
                    }
                }
            });
        } else if is_video {
            // Відео: витягуємо всі кадри через ffmpeg
            let path_str = path.to_string_lossy().to_string();
            let dir = cache_dir.clone();
            let flag = extraction_complete.clone();
            std::thread::spawn(move || {
                std::fs::create_dir_all(&dir).ok();
                let out_pattern = dir.join("%06d.jpg");
                let Some(out_str) = out_pattern.to_str() else { return };
                let mut ffmpeg_preview = std::process::Command::new(crate::bundle::ffmpeg_path());
                ffmpeg_preview.args([
                    "-y", "-v", "error", "-threads", "1",
                    "-i", &path_str,
                    "-vf", &format!("scale=640:-2,fps={}", PREVIEW_FPS),
                    "-q:v", "5",
                    out_str,
                ]);
                crate::bundle::set_no_window(&mut ffmpeg_preview);
                let status = ffmpeg_preview.status();
                if matches!(status, Ok(s) if s.success()) {
                    std::fs::write(dir.join(".complete"), b"1").ok();
                    flag.store(true, Ordering::Relaxed);
                } else {
                    // Fallback: витягуємо тільки перший кадр без фільтрів
                    let first_frame = dir.join("000001.jpg");
                    let mut ffmpeg_fallback = std::process::Command::new(crate::bundle::ffmpeg_path());
                    ffmpeg_fallback.args([
                        "-y", "-v", "error", "-threads", "1",
                        "-i", &path_str,
                        "-vframes", "1",
                        "-q:v", "5",
                        first_frame.to_str().unwrap_or(""),
                    ]);
                    crate::bundle::set_no_window(&mut ffmpeg_fallback);
                    let _ = ffmpeg_fallback.status();
                    flag.store(true, Ordering::Relaxed);
                }
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
            extraction_complete,
        }
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
