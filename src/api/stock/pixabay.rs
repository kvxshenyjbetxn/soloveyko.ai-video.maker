use serde::Deserialize;
use super::{StockPhoto, StockVideo, StockProvider};

pub struct PixabayProvider;

// ─── Десеріалізація відповідей Pixabay API ───────────────────────────────────

#[derive(Deserialize)]
struct ImageResponse {
    hits: Vec<PixabayImage>,
}

#[derive(Deserialize)]
struct PixabayImage {
    id: u64,
    #[serde(rename = "webformatURL")]
    webformat_url: String,
    #[serde(rename = "largeImageURL")]
    large_image_url: String,
    #[serde(rename = "imageWidth")]
    image_width: u32,
    #[serde(rename = "imageHeight")]
    image_height: u32,
    user: String,
}

#[derive(Deserialize)]
struct VideoResponse {
    hits: Vec<PixabayVideo>,
}

#[derive(Deserialize)]
struct PixabayVideo {
    id: u64,
    duration: u32,
    videos: PixabayVideoSizes,
    user: String,
}

#[derive(Deserialize)]
struct PixabayVideoSizes {
    large: PixabayVideoSize,
    medium: PixabayVideoSize,
}

#[derive(Deserialize)]
struct PixabayVideoSize {
    #[serde(default)]
    url: String,
    width: u32,
    height: u32,
    thumbnail: String,
}

// ─── Реалізація трейту ────────────────────────────────────────────────────────

impl StockProvider for PixabayProvider {
    fn name(&self) -> &str { "Pixabay" }

    fn search_photos(&self, key: &str, query: &str, per_page: u32) -> Result<Vec<StockPhoto>, String> {
        let encoded = query.replace(' ', "+");
        let url = format!(
            "https://pixabay.com/api/?key={}&q={}&per_page={}&image_type=photo&orientation=horizontal&safesearch=true",
            key, encoded, per_page
        );

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let data = agent
            .get(&url)
            .call()
            .map_err(|e| format!("Pixabay API error: {e}"))?
            .into_json::<ImageResponse>()
            .map_err(|e| format!("Pixabay JSON error: {e}"))?;

        Ok(data.hits.into_iter().map(|img| StockPhoto {
            id: img.id.to_string(),
            preview_url: img.webformat_url,
            original_url: img.large_image_url,
            width: img.image_width,
            height: img.image_height,
            author: img.user,
        }).collect())
    }

    fn search_videos(&self, key: &str, query: &str, per_page: u32) -> Result<Vec<StockVideo>, String> {
        let encoded = query.replace(' ', "+");
        let url = format!(
            "https://pixabay.com/api/videos/?key={}&q={}&per_page={}&safesearch=true",
            key, encoded, per_page
        );

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let data = agent
            .get(&url)
            .call()
            .map_err(|e| format!("Pixabay API error: {e}"))?
            .into_json::<VideoResponse>()
            .map_err(|e| format!("Pixabay JSON error: {e}"))?;

        Ok(data.hits.into_iter().filter_map(|v| {
            // large (1920x1080) якщо є, інакше medium
            let best = if !v.videos.large.url.is_empty() {
                &v.videos.large
            } else {
                &v.videos.medium
            };
            if best.url.is_empty() { return None; }

            Some(StockVideo {
                id: v.id.to_string(),
                thumbnail_url: best.thumbnail.clone(),
                duration_secs: v.duration,
                download_url: best.url.clone(),
                width: best.width,
                height: best.height,
                author: v.user,
            })
        }).collect())
    }
}

/// Перевіряє Pixabay API ключ — повертає статусний рядок.
/// Pixabay повертає 400 для невалідного ключа.
pub fn check_key(key: &str) -> String {
    let url = format!("https://pixabay.com/api/?key={}&q=test&per_page=3", key);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build();

    match agent.get(&url).call() {
        Ok(_) => "✔ Ключ валідний".to_string(),
        Err(ureq::Error::Status(400, _)) => "❌ Невірний ключ".to_string(),
        Err(ureq::Error::Status(code, _)) => format!("❌ Помилка ({})", code),
        Err(_) => "❌ Помилка мережі. Перевірте з'єднання.".to_string(),
    }
}
