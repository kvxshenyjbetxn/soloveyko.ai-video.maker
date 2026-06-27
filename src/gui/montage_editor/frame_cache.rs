use super::media::MediaItem;
use super::types::{ClipKind, PreviewRenderSettings};
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

const PREFETCH_AHEAD_FRAMES: u32 = 2;
const MAX_PARALLEL_FRAME_LOADS: usize = 3;

type FrameLoadResult = (String, Option<egui::TextureHandle>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameQuality {
    Scrub,
    Sharp,
}

// ─── LRU кеш кадрів превью ───────────────────────────────────────────────────

pub struct FrameCache {
    textures: HashMap<String, egui::TextureHandle>,
    access_order: VecDeque<String>,
    max_size: usize,
    rx: Option<std::sync::mpsc::Receiver<FrameLoadResult>>,
    tx: std::sync::mpsc::Sender<FrameLoadResult>,
    loading_keys: HashSet<String>,
}

impl FrameCache {
    pub fn new(max_size: usize) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            textures: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            rx: Some(rx),
            tx,
            loading_keys: HashSet::new(),
        }
    }

    /// Допоміжний метод для завантаження кадру з диска (виконується в потоці).
    fn load_frame_from_disk(
        ctx: &egui::Context,
        frame_path: &Path,
        key: &str,
    ) -> Option<egui::TextureHandle> {
        if !frame_path.exists() {
            return None;
        }
        let bytes = std::fs::read(frame_path).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let ci =
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba.into_raw());
        Some(ctx.load_texture(key, ci, egui::TextureOptions::LINEAR))
    }

    fn save_jpeg_atomic(
        img: &image::DynamicImage,
        out: &Path,
        quality: u8,
    ) -> image::ImageResult<()> {
        let rgb = img.to_rgb8();
        let ext = out.extension().and_then(|s| s.to_str()).unwrap_or("jpg");
        let tmp = out.with_extension(format!("{ext}.tmp"));
        let _ = std::fs::remove_file(&tmp);

        let file = std::fs::File::create(&tmp)?;
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
        encoder.encode_image(&rgb)?;

        if out.exists() {
            let _ = std::fs::remove_file(out);
        }
        std::fs::rename(&tmp, out)?;
        Ok(())
    }

    fn frame_idx(media: &MediaItem, time: f32, settings: PreviewRenderSettings) -> u32 {
        if matches!(media.kind, ClipKind::Image) {
            1
        } else {
            (time.clamp(0.0, media.duration_secs) * settings.fps).round() as u32 + 1
        }
    }

    fn quality_prefix(media: &MediaItem, quality: FrameQuality) -> String {
        let prefix = match quality {
            FrameQuality::Scrub => "s",
            FrameQuality::Sharp => "h",
        };
        format!("{}_{}", media.id, prefix)
    }

    fn frame_key(media: &MediaItem, frame_idx: u32, quality: FrameQuality) -> String {
        format!("{}_{:06}", Self::quality_prefix(media, quality), frame_idx)
    }

    fn frame_index_from_key(prefix: &str, key: &str) -> Option<u32> {
        key.strip_prefix(prefix)
            .and_then(|tail| tail.strip_prefix('_'))
            .and_then(|tail| tail.parse::<u32>().ok())
    }

    fn frame_path(media: &MediaItem, frame_idx: u32, quality: FrameQuality) -> PathBuf {
        match quality {
            FrameQuality::Scrub => media.cache_dir.join(format!("{:06}.jpg", frame_idx)),
            FrameQuality::Sharp => media.sharp_cache_dir.join(format!("{:06}.jpg", frame_idx)),
        }
    }

    fn touch_key(&mut self, key: &str) {
        if let Some(pos) = self.access_order.iter().position(|x| x == key) {
            self.access_order.remove(pos);
        }
        self.access_order.push_back(key.to_string());
    }

    fn insert_texture(&mut self, key: String, texture: egui::TextureHandle) {
        if !self.textures.contains_key(&key) {
            while self.textures.len() >= self.max_size {
                if let Some(oldest) = self.access_order.pop_front() {
                    self.textures.remove(&oldest);
                } else {
                    break;
                }
            }
        }
        self.textures.insert(key.clone(), texture);
        self.touch_key(&key);
    }

    fn drain_loaded_frames(&mut self) {
        if let Some(ref rx) = self.rx {
            let mut loaded = Vec::new();
            while let Ok(item) = rx.try_recv() {
                loaded.push(item);
            }
            for (key, texture) in loaded {
                self.loading_keys.remove(&key);
                if let Some(texture) = texture {
                    self.insert_texture(key, texture);
                }
            }
        }
    }

    fn generate_sharp_frame(
        media: &MediaItem,
        frame_idx: u32,
        settings: PreviewRenderSettings,
        out: &Path,
    ) -> bool {
        if std::fs::create_dir_all(&media.sharp_cache_dir).is_err() {
            return false;
        }

        match media.kind {
            ClipKind::Image => {
                let Ok(bytes) = std::fs::read(&media.path) else {
                    return false;
                };
                let Ok(img) = image::load_from_memory(&bytes) else {
                    return false;
                };
                let width = settings.quality.sharp_width();
                let thumb = img.thumbnail(width, width * 2);
                Self::save_jpeg_atomic(&thumb, out, settings.quality.sharp_jpeg_quality()).is_ok()
            }
            ClipKind::Video => {
                let seek = (frame_idx.saturating_sub(1) as f32 / settings.fps)
                    .min(media.duration_secs)
                    .max(0.0);
                let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
                cmd.args([
                    "-y",
                    "-v",
                    "error",
                    "-threads",
                    "1",
                    "-ss",
                    &format!("{seek:.3}"),
                    "-i",
                    media.path.to_string_lossy().as_ref(),
                    "-vframes",
                    "1",
                    "-vf",
                    &format!("scale={}:-2", settings.quality.sharp_width()),
                    "-q:v",
                    settings.quality.sharp_ffmpeg_qscale(),
                    out.to_string_lossy().as_ref(),
                ]);
                crate::bundle::set_no_window(&mut cmd);
                matches!(crate::api::ffmpeg::run_tracked(&mut cmd), Ok(status) if status.success())
                    && out.exists()
            }
            ClipKind::Audio => false,
        }
    }

    fn generate_scrub_frame(
        media: &MediaItem,
        frame_idx: u32,
        settings: PreviewRenderSettings,
        out: &Path,
    ) -> bool {
        if std::fs::create_dir_all(&media.cache_dir).is_err() {
            return false;
        }

        match media.kind {
            ClipKind::Image => {
                let Ok(bytes) = std::fs::read(&media.path) else {
                    return false;
                };
                let Ok(img) = image::load_from_memory(&bytes) else {
                    return false;
                };
                let width = settings.quality.scrub_width();
                let thumb = img.thumbnail(width, width * 2);
                Self::save_jpeg_atomic(&thumb, out, settings.quality.jpeg_quality()).is_ok()
            }
            ClipKind::Video => {
                let seek = (frame_idx.saturating_sub(1) as f32 / settings.fps)
                    .min(media.duration_secs)
                    .max(0.0);
                let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
                cmd.args([
                    "-y",
                    "-v",
                    "error",
                    "-threads",
                    "1",
                    "-ss",
                    &format!("{seek:.3}"),
                    "-i",
                    media.path.to_string_lossy().as_ref(),
                    "-vframes",
                    "1",
                    "-vf",
                    &format!("scale={}:-2", settings.quality.scrub_width()),
                    "-q:v",
                    settings.quality.ffmpeg_qscale(),
                    out.to_string_lossy().as_ref(),
                ]);
                crate::bundle::set_no_window(&mut cmd);
                matches!(crate::api::ffmpeg::run_tracked(&mut cmd), Ok(status) if status.success())
                    && out.exists()
            }
            ClipKind::Audio => false,
        }
    }

    /// Ставить кадр у фонову чергу. UI-потік ніколи не чекає декодування JPEG.
    fn request_frame_async(
        &mut self,
        ctx: &egui::Context,
        media: &MediaItem,
        frame_idx: u32,
        settings: PreviewRenderSettings,
        quality: FrameQuality,
    ) {
        let key = Self::frame_key(media, frame_idx, quality);
        if self.textures.contains_key(&key) || self.loading_keys.contains(&key) {
            return;
        }
        if self.loading_keys.len() >= MAX_PARALLEL_FRAME_LOADS {
            return;
        }

        let frame_path = Self::frame_path(media, frame_idx, quality);

        self.loading_keys.insert(key.clone());
        let tx = self.tx.clone();
        let ctx_clone = ctx.clone();
        let media_clone = media.clone();
        std::thread::spawn(move || {
            let ready = if frame_path.exists() {
                true
            } else {
                match quality {
                    FrameQuality::Sharp => {
                        Self::generate_sharp_frame(&media_clone, frame_idx, settings, &frame_path)
                    }
                    FrameQuality::Scrub => {
                        Self::generate_scrub_frame(&media_clone, frame_idx, settings, &frame_path)
                    }
                }
            };

            if ready {
                media_clone
                    .extraction_complete
                    .store(true, Ordering::Relaxed);
                if matches!(quality, FrameQuality::Scrub) {
                    let _ = std::fs::write(media_clone.cache_dir.join(".complete"), b"1");
                }
            }

            let texture = if ready {
                Self::load_frame_from_disk(&ctx_clone, &frame_path, &key)
            } else {
                None
            };
            let _ = tx.send((key, texture));
            ctx_clone.request_repaint();
        });
    }

    /// Повертає найближчий вже готовий кадр цієї якості.
    /// Шукає в обох напрямках — це прибирає мерехтіння при скрабінгу назад,
    /// коли кеш містить кадри "попереду" поточної позиції плейхеду.
    fn cached_near_frame(
        &self,
        media: &MediaItem,
        frame_idx: u32,
        quality: FrameQuality,
        max_distance: u32,
    ) -> Option<egui::TextureHandle> {
        let prefix = Self::quality_prefix(media, quality);
        self.access_order
            .iter()
            .rev()
            .filter_map(|key| {
                let idx = Self::frame_index_from_key(&prefix, key)?;
                let dist = if idx <= frame_idx {
                    frame_idx - idx
                } else {
                    idx - frame_idx
                };
                if dist <= max_distance {
                    Some((dist, key))
                } else {
                    None
                }
            })
            .min_by_key(|(distance, _)| *distance)
            .and_then(|(_, key)| self.textures.get(key))
            .cloned()
    }

    fn request_first_frame_if_needed(
        &mut self,
        ctx: &egui::Context,
        media: &MediaItem,
        frame_idx: u32,
        settings: PreviewRenderSettings,
    ) {
        if frame_idx == 1
            || self
                .cached_near_frame(
                    media,
                    frame_idx,
                    FrameQuality::Scrub,
                    settings.fps.ceil() as u32,
                )
                .is_some()
        {
            return;
        }
        self.request_frame_async(ctx, media, 1, settings, FrameQuality::Scrub);
    }

    /// Повертає текстуру для заданого медіа та часу.
    /// `sharp_when_idle` вмикає high-res still-кадр тільки коли користувач не скрабить.
    pub fn get_frame(
        &mut self,
        ctx: &egui::Context,
        media: &MediaItem,
        time: f32,
        sharp_when_idle: bool,
        settings: PreviewRenderSettings,
    ) -> Option<egui::TextureHandle> {
        if matches!(media.kind, ClipKind::Audio) {
            return None;
        }

        let frame_idx = Self::frame_idx(media, time, settings);
        self.drain_loaded_frames();

        if sharp_when_idle {
            let sharp_key = Self::frame_key(media, frame_idx, FrameQuality::Sharp);
            if let Some(texture) = self.textures.get(&sharp_key).cloned() {
                self.touch_key(&sharp_key);
                return Some(texture);
            }
            self.request_frame_async(ctx, media, frame_idx, settings, FrameQuality::Sharp);
        }

        let scrub_key = Self::frame_key(media, frame_idx, FrameQuality::Scrub);
        if let Some(texture) = self.textures.get(&scrub_key).cloned() {
            self.touch_key(&scrub_key);
            if !sharp_when_idle && !matches!(media.kind, ClipKind::Image) {
                for i in 1..=PREFETCH_AHEAD_FRAMES {
                    self.request_frame_async(
                        ctx,
                        media,
                        frame_idx + i,
                        settings,
                        FrameQuality::Scrub,
                    );
                }
            }
            return Some(texture);
        }

        // Cache miss — не декодуємо в UI-потоці. Це прибирає лаги при перетягуванні плейхеду.
        self.request_frame_async(ctx, media, frame_idx, settings, FrameQuality::Scrub);
        self.request_first_frame_if_needed(ctx, media, frame_idx, settings);
        ctx.request_repaint();

        // Фолбек на найближчий кадр (в будь-який бік) поки async-завантаження не завершилось.
        // fps/3 ≈ 8-10 кадрів — достатньо для плавного скрабінгу в обидва боки.
        let max_distance = (settings.fps / 3.0).ceil().max(3.0) as u32;
        self.cached_near_frame(media, frame_idx, FrameQuality::Scrub, max_distance)
    }

    /// Видаляє всі закешовані кадри для конкретного media_id (після перегенерації).
    pub fn clear_for_media_id(&mut self, media_id: &str) {
        let prefix = format!("{}_", media_id);
        self.textures.retain(|k, _| !k.starts_with(&prefix));
        self.access_order.retain(|k| !k.starts_with(&prefix));
        self.loading_keys.retain(|k| !k.starts_with(&prefix));
    }
}
