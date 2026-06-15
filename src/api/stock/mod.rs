pub mod pexels;
pub mod pixabay;

use std::io::Read;

// ─── Серіалізовані типи для stock_cache.json ─────────────────────────────────

/// Одна стокова фотографія у кеші
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedPhoto {
    pub id: String,
    pub preview_url: String,
    pub original_url: String,
    pub width: u32,
    pub height: u32,
    pub author: String,
}

/// Одне стокове відео у кеші
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedVideo {
    pub id: String,
    pub thumbnail_url: String,
    pub duration_secs: u32,
    pub download_url: String,
    pub width: u32,
    pub height: u32,
    pub author: String,
}

/// Обраний медіафайл для сегмента
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SelectedMedia {
    /// "photo" або "video"
    pub kind: String,
    pub id: String,
    /// URL для завантаження
    pub url: String,
    /// Ім'я файлу в папці media/ (наприклад "0001.jpg")
    pub filename: String,
    /// Початок обрізки для відео (секунди від початку файлу)
    #[serde(default)]
    pub trim_start: f32,
    /// Кінець обрізки для відео (0.0 = до кінця)
    #[serde(default)]
    pub trim_end: f32,
}

/// Результати пошуку для одного сегмента
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SegmentCache {
    pub index: usize,
    pub keyword: String,
    pub segment_text: String,
    /// Тривалість сегменту з timeline.json (секунди)
    #[serde(default)]
    pub segment_duration: f32,
    #[serde(default)]
    pub photos: Vec<CachedPhoto>,
    #[serde(default)]
    pub videos: Vec<CachedVideo>,
    /// Обраний медіафайл; None — ще не обрано
    pub selected: Option<SelectedMedia>,
}

impl From<&StockPhoto> for CachedPhoto {
    fn from(p: &StockPhoto) -> Self {
        Self {
            id: p.id.clone(),
            preview_url: p.preview_url.clone(),
            original_url: p.original_url.clone(),
            width: p.width,
            height: p.height,
            author: p.author.clone(),
        }
    }
}

impl From<&StockVideo> for CachedVideo {
    fn from(v: &StockVideo) -> Self {
        Self {
            id: v.id.clone(),
            thumbnail_url: v.thumbnail_url.clone(),
            duration_secs: v.duration_secs,
            download_url: v.download_url.clone(),
            width: v.width,
            height: v.height,
            author: v.author.clone(),
        }
    }
}

/// Читає stock_cache.json з папки задачі
pub fn load_cache(save_dir: &std::path::Path) -> Option<Vec<SegmentCache>> {
    let path = save_dir.join("stock_cache.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Зберігає stock_cache.json у папку задачі
pub fn save_cache(save_dir: &std::path::Path, cache: &[SegmentCache]) -> Result<(), String> {
    let path = save_dir.join("stock_cache.json");
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("JSON error: {e}"))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Write error: {e}"))
}

/// Стокове фото з довільного провайдера
#[derive(Clone, Debug)]
pub struct StockPhoto {
    pub id: String,
    /// URL мініатюри для показу в пікері
    pub preview_url: String,
    /// URL оригінального розміру для завантаження
    pub original_url: String,
    pub width: u32,
    pub height: u32,
    pub author: String,
}

/// Стокове відео з довільного провайдера
#[derive(Clone, Debug)]
pub struct StockVideo {
    pub id: String,
    /// URL мініатюри (зображення) для показу в пікері
    pub thumbnail_url: String,
    pub duration_secs: u32,
    /// URL найкращого HD файлу для завантаження
    pub download_url: String,
    pub width: u32,
    pub height: u32,
    pub author: String,
}

/// Трейт для провайдерів стокових медіа
pub trait StockProvider: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn search_photos(&self, key: &str, query: &str, per_page: u32) -> Result<Vec<StockPhoto>, String>;
    fn search_videos(&self, key: &str, query: &str, per_page: u32) -> Result<Vec<StockVideo>, String>;
}

/// Завантажує файл з URL і зберігає на диск
pub fn download_file(url: &str, dest: &std::path::Path) -> Result<(), String> {
    download_file_with_progress(url, dest, None)
}

/// Завантажує файл з відстеженням прогресу (0.0..1.0).
/// -1.0 у progress означає помилку.
pub fn download_file_with_progress(
    url: &str,
    dest: &std::path::Path,
    progress: Option<&std::sync::Arc<std::sync::Mutex<f32>>>,
) -> Result<(), String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(120))
        .build();

    let response = agent
        .get(url)
        .call()
        .map_err(|e| format!("Помилка завантаження: {e}"))?;

    let total_size = response.header("Content-Length")
        .and_then(|s| s.parse::<usize>().ok());

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    let mut buf = [0u8; 65536];

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                bytes.extend_from_slice(&buf[..n]);
                if let (Some(prog), Some(total)) = (progress, total_size) {
                    if total > 0 {
                        *prog.lock().unwrap() = bytes.len() as f32 / total as f32;
                    }
                }
            }
            Err(e) => {
                if let Some(prog) = progress {
                    *prog.lock().unwrap() = -1.0;
                }
                return Err(format!("Помилка читання: {e}"));
            }
        }
    }

    if let Some(prog) = progress {
        *prog.lock().unwrap() = 1.0;
    }
    std::fs::write(dest, &bytes)
        .map_err(|e| format!("Помилка збереження: {e}"))
}
