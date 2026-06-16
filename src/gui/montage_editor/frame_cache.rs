use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use eframe::egui;
use super::types::{ClipKind, PREVIEW_FPS};
use super::media::MediaItem;

// ─── LRU кеш кадрів превью ───────────────────────────────────────────────────

pub struct FrameCache {
    textures: HashMap<String, egui::TextureHandle>,
    access_order: VecDeque<String>,
    max_size: usize,
    rx: Option<std::sync::mpsc::Receiver<(String, egui::TextureHandle)>>,
    tx: std::sync::mpsc::Sender<(String, egui::TextureHandle)>,
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

    /// Допоміжний метод для завантаження кадру з диска (виконується в потоці)
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
        let ci = egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgba.into_raw(),
        );
        Some(ctx.load_texture(key, ci, egui::TextureOptions::LINEAR))
    }

    /// Синхронне завантаження поточного кадру
    fn load_frame_sync(
        &self,
        ctx: &egui::Context,
        media: &MediaItem,
        frame_idx: u32,
    ) -> Option<egui::TextureHandle> {
        let frame_path = media.cache_dir.join(format!("{:06}.jpg", frame_idx));
        let frame_path = if frame_path.exists() {
            frame_path
        } else {
            let first = media.cache_dir.join("000001.jpg");
            if first.exists() { first } else { return None; }
        };
        let key = format!("{}_{:06}", media.id, frame_idx);
        Self::load_frame_from_disk(ctx, &frame_path, &key)
    }

    /// Запуск фонового завантаження наступних кадрів
    fn prefetch_ahead(
        &mut self,
        ctx: &egui::Context,
        media: &MediaItem,
        current_idx: u32,
        count: u32,
    ) {
        if matches!(media.kind, ClipKind::Image) || matches!(media.kind, ClipKind::Audio) {
            return;
        }

        let total_frames = (media.duration_secs * PREVIEW_FPS).round() as u32 + 1;

        for i in 1..=count {
            let next_idx = current_idx + i;
            if next_idx > total_frames {
                break;
            }

            let key = format!("{}_{:06}", media.id, next_idx);

            if !self.textures.contains_key(&key) && self.loading_keys.insert(key.clone()) {
                let tx = self.tx.clone();
                let ctx_clone = ctx.clone();
                let frame_path = media.cache_dir.join(format!("{:06}.jpg", next_idx));
                let key_clone = key.clone();

                std::thread::spawn(move || {
                    if let Some(texture) = Self::load_frame_from_disk(&ctx_clone, &frame_path, &key_clone) {
                        let _ = tx.send((key_clone, texture));
                    }
                });
            }
        }
    }

    /// Повертає текстуру для заданого медіа та часу.
    /// Читає JPG з диска асинхронно з підтримкою фонового префетчингу.
    pub fn get_frame(
        &mut self,
        ctx: &egui::Context,
        media: &MediaItem,
        time: f32,
    ) -> Option<egui::TextureHandle> {
        if matches!(media.kind, ClipKind::Audio) {
            return None;
        }

        let frame_idx = if matches!(media.kind, ClipKind::Image) {
            1u32
        } else {
            (time.clamp(0.0, media.duration_secs) * PREVIEW_FPS).round() as u32 + 1
        };

        let key = format!("{}_{:06}", media.id, frame_idx);

        // 1. Збираємо завантажені кадри з фону
        if let Some(ref rx) = self.rx {
            while let Ok((k, texture)) = rx.try_recv() {
                self.loading_keys.remove(&k);
                if !self.textures.contains_key(&k) {
                    if self.textures.len() >= self.max_size {
                        if let Some(oldest) = self.access_order.pop_front() {
                            self.textures.remove(&oldest);
                        }
                    }
                    self.textures.insert(k.clone(), texture);
                    self.access_order.push_back(k);
                }
            }
        }

        // 2. LRU hit
        if self.textures.contains_key(&key) {
            if let Some(pos) = self.access_order.iter().position(|x| x == &key) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(key.clone());

            // Префетч наступних кадрів
            self.prefetch_ahead(ctx, media, frame_idx, 10);

            return Some(self.textures[&key].clone());
        }

        // 3. Cache miss для поточного кадру (завантажуємо синхронно як fallback)
        let texture = self.load_frame_sync(ctx, media, frame_idx);
        if let Some(ref tex) = texture {
            self.loading_keys.remove(&key);
            if self.textures.len() >= self.max_size {
                if let Some(oldest) = self.access_order.pop_front() {
                    self.textures.remove(&oldest);
                }
            }
            self.textures.insert(key.clone(), tex.clone());
            self.access_order.push_back(key);
        }

        // Префетч наступних кадрів
        self.prefetch_ahead(ctx, media, frame_idx, 10);

        texture
    }

    /// Видаляє всі закешовані кадри для конкретного media_id (після перегенерації)
    pub fn clear_for_media_id(&mut self, media_id: &str) {
        let prefix = format!("{}_", media_id);
        self.textures.retain(|k, _| !k.starts_with(&prefix));
        self.access_order.retain(|k| !k.starts_with(&prefix));
        self.loading_keys.retain(|k| !k.starts_with(&prefix));
    }
}
