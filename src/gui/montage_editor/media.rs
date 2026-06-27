use super::types::{ClipKind, PreviewRenderSettings};
use super::utils::{frame_cache_dir, probe_duration_and_audio, sharp_frame_cache_dir, uuid_str};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ─── Медіа-файл у пулі ───────────────────────────────────────────────────────

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
    /// Результат фонового ffprobe (duration_secs, has_audio); None = ще виконується
    probe_result: Arc<Mutex<Option<(f32, bool)>>>,
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
                let (dur, has_audio) = probe_duration_and_audio(&path_c);
                let duration = dur.unwrap_or(10.0);
                let audio = is_vid && has_audio;
                if let Ok(mut g) = probe_arc_c.lock() {
                    *g = Some((duration, audio));
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

        if cache_dir.join(".complete").exists() || cache_dir.join("000001.jpg").exists() {
            // Вже є готовий прев'ю-кадр з попередньої сесії.
            extraction_complete.store(true, Ordering::Relaxed);
        } else if is_image {
            // Зображення більше не декодуємо масово під час відкриття редактора.
            // Кадр буде створено ліниво у FrameCache, коли він реально знадобиться.
        } else if is_video {
            // Відео не обробляємо наперед: кадри витягуються тільки коли їх реально
            // просить preview/thumbnail. Це прибирає масовий старт ffmpeg при
            // відкритті редактора з великим пулом відео.
            extraction_complete.store(true, Ordering::Relaxed);
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
