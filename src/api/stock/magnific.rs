use super::{StockPhoto, StockProvider, StockVideo};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;

const ACCEPT_LANGUAGE: &str = "en-US";
const MAX_IMAGE_PAGES: u32 = 3;

pub struct MagnificProvider;

#[derive(Deserialize, Default)]
struct ResourceSearchResponse {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    data: Vec<ResourceHit>,
}

#[derive(Deserialize, Clone, Default)]
struct ResourceHit {
    id: u64,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    image: ResourceImage,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    author: MagnificAuthor,
}

impl ResourceHit {
    fn preview_url(&self) -> Option<&str> {
        let url = self.image.source.url.trim();
        (!url.is_empty()).then_some(url)
    }

    fn dimensions(&self) -> Option<(u32, u32)> {
        parse_size_dimensions(&self.image.source.size)
    }

    fn is_sixteen_by_nine(&self) -> bool {
        self.dimensions()
            .map(|(width, height)| aspect_matches_sixteen_by_nine(width, height))
            .unwrap_or(false)
    }

    fn area(&self) -> u64 {
        self.dimensions()
            .map(|(width, height)| width as u64 * height as u64)
            .unwrap_or(0)
    }
}

#[derive(Deserialize, Clone, Default)]
struct ResourceImage {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    source: ResourceSource,
}

#[derive(Deserialize, Clone, Default)]
struct ResourceSource {
    #[serde(default)]
    url: String,
    #[serde(default)]
    size: String,
}

#[derive(Deserialize, Clone, Default)]
struct MagnificAuthor {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize, Default)]
struct VideoSearchResponse {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    data: Vec<MagnificVideo>,
}

#[derive(Deserialize, Clone, Default)]
struct MagnificVideo {
    id: u64,
    #[serde(rename = "aspect-ratio", alias = "aspect_ratio", default)]
    aspect_ratio: String,
    #[serde(default)]
    quality: String,
    #[serde(default)]
    duration: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    author: MagnificAuthor,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    thumbnails: Vec<VideoAsset>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    previews: Vec<VideoAsset>,
}

impl MagnificVideo {
    fn thumb_url(&self) -> Option<&str> {
        self.thumbnails.iter().find_map(|asset| {
            let url = asset.url.trim();
            (!url.is_empty()).then_some(url)
        })
    }

    fn preview_url(&self) -> Option<&str> {
        self.previews.iter().find_map(|asset| {
            let url = asset.url.trim();
            (!url.is_empty()).then_some(url)
        })
    }

    fn best_asset(&self) -> Option<&VideoAsset> {
        self.previews
            .iter()
            .chain(self.thumbnails.iter())
            .filter(|asset| !asset.url.trim().is_empty())
            .max_by_key(|asset| asset.width as u64 * asset.height as u64)
    }

    fn is_sixteen_by_nine(&self) -> bool {
        if aspect_text_matches_sixteen_by_nine(&self.aspect_ratio) {
            return true;
        }

        self.thumbnails
            .iter()
            .chain(self.previews.iter())
            .any(|asset| aspect_matches_sixteen_by_nine(asset.width, asset.height))
    }

    fn quality_score(&self) -> u32 {
        parse_video_quality_score(&self.quality)
    }

    fn duration_secs(&self) -> u32 {
        parse_duration_secs(&self.duration)
    }

    fn area(&self) -> u64 {
        self.best_asset()
            .map(|asset| asset.width as u64 * asset.height as u64)
            .unwrap_or(0)
    }
}

#[derive(Deserialize, Clone, Default)]
struct VideoAsset {
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    url: String,
}

#[derive(Deserialize)]
struct DownloadResponse {
    data: DownloadData,
}

#[derive(Deserialize)]
struct DownloadData {
    url: String,
    #[serde(default)]
    signed_url: Option<String>,
}

impl StockProvider for MagnificProvider {
    fn name(&self) -> &str {
        "Magnific"
    }

