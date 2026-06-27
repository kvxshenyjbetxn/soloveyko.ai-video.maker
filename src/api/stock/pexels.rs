use super::{StockPhoto, StockProvider, StockVideo};
use serde::Deserialize;

pub struct PexelsProvider;

// ─── Десеріалізація відповідей Pexels API ────────────────────────────────────

/// null у JSON → порожній рядок
fn null_to_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Option::<String>::deserialize(d).map(|o| o.unwrap_or_default())
}

/// null або відсутнє поле → 0
fn null_to_u32<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    Option::<u32>::deserialize(d).map(|o| o.unwrap_or(0))
}

#[derive(Deserialize)]
struct PhotoSrc {
    #[serde(deserialize_with = "null_to_string")]
    medium: String,
    #[serde(deserialize_with = "null_to_string")]
    original: String,
}

#[derive(Deserialize)]
struct PexelsPhoto {
    id: u64,
    #[serde(default, deserialize_with = "null_to_string")]
    photographer: String,
    #[serde(default, deserialize_with = "null_to_u32")]
    width: u32,
    #[serde(default, deserialize_with = "null_to_u32")]
    height: u32,
    src: PhotoSrc,
}

#[derive(Deserialize)]
struct PhotosResponse {
    #[serde(default)]
    photos: Vec<PexelsPhoto>,
}

#[derive(Deserialize)]
struct VideoUser {
    #[serde(default, deserialize_with = "null_to_string")]
    name: String,
}

#[derive(Deserialize)]
struct VideoFile {
    #[serde(default, deserialize_with = "null_to_string")]
    quality: String,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(default, deserialize_with = "null_to_string")]
    link: String,
}

#[derive(Deserialize)]
struct PexelsVideo {
    id: u64,
    #[serde(default, deserialize_with = "null_to_u32")]
    duration: u32,
    #[serde(default, deserialize_with = "null_to_string")]
    image: String,
    user: VideoUser,
    #[serde(default)]
    video_files: Vec<VideoFile>,
}

#[derive(Deserialize)]
struct VideosResponse {
    videos: Vec<PexelsVideo>,
}

// ─── Реалізація трейту ────────────────────────────────────────────────────────

impl StockProvider for PexelsProvider {
    fn name(&self) -> &str {
        "Pexels"
    }

    fn search_photos(
        &self,
        key: &str,
        query: &str,
        per_page: u32,
    ) -> Result<Vec<StockPhoto>, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let response = agent
            .get("https://api.pexels.com/v1/search")
            .set("Authorization", key)
            .query("query", query)
            .query("per_page", &per_page.to_string())
            .query("orientation", "landscape")
            .call()
            .map_err(|e| format!("Pexels API error: {e}"))?;

        let data = response
            .into_json::<PhotosResponse>()
            .map_err(|e| format!("Pexels JSON error: {e}"))?;

        Ok(data
            .photos
            .into_iter()
            .map(|p| StockPhoto {
                id: p.id.to_string(),
                preview_url: p.src.medium,
                original_url: p.src.original,
                width: p.width,
                height: p.height,
                author: p.photographer,
            })
            .collect())
    }

    fn search_videos(
        &self,
        key: &str,
        query: &str,
        per_page: u32,
    ) -> Result<Vec<StockVideo>, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let response = agent
            .get("https://api.pexels.com/videos/search")
            .set("Authorization", key)
            .query("query", query)
            .query("per_page", &per_page.to_string())
            .query("orientation", "landscape")
            .call()
            .map_err(|e| format!("Pexels API error: {e}"))?;

        let data = response
            .into_json::<VideosResponse>()
            .map_err(|e| format!("Pexels JSON error: {e}"))?;

        Ok(data
            .videos
            .into_iter()
            .filter_map(|v| {
                // Обираємо найкращий HD landscape файл (width > height), потім будь-який HD, потім перший
                let best = v
                    .video_files
                    .iter()
                    .filter(|f| f.quality == "hd" && f.width.unwrap_or(0) > f.height.unwrap_or(0))
                    .max_by_key(|f| f.width.unwrap_or(0))
                    .or_else(|| {
                        v.video_files
                            .iter()
                            .filter(|f| f.quality == "hd")
                            .max_by_key(|f| f.width.unwrap_or(0))
                    })
                    .or_else(|| v.video_files.first())?;

                Some(StockVideo {
                    id: v.id.to_string(),
                    thumbnail_url: v.image,
                    duration_secs: v.duration,
                    download_url: best.link.clone(),
                    width: best.width.unwrap_or(0),
                    height: best.height.unwrap_or(0),
                    author: v.user.name,
                })
            })
            .collect())
    }
}

/// Перевіряє Pexels API ключ — повертає статусний рядок
pub fn check_key(key: &str) -> String {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build();

    match agent
        .get("https://api.pexels.com/v1/search")
        .set("Authorization", key)
        .query("query", "test")
        .query("per_page", "1")
        .call()
    {
        Ok(_) => "✔ Ключ валідний".to_string(),
        Err(ureq::Error::Status(401, _)) => "❌ Невірний ключ".to_string(),
        Err(ureq::Error::Status(code, _)) => format!("❌ Помилка ({})", code),
        Err(_) => "❌ Помилка мережі. Перевірте з'єднання.".to_string(),
    }
}