    fn search_photos(
        &self,
        key: &str,
        query: &str,
        per_page: u32,
    ) -> Result<Vec<StockPhoto>, String> {
        let target = per_page.max(1) as usize;
        let mut seen_ids = HashSet::new();
        let mut hits = Vec::new();

        for page in 1..=MAX_IMAGE_PAGES {
            let response: ResourceSearchResponse = request_json(
                http_agent()
                    .get("https://api.magnific.com/v1/resources")
                    .set("x-magnific-api-key", key)
                    .set("Accept-Language", ACCEPT_LANGUAGE)
                    .query("term", query)
                    .query("page", &page.to_string())
                    .query("limit", &per_page.to_string())
                    .query("order", "relevance")
                    .query("filters[content_type][photo]", "1"),
            )?;

            let mut added_on_page = 0usize;
            for hit in response.data {
                if !hit.is_sixteen_by_nine() || !seen_ids.insert(hit.id) {
                    continue;
                }
                if hit.preview_url().is_none() {
                    continue;
                }
                hits.push(hit);
                added_on_page += 1;
            }

            if hits.len() >= target || added_on_page == 0 {
                break;
            }
        }

        hits.sort_by(|left, right| right.area().cmp(&left.area()));

        Ok(hits
            .into_iter()
            .take(target)
            .filter_map(|hit| {
                let preview_url = hit.preview_url()?.to_string();
                let (width, height) = hit.dimensions().unwrap_or((0, 0));
                Some(StockPhoto {
                    id: hit.id.to_string(),
                    preview_url: preview_url.clone(),
                    original_url: preview_url,
                    width,
                    height,
                    author: hit.author.name,
                })
            })
            .collect())
    }

    fn search_videos(
        &self,
        key: &str,
        query: &str,
        per_page: u32,
    ) -> Result<Vec<StockVideo>, String> {
        let response: VideoSearchResponse = request_json(
            http_agent()
                .get("https://api.magnific.com/v1/videos")
                .set("x-magnific-api-key", key)
                .set("Accept-Language", ACCEPT_LANGUAGE)
                .query("term", query)
                .query("page", "1")
                .query("order", "relevance"),
        )?;

        let mut hits: Vec<MagnificVideo> = response
            .data
            .into_iter()
            .filter(|hit| hit.is_sixteen_by_nine() && hit.thumb_url().is_some())
            .collect();

        hits.sort_by(|left, right| {
            right
                .quality_score()
                .cmp(&left.quality_score())
                .then_with(|| right.area().cmp(&left.area()))
        });

        Ok(hits
            .into_iter()
            .take(per_page.max(1) as usize)
            .filter_map(|hit| {
                let thumbnail_url = hit.thumb_url()?.to_string();
                let fallback_video_url = hit.preview_url().unwrap_or("").to_string();
                let best = hit.best_asset();
                Some(StockVideo {
                    id: hit.id.to_string(),
                    thumbnail_url,
                    duration_secs: hit.duration_secs(),
                    download_url: fallback_video_url,
                    width: best.map(|asset| asset.width).unwrap_or(0),
                    height: best.map(|asset| asset.height).unwrap_or(0),
                    author: hit.author.name,
                })
            })
            .collect())
    }
}

pub fn resolve_photo_download(key: &str, id: &str) -> Result<String, String> {
    let response: DownloadResponse = request_json(
        http_agent()
            .get(&format!(
                "https://api.magnific.com/v1/resources/{id}/download"
            ))
            .set("x-magnific-api-key", key)
            .set("Accept-Language", ACCEPT_LANGUAGE),
    )
    .map_err(friendly_magnific_error)?;

    let url = response
        .data
        .signed_url
        .filter(|url| !url.trim().is_empty())
        .unwrap_or(response.data.url);

    if url.trim().is_empty() {
        Err("Magnific повернув порожній download URL для ресурсу.".to_string())
    } else {
        Ok(url)
    }
}

pub fn resolve_video_download(key: &str, id: &str) -> Result<String, String> {
    let response: DownloadResponse = request_json(
        http_agent()
            .get(&format!("https://api.magnific.com/v1/videos/{id}/download"))
            .set("x-magnific-api-key", key)
            .set("Accept-Language", ACCEPT_LANGUAGE),
    )
    .map_err(friendly_magnific_error)?;

    let url = response
        .data
        .signed_url
        .filter(|url| !url.trim().is_empty())
        .unwrap_or(response.data.url);

    if url.trim().is_empty() {
        Err("Magnific повернув порожній download URL для відео.".to_string())
    } else {
        Ok(url)
    }
}

pub fn check_key(key: &str) -> String {
    match http_agent()
        .get("https://api.magnific.com/v1/resources")
        .set("x-magnific-api-key", key)
        .set("Accept-Language", ACCEPT_LANGUAGE)
        .query("term", "test")
        .query("page", "1")
        .query("limit", "1")
        .query("filters[content_type][photo]", "1")
        .call()
    {
        Ok(_) => "✔ Ключ валідний".to_string(),
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
            "❌ Невірний ключ".to_string()
        }
        Err(ureq::Error::Status(code, _)) if code >= 500 => {
            format!("⚠ Сервер тимчасово недоступний ({code})")
        }
        Err(ureq::Error::Status(code, _)) => format!("❌ Помилка ({code})"),
        Err(_) => "❌ Помилка мережі. Перевірте з'єднання.".to_string(),
    }
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(20))
        .build()
}

fn request_json<T: DeserializeOwned>(request: ureq::Request) -> Result<T, String> {
    match request.call() {
        Ok(response) => response.into_json::<T>().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            let trimmed = body.trim();
            let sniff = trimmed
                .chars()
                .take(64)
                .collect::<String>()
                .to_ascii_lowercase();

            if trimmed.is_empty() {
                Err(format!("HTTP {code}"))
            } else if sniff.starts_with("<!doctype html") || sniff.starts_with("<html") {
                Err(format!(
                    "HTTP {code}: Magnific повернув HTML error page замість JSON"
                ))
            } else {
                Err(format!("HTTP {code}: {trimmed}"))
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

fn is_server_error(message: &str) -> bool {
    message.starts_with("HTTP 500")
        || message.starts_with("HTTP 502")
        || message.starts_with("HTTP 503")
}

fn friendly_magnific_error(message: String) -> String {
    if is_server_error(&message) {
        "Magnific тимчасово віддає 5xx. Спробуй інший query або повтори пізніше.".to_string()
    } else {
        message
    }
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(|value| value.unwrap_or_default())
}

fn parse_size_dimensions(size: &str) -> Option<(u32, u32)> {
    let normalized = size.trim().replace('×', "x").replace('X', "x");
    let (width, height) = normalized.split_once('x')?;
    Some((width.trim().parse().ok()?, height.trim().parse().ok()?))
}

fn aspect_matches_sixteen_by_nine(width: u32, height: u32) -> bool {
    if width == 0 || height == 0 {
        return false;
    }

    let left = width as i64 * 9;
    let right = height as i64 * 16;
    (left - right).abs() * 100 <= right.max(1) * 3
}

fn aspect_text_matches_sixteen_by_nine(text: &str) -> bool {
    let normalized = text.trim().replace(' ', "");
    if normalized.is_empty() {
        return false;
    }

    if let Some((width, height)) = normalized
        .split_once(':')
        .or_else(|| normalized.split_once('/'))
    {
        if let (Ok(width), Ok(height)) = (width.parse::<u32>(), height.parse::<u32>()) {
            return aspect_matches_sixteen_by_nine(width, height);
        }
    }

    normalized
        .parse::<f32>()
        .map(|ratio| (ratio - (16.0 / 9.0)).abs() < 0.05)
        .unwrap_or(false)
}

fn parse_video_quality_score(quality: &str) -> u32 {
    quality
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

fn parse_duration_secs(text: &str) -> u32 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let parts: Vec<&str> = trimmed.split(':').collect();
    match parts.as_slice() {
        [seconds] => seconds
            .parse::<f32>()
            .ok()
            .map(|value| value.round() as u32)
            .unwrap_or(0),
        [minutes, seconds] => match (minutes.parse::<u32>(), seconds.parse::<f32>()) {
            (Ok(minutes), Ok(seconds)) => minutes * 60 + seconds.round() as u32,
            _ => 0,
        },
        [hours, minutes, seconds] => {
            match (
                hours.parse::<u32>(),
                minutes.parse::<u32>(),
                seconds.parse::<f32>(),
            ) {
                (Ok(hours), Ok(minutes), Ok(seconds)) => {
                    hours * 3600 + minutes * 60 + seconds.round() as u32
                }
                _ => 0,
            }
        }
        _ => 0,
    }
}
