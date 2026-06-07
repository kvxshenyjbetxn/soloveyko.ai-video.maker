use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// ─── Допоміжні функції ───────────────────────────────────────────────────────

/// Конвертує секунди у мікросекунди (одиниця часу CapCut).
fn secs_to_us(secs: f64) -> i64 {
    (secs * 1_000_000.0).round() as i64
}

/// Генерує унікальний UUID для CapCut на основі seed + лічильника.
fn gen_uuid(seed: u64, n: usize) -> String {
    let x = seed.wrapping_add(n as u64).wrapping_mul(0x9E3779B97F4A7C15);
    let x = x ^ (x >> 30);
    let x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    let x = x ^ (x >> 27);
    let x = x.wrapping_mul(0x94d049bb133111eb);
    let x = x ^ (x >> 31);
    format!("{:08X}-{:04X}-4{:03X}-{:04X}-{:012X}",
        (x >> 32) as u32,
        (x >> 16) as u16,
        (x >> 4) as u16 & 0x0FFF,
        ((x >> 48) as u16 & 0x3FFF) | 0x8000,
        x & 0x0000_FFFF_FFFF_FFFF,
    )
}

/// Прямі слеші — для draft_fold_path (CapCut так зберігає).
fn forward_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Нативні слеші ОС — для draft_root_path (CapCut використовує зворотні на Windows).
fn native_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Літера диска без двокрапки: "E:\\..." → "E". На macOS завжди порожній рядок.
fn drive_letter(path: &Path) -> String {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy();
        if s.len() >= 2 && s.as_bytes()[1] == b':' {
            return s[..1].to_uppercase();
        }
    }
    let _ = path;
    String::new()
}

/// Зчитує розміри зображення (jpg/png/webp).
fn image_dims(path: &Path) -> (u32, u32) {
    image::image_dimensions(path).unwrap_or((1920, 1080))
}

/// Зчитує розміри та тривалість відео/аудіо через ffprobe.
fn probe_media(path: &Path) -> (u32, u32, f64) {
    let ffprobe = crate::bundle::ffprobe_path();
    let out = std::process::Command::new(&ffprobe)
        .args(["-v", "quiet", "-print_format", "json", "-show_streams", "-show_format"])
        .arg(path)
        .output();

    if let Ok(output) = out {
        if let Ok(text) = std::str::from_utf8(&output.stdout) {
            if let Ok(v) = serde_json::from_str::<Value>(text) {
                let mut w = 0u32;
                let mut h = 0u32;
                if let Some(streams) = v["streams"].as_array() {
                    for s in streams {
                        if s["codec_type"].as_str() == Some("video") {
                            w = s["width"].as_u64().unwrap_or(1280) as u32;
                            h = s["height"].as_u64().unwrap_or(720) as u32;
                            break;
                        }
                    }
                }
                let dur = v["format"]["duration"].as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0);
                return (w, h, dur);
            }
        }
    }
    (0, 0, 0.0)
}

/// Тип медіафайлу.
#[derive(Clone, PartialEq)]
enum MediaKind { Photo, Video }

/// Дані про один медіафайл у пулі.
struct MediaInfo {
    mat_id: String,
    pool_id: String,
    path: PathBuf,
    kind: MediaKind,
    width: u32,
    height: u32,
    duration_us: i64,
    create_time: i64,
}

// ─── Головна функція ─────────────────────────────────────────────────────────

pub fn generate_capcut_project(
    save_dir: &Path,
    draft_root: &Path,
    project_name: &str,
    audio_duration_hint: Option<f64>,
    log_fn: impl Fn(&str),
) -> Result<(), String> {

    // ─── 1. Зчитуємо timeline.json ──────────────────────────────────────────
    #[derive(serde::Deserialize)]
    struct SegTiming {
        start_secs: f64,
        end_secs: f64,
        media: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct OverlaySegTiming {
        start_secs: f64,
        end_secs: f64,
        media: Option<String>,
        #[serde(default = "default_one")]
        scale: f64,
        #[serde(default)]
        pos_x: f64,
        #[serde(default)]
        pos_y: f64,
    }
    fn default_one() -> f64 { 1.0 }
    #[derive(serde::Deserialize)]
    struct OverlayTrackData {
        track_idx: usize,
        segments: Vec<OverlaySegTiming>,
    }
    #[derive(serde::Deserialize)]
    struct TimelineData {
        total_duration_secs: f64,
        #[serde(default)]
        audio_start_secs: f64,
        segments: Vec<SegTiming>,
        #[serde(default)]
        overlay_tracks: Vec<OverlayTrackData>,
    }

    let tl_text = std::fs::read_to_string(save_dir.join("timeline.json"))
        .map_err(|e| format!("Не вдалося прочитати timeline.json: {}", e))?;
    let tl: TimelineData = serde_json::from_str(&tl_text)
        .map_err(|e| format!("Не вдалося розпарсити timeline.json: {}", e))?;

    log_fn(&format!("CapCut: завантажено {} сегментів із timeline.json", tl.segments.len()));

    // ─── 2. Знаходимо аудіофайл ─────────────────────────────────────────────
    let voice_path: Option<PathBuf> = ["voice.wav", "voice.mp3"]
        .iter()
        .map(|n| save_dir.join(n))
        .find(|p| p.exists());

    // project_dir визначаємо рано — потрібно для копіювання медіа на macOS
    let project_dir = draft_root.join(project_name);

    // ─── 3. UUID-генератор ───────────────────────────────────────────────────
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut n = 0usize;
    let mut uid = || { let id = gen_uuid(seed, n); n += 1; id };

    // ─── 4. Збираємо унікальні медіафайли ───────────────────────────────────
    let mut media_map: std::collections::HashMap<String, usize> = Default::default();
    let mut media_list: Vec<MediaInfo> = Vec::new();

    // Збираємо всі медіа-шляхи з основної доріжки та overlay-доріжок
    let all_media_rels: Vec<String> = tl.segments.iter()
        .filter_map(|s| s.media.clone())
        .chain(
            tl.overlay_tracks.iter()
                .flat_map(|t| t.segments.iter())
                .filter_map(|s| s.media.clone())
        )
        .filter(|m| !m.is_empty())
        .collect();

    for rel in all_media_rels {
        if media_map.contains_key(&rel) { continue; }

        let abs_path = save_dir.join(&rel);
        let ext = abs_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let (kind, width, height, duration_us) = match ext.as_str() {
            "jpg" | "jpeg" | "png" | "webp" => {
                let (w, h) = image_dims(&abs_path);
                (MediaKind::Photo, w, h, 10_800_000_000i64)
            }
            "mp4" | "mov" | "avi" | "mkv" => {
                let (w, h, dur) = probe_media(&abs_path);
                let w = if w == 0 { 1280 } else { w };
                let h = if h == 0 { 720 } else { h };
                (MediaKind::Video, w, h, secs_to_us(dur))
            }
            _ => (MediaKind::Photo, 1920, 1080, 10_800_000_000i64),
        };

        let create_time = abs_path.metadata().ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        media_map.insert(rel, media_list.len());
        media_list.push(MediaInfo {
            mat_id: uid(),
            pool_id: uid(),
            path: abs_path,
            kind,
            width,
            height,
            duration_us,
            create_time,
        });
    }
    log_fn(&format!("CapCut: {} унікальних медіафайлів", media_list.len()));

    // ─── 4.5. macOS: CapCut sandboxed — копіюємо медіа у папку проекту ──────
    // На macOS CapCut не може читати файли поза ~/Movies/, тому копіюємо всі
    // медіафайли та аудіо прямо в project_dir/resources/ де CapCut має доступ.
    #[cfg(target_os = "macos")]
    {
        let res_dir = project_dir.join("resources");
        std::fs::create_dir_all(&res_dir)
            .map_err(|e| format!("Не вдалося створити resources/: {}", e))?;

        for m in &mut media_list {
            if m.path.exists() {
                let fname = m.path.file_name().unwrap_or_default().to_os_string();
                let dst = res_dir.join(&fname);
                std::fs::copy(&m.path, &dst)
                    .map_err(|e| format!("Копіювання {}: {}", fname.to_string_lossy(), e))?;
                m.path = dst;
            }
        }
        if let Some(ref vp) = voice_path.clone() {
            if vp.exists() {
                let fname = vp.file_name().unwrap_or_default().to_os_string();
                let dst = res_dir.join(&fname);
                std::fs::copy(vp, &dst)
                    .map_err(|e| format!("Копіювання voice: {}", e))?;
                voice_path = Some(dst);
            }
        }
        log_fn("CapCut: медіафайли скопійовано в папку проекту (macOS sandbox)");
    }

    // ─── 5. Тривалість аудіо ────────────────────────────────────────────────
    let audio_dur_secs = audio_duration_hint
        .filter(|&d| d > 0.0)
        .unwrap_or_else(|| {
            voice_path.as_ref()
                .map(|p| probe_media(p).2)
                .filter(|&d| d > 0.0)
                .unwrap_or(tl.total_duration_secs)
        });
    let audio_dur_us = secs_to_us(audio_dur_secs);

    // ─── 6. UUID-и для аудіо та проекту ─────────────────────────────────────
    let audio_mat_id   = uid();
    let audio_pool_id  = uid();
    let audio_music_id = uid();
    let timeline_id    = uid(); // ID таймлайну (ім'я папки Timelines/{id}/)
    let project_id     = uid(); // ID запису в Timelines/project.json
    let video_track_id = uid();
    let audio_track_id = uid();
    let project_uuid   = uid(); // draft_id у draft_meta_info

    // ─── 7. Часові мітки ────────────────────────────────────────────────────
    let now_us   = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as i64;
    let now_secs = now_us / 1_000_000;

    // ─── 8. Матеріали відео/фото ─────────────────────────────────────────────
    let mut mat_videos:       Vec<Value> = Vec::new();
    let mut mat_audios:       Vec<Value> = Vec::new();
    let mut mat_speeds:       Vec<Value> = Vec::new();
    let mut mat_canvases:     Vec<Value> = Vec::new();
    let mut mat_sound_maps:   Vec<Value> = Vec::new();
    let mut mat_colors:       Vec<Value> = Vec::new();
    let mut mat_vocal_seps:   Vec<Value> = Vec::new();
    let mut mat_ph_infos:     Vec<Value> = Vec::new();
    let mut mat_beats:        Vec<Value> = Vec::new();

    for m in &media_list {
        let type_str = if m.kind == MediaKind::Photo { "photo" } else { "video" };
        let fname = m.path.file_name()
            .and_then(|n| n.to_str()).unwrap_or("file").to_string();
        let local_mat_id = if m.kind == MediaKind::Video { m.pool_id.clone() } else { String::new() };

        let crop = json!({
            "upper_left_x": 0.0, "upper_left_y": 0.0,
            "upper_right_x": 1.0, "upper_right_y": 0.0,
            "lower_left_x": 0.0, "lower_left_y": 1.0,
            "lower_right_x": 1.0, "lower_right_y": 1.0
        });
        let mut mat_entry = serde_json::Map::new();
        mat_entry.insert("id".into(), json!(m.mat_id));
        mat_entry.insert("unique_id".into(), json!(""));
        mat_entry.insert("type".into(), json!(type_str));
        mat_entry.insert("path".into(), json!(forward_path(&m.path)));
        mat_entry.insert("media_path".into(), json!(""));
        mat_entry.insert("local_id".into(), json!(""));
        mat_entry.insert("has_audio".into(), json!(m.kind == MediaKind::Video));
        mat_entry.insert("reverse_path".into(), json!(""));
        mat_entry.insert("intensifies_path".into(), json!(""));
        mat_entry.insert("reverse_intensifies_path".into(), json!(""));
        mat_entry.insert("intensifies_audio_path".into(), json!(""));
        mat_entry.insert("cartoon_path".into(), json!(""));
        mat_entry.insert("width".into(), json!(m.width));
        mat_entry.insert("height".into(), json!(m.height));
        mat_entry.insert("duration".into(), json!(m.duration_us));
        mat_entry.insert("category_id".into(), json!(""));
        mat_entry.insert("category_name".into(), json!("local"));
        mat_entry.insert("material_id".into(), json!(""));
        mat_entry.insert("material_name".into(), json!(fname));
        mat_entry.insert("material_url".into(), json!(""));
        mat_entry.insert("crop".into(), crop);
        mat_entry.insert("crop_ratio".into(), json!("free"));
        mat_entry.insert("audio_fade".into(), Value::Null);
        mat_entry.insert("crop_scale".into(), json!(1.0));
        mat_entry.insert("extra_type_option".into(), json!(0));
        mat_entry.insert("stable".into(), Value::Null);
        mat_entry.insert("matting".into(), Value::Null);
        mat_entry.insert("source".into(), json!(0));
        mat_entry.insert("source_platform".into(), json!(0));
        mat_entry.insert("formula_id".into(), json!(""));
        mat_entry.insert("check_flag".into(), json!(62978047));
        mat_entry.insert("picture_from".into(), json!("none"));
        mat_entry.insert("picture_set_category_id".into(), json!(""));
        mat_entry.insert("picture_set_category_name".into(), json!(""));
        mat_entry.insert("local_material_id".into(), json!(local_mat_id));
        mat_entry.insert("origin_material_id".into(), json!(""));
        mat_entry.insert("request_id".into(), json!(""));
        mat_entry.insert("has_sound_separated".into(), json!(false));
        mat_entry.insert("is_text_edit_overdub".into(), json!(false));
        mat_entry.insert("is_ai_generate_content".into(), json!(false));
        mat_entry.insert("aigc_type".into(), json!("none"));
        mat_entry.insert("is_copyright".into(), json!(false));
        mat_entry.insert("aigc_history_id".into(), json!(""));
        mat_entry.insert("aigc_item_id".into(), json!(""));
        mat_entry.insert("local_material_from".into(), json!(""));
        mat_entry.insert("live_photo_timestamp".into(), json!(-1));
        mat_entry.insert("live_photo_cover_path".into(), json!(""));
        mat_entry.insert("content_feature_info".into(), Value::Null);
        mat_videos.push(Value::Object(mat_entry));
    }

    // ─── 9. Відеосегменти ────────────────────────────────────────────────────
    let mut video_segments: Vec<Value> = Vec::new();

    for seg in &tl.segments {
        let rel = match &seg.media {
            Some(m) if !m.is_empty() => m,
            _ => continue,
        };
        let mat = match media_map.get(rel).and_then(|&i| media_list.get(i)) {
            Some(m) => m,
            None => continue,
        };

        let seg_id       = uid();
        let speed_id     = uid();
        let ph_id        = uid();
        let canvas_id    = uid();
        let sound_map_id = uid();
        let color_id     = uid();
        let vocal_id     = uid();

        let t_start = secs_to_us(seg.start_secs);
        let t_dur   = secs_to_us(seg.end_secs - seg.start_secs);

        mat_speeds.push(json!({ "id": speed_id, "type": "speed", "mode": 0, "speed": 1.0, "curve_speed": null }));
        mat_ph_infos.push(json!({ "id": ph_id, "type": "placeholder_info", "meta_type": "none", "res_path": "", "res_text": "", "error_path": "", "error_text": "" }));
        mat_canvases.push(json!({ "id": canvas_id, "type": "canvas_color", "color": "", "blur": 0.0, "image": "", "album_image": "", "image_id": "", "image_name": "", "source_platform": 0, "team_id": "" }));
        mat_sound_maps.push(json!({ "id": sound_map_id, "type": "", "audio_channel_mapping": 0, "is_config_open": false }));
        mat_colors.push(json!({ "id": color_id, "is_color_clip": false, "is_gradient": false, "solid_color": "", "gradient_colors": [], "gradient_percents": [], "gradient_angle": 90, "width": 0, "height": 0 }));
        mat_vocal_seps.push(json!({ "id": vocal_id, "type": "vocal_separation", "choice": 0, "removed_sounds": [], "time_range": null, "production_path": "", "final_algorithm": "", "enter_from": "" }));

        video_segments.push(json!({
            "id": seg_id,
            "source_timerange": { "start": 0, "duration": t_dur },
            "target_timerange": { "start": t_start, "duration": t_dur },
            "render_timerange": { "start": 0, "duration": 0 },
            "desc": "",
            "state": 0,
            "speed": 1.0,
            "is_loop": false,
            "is_tone_modify": false,
            "reverse": false,
            "intensifies_audio": false,
            "cartoon": false,
            "volume": 1.0,
            "last_nonzero_volume": 1.0,
            "clip": {
                "scale": { "x": 1.0, "y": 1.0 },
                "rotation": 0.0,
                "transform": { "x": 0.0, "y": 0.0 },
                "flip": { "vertical": false, "horizontal": false },
                "alpha": 1.0
            },
            "uniform_scale": { "on": true, "value": 1.0 },
            "material_id": mat.mat_id,
            "extra_material_refs": [speed_id, ph_id, canvas_id, sound_map_id, color_id, vocal_id],
            "render_index": 0,
            "track_render_index": 0,
            "track_attribute": 0,
            "keyframe_refs": [],
            "common_keyframes": [],
            "enable_lut": true,
            "enable_adjust": true,
            "enable_hsl": false,
            "enable_color_curves": true,
            "enable_hsl_curves": true,
            "enable_color_wheels": true,
            "enable_smart_color_adjust": false,
            "enable_color_match_adjust": false,
            "enable_color_correct_adjust": false,
            "enable_adjust_mask": false,
            "enable_video_mask": true,
            "enable_mask_stroke": false,
            "enable_mask_shadow": false,
            "enable_color_adjust_pro": false,
            "visible": true,
            "group_id": "",
            "template_id": "",
            "template_scene": "default",
            "is_placeholder": false,
            "caption_info": null,
            "lyric_keyframes": null,
            "hdr_settings": { "mode": 1, "intensity": 1.0, "nits": 1000 },
            "responsive_layout": {
                "enable": false,
                "target_follow": "",
                "size_layout": 0,
                "horizontal_pos_layout": 0,
                "vertical_pos_layout": 0
            },
            "source": "segmentsourcenormal",
            "raw_segment_id": "",
            "digital_human_template_group_id": "",
            "color_correct_alg_result": ""
        }));
    }

    // ─── 10. Аудіосегмент ───────────────────────────────────────────────────
    let a_seg_id    = uid();
    let a_speed_id  = uid();
    let a_ph_id     = uid();
    let a_beats_id  = uid();
    let a_sound_id  = uid();
    let a_vocal_id  = uid();

    let audio_target_start = secs_to_us(tl.audio_start_secs);

    mat_speeds.push(json!({ "id": a_speed_id, "type": "speed", "mode": 0, "speed": 1.0, "curve_speed": null }));
    mat_ph_infos.push(json!({ "id": a_ph_id, "type": "placeholder_info", "meta_type": "none", "res_path": "", "res_text": "", "error_path": "", "error_text": "" }));
    mat_beats.push(json!({ "id": a_beats_id, "type": "beats", "enable_ai_beats": false, "gear": 404, "gear_count": 0, "mode": 404, "user_beats": [], "user_delete_ai_beats": null, "ai_beats": null }));
    mat_sound_maps.push(json!({ "id": a_sound_id, "type": "", "audio_channel_mapping": 0, "is_config_open": false }));
    mat_vocal_seps.push(json!({ "id": a_vocal_id, "type": "vocal_separation", "choice": 0, "removed_sounds": [], "time_range": null, "production_path": "", "final_algorithm": "", "enter_from": "" }));

    let voice_path_str = voice_path.as_deref().map(forward_path).unwrap_or_default();
    let voice_name = voice_path.as_deref()
        .and_then(|p| p.file_name()).and_then(|n| n.to_str())
        .unwrap_or("voice.mp3").to_string();
    let voice_ctime = voice_path.as_deref()
        .and_then(|p| p.metadata().ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(now_secs);

    mat_audios.push(json!({
        "id": audio_mat_id,
        "unique_id": "",
        "type": "extract_music",
        "name": voice_name,
        "path": voice_path_str,
        "duration": audio_dur_us,
        "category_id": "",
        "category_name": "local",
        "local_material_id": audio_pool_id,
        "source_platform": 0,
        "wave_points": [],
        "music_id": audio_music_id,
        "app_id": 0,
        "text_id": "",
        "tone_type": null,
        "effect_id": "",
        "resource_id": "",
        "third_resource_id": "",
        "intensifies_path": "",
        "formula_id": "",
        "check_flag": 1,
        "team_id": "",
        "lyric_type": 0,
        "is_ugc": false,
        "is_ai_clone_tone": false,
        "source_from": "",
        "copyright_limit_type": "none"
    }));

    let audio_segment = json!({
        "id": a_seg_id,
        "source_timerange": { "start": 0, "duration": audio_dur_us },
        "target_timerange": { "start": audio_target_start, "duration": audio_dur_us },
        "render_timerange": { "start": 0, "duration": 0 },
        "desc": "",
        "state": 0,
        "speed": 1.0,
        "is_loop": false,
        "reverse": false,
        "volume": 1.0,
        "last_nonzero_volume": 1.0,
        "clip": null,
        "material_id": audio_mat_id,
        "extra_material_refs": [a_speed_id, a_ph_id, a_beats_id, a_sound_id, a_vocal_id],
        "render_index": 0,
        "track_render_index": 1,
        "keyframe_refs": [],
        "common_keyframes": [],
        "visible": true,
        "group_id": "",
        "template_id": "",
        "is_placeholder": false,
        "caption_info": null,
        "lyric_keyframes": null,
        "source": "segmentsourcenormal",
        "raw_segment_id": ""
    });

    // ─── 11. Загальна тривалість ─────────────────────────────────────────────
    let total_dur_us = secs_to_us(tl.total_duration_secs)
        .max(audio_target_start + audio_dur_us);

    // ─── 12. Overlay-треки (доріжки 1+) ─────────────────────────────────────
    // pos_x/pos_y у редакторі: 0 = центр, ±1 = край canvas.
    // CapCut transform.x/y — піксельний зсув від центру (1920×1080 → ±960, ±540).
    const CANVAS_HALF_W: f64 = 960.0;
    const CANVAS_HALF_H: f64 = 540.0;

    let mut overlay_capcut_tracks: Vec<Value> = Vec::new();
    for ot in &tl.overlay_tracks {
        let mut ov_segments: Vec<Value> = Vec::new();
        for seg in &ot.segments {
            let rel = match &seg.media {
                Some(m) if !m.is_empty() => m,
                _ => continue,
            };
            let mat = match media_map.get(rel).and_then(|&i| media_list.get(i)) {
                Some(m) => m,
                None => continue,
            };

            let seg_id       = uid();
            let speed_id     = uid();
            let ph_id        = uid();
            let canvas_id    = uid();
            let sound_map_id = uid();
            let color_id     = uid();
            let vocal_id     = uid();

            let t_start = secs_to_us(seg.start_secs);
            let t_dur   = secs_to_us(seg.end_secs - seg.start_secs);
            let tx = seg.pos_x * CANVAS_HALF_W;
            let ty = seg.pos_y * CANVAS_HALF_H;

            mat_speeds.push(json!({ "id": speed_id, "type": "speed", "mode": 0, "speed": 1.0, "curve_speed": null }));
            mat_ph_infos.push(json!({ "id": ph_id, "type": "placeholder_info", "meta_type": "none", "res_path": "", "res_text": "", "error_path": "", "error_text": "" }));
            mat_canvases.push(json!({ "id": canvas_id, "type": "canvas_color", "color": "", "blur": 0.0, "image": "", "album_image": "", "image_id": "", "image_name": "", "source_platform": 0, "team_id": "" }));
            mat_sound_maps.push(json!({ "id": sound_map_id, "type": "", "audio_channel_mapping": 0, "is_config_open": false }));
            mat_colors.push(json!({ "id": color_id, "is_color_clip": false, "is_gradient": false, "solid_color": "", "gradient_colors": [], "gradient_percents": [], "gradient_angle": 90, "width": 0, "height": 0 }));
            mat_vocal_seps.push(json!({ "id": vocal_id, "type": "vocal_separation", "choice": 0, "removed_sounds": [], "time_range": null, "production_path": "", "final_algorithm": "", "enter_from": "" }));

            ov_segments.push(json!({
                "id": seg_id,
                "source_timerange": { "start": 0, "duration": t_dur },
                "target_timerange": { "start": t_start, "duration": t_dur },
                "render_timerange": { "start": 0, "duration": 0 },
                "desc": "",
                "state": 0,
                "speed": 1.0,
                "is_loop": false,
                "is_tone_modify": false,
                "reverse": false,
                "intensifies_audio": false,
                "cartoon": false,
                "volume": 1.0,
                "last_nonzero_volume": 1.0,
                "clip": {
                    "scale": { "x": seg.scale, "y": seg.scale },
                    "rotation": 0.0,
                    "transform": { "x": tx, "y": ty },
                    "flip": { "vertical": false, "horizontal": false },
                    "alpha": 1.0
                },
                "uniform_scale": { "on": true, "value": seg.scale },
                "material_id": mat.mat_id,
                "extra_material_refs": [speed_id, ph_id, canvas_id, sound_map_id, color_id, vocal_id],
                "render_index": 0,
                "track_render_index": ot.track_idx,
                "track_attribute": 0,
                "keyframe_refs": [],
                "common_keyframes": [],
                "enable_lut": true,
                "enable_adjust": true,
                "enable_hsl": false,
                "enable_color_curves": true,
                "enable_hsl_curves": true,
                "enable_color_wheels": true,
                "enable_smart_color_adjust": false,
                "enable_color_match_adjust": false,
                "enable_color_correct_adjust": false,
                "enable_adjust_mask": false,
                "enable_video_mask": true,
                "enable_mask_stroke": false,
                "enable_mask_shadow": false,
                "enable_color_adjust_pro": false,
                "visible": true,
                "group_id": "",
                "template_id": "",
                "template_scene": "default",
                "is_placeholder": false,
                "caption_info": null,
                "lyric_keyframes": null,
                "hdr_settings": { "mode": 1, "intensity": 1.0, "nits": 1000 },
                "responsive_layout": {
                    "enable": false,
                    "target_follow": "",
                    "size_layout": 0,
                    "horizontal_pos_layout": 0,
                    "vertical_pos_layout": 0
                },
                "source": "segmentsourcenormal",
                "raw_segment_id": "",
                "digital_human_template_group_id": "",
                "color_correct_alg_result": ""
            }));
        }
        if !ov_segments.is_empty() {
            let track_id = uid();
            overlay_capcut_tracks.push(json!({
                "id": track_id,
                "type": "video",
                "flag": 0,
                "attribute": 0,
                "name": "",
                "is_default_name": true,
                "segments": ov_segments
            }));
        }
    }

    // ─── 13. Всі треки ───────────────────────────────────────────────────────
    let mut tracks = vec![json!({
        "id": video_track_id,
        "type": "video",
        "flag": 0,
        "attribute": 0,
        "name": "",
        "is_default_name": true,
        "segments": video_segments
    })];
    // Overlay-треки йдуть після основного відеотреку
    tracks.extend(overlay_capcut_tracks);
    if voice_path.is_some() {
        tracks.push(json!({
            "id": audio_track_id,
            "type": "audio",
            "flag": 0,
            "attribute": 0,
            "name": "",
            "is_default_name": true,
            "segments": [audio_segment]
        }));
    }

    // Назва ОС для блоку platform у draft_content.json
    let os_name = if cfg!(target_os = "macos") { "mac" } else { "windows" };

    // ─── 14. draft_content.json ──────────────────────────────────────────────
    let draft_content = json!({
        "id": timeline_id,
        "version": 360000,
        "new_version": "171.0.0",
        "name": "",
        "duration": total_dur_us,
        "create_time": 0,
        "update_time": 0,
        "fps": 30.0,
        "is_drop_frame_timecode": false,
        "color_space": 0,
        "canvas_config": {
            "ratio": "original",
            "width": 1920,
            "height": 1080,
            "background": null
        },
        "config": {
            "video_mute": false,
            "record_audio_last_index": 1,
            "extract_audio_last_index": 1,
            "original_sound_last_index": 1,
            "subtitle_recognition_id": "",
            "subtitle_taskinfo": [],
            "lyrics_recognition_id": "",
            "lyrics_taskinfo": [],
            "subtitle_sync": true,
            "lyrics_sync": true,
            "voice_change_sync": false,
            "sticker_max_index": 1,
            "adjust_max_index": 1,
            "material_save_mode": 0,
            "export_range": null,
            "maintrack_adsorb": true,
            "combination_max_index": 1,
            "attachment_info": [],
            "zoom_info_params": null,
            "system_font_list": [],
            "multi_language_mode": "none",
            "multi_language_main": "none",
            "multi_language_current": "none",
            "multi_language_list": [],
            "subtitle_keywords_config": null,
            "use_float_render": false
        },
        "tracks": tracks,
        "materials": {
            "videos": mat_videos,
            "audios": mat_audios,
            "speeds": mat_speeds,
            "canvases": mat_canvases,
            "sound_channel_mappings": mat_sound_maps,
            "material_colors": mat_colors,
            "vocal_separations": mat_vocal_seps,
            "placeholder_infos": mat_ph_infos,
            "beats": mat_beats,
            "texts": [],
            "stickers": [],
            "transitions": [],
            "filters": [],
            "effects": [],
            "audio_effects": [],
            "audio_fades": [],
            "audio_pannings": [],
            "audio_pitch_shifts": [],
            "audio_track_indexes": [],
            "material_animations": [],
            "flowers": [],
            "images": [],
            "tail_leaders": [],
            "text_templates": [],
            "common_mask": [],
            "chromas": [],
            "green_screens": [],
            "shapes": [],
            "video_effects": [],
            "video_trackings": [],
            "hsl": [],
            "color_curves": [],
            "hsl_curves": [],
            "primary_color_wheels": [],
            "log_color_wheels": [],
            "loudnesses": [],
            "realtime_denoises": [],
            "smart_crops": [],
            "ai_translates": [],
            "vocal_beautifys": [],
            "time_marks": [],
            "video_shadows": [],
            "video_strokes": [],
            "video_radius": [],
            "multi_language_refs": [],
            "manual_deformations": [],
            "manual_beautys": [],
            "plugin_effects": [],
            "digital_humans": [],
            "digital_human_model_dressing": [],
            "smart_relights": [],
            "drafts": [],
            "audio_balances": [],
            "handwrites": []
        },
        "keyframes": {
            "videos": [],
            "audios": [],
            "texts": [],
            "stickers": [],
            "filters": [],
            "adjusts": [],
            "handwrites": [],
            "effects": []
        },
        "keyframe_graph_list": [],
        "relationships": [],
        "render_index_track_mode_on": true,
        "free_render_index_mode_on": false,
        "static_cover_image_path": "",
        "source": "default",
        "time_marks": null,
        "path": "",
        "lyrics_effects": [],
        "draft_type": "video",
        "mutable_config": null,
        "cover": null,
        "retouch_cover": null,
        "extra_info": null,
        "group_container": null,
        "platform": {
            "os": os_name,
            "app_id": 359289,
            "app_version": "8.7.0",
            "app_source": "cc",
            "device_id": "",
            "hard_disk_id": "",
            "mac_address": ""
        },
        "last_modified_platform": {
            "os": os_name,
            "app_id": 359289,
            "app_version": "8.7.0",
            "app_source": "cc",
            "device_id": "",
            "hard_disk_id": "",
            "mac_address": ""
        },
        "smart_ads_info": { "page_from": "", "routine": "", "draft_url": "" },
        "function_assistant_info": {
            "smart_rec_applied": false,
            "fixed_rec_applied": false,
            "auto_adjust": false,
            "auto_adjust_segid_list": [],
            "color_correction": false,
            "color_correction_segid_list": [],
            "enhance_quality": false,
            "smooth_slow_motion": false,
            "deflicker_segid_list": [],
            "video_noise_segid_list": [],
            "enhance_quality_segid_list": [],
            "smart_segid_list": [],
            "retouch": false,
            "retouch_segid_list": [],
            "enhande_voice": false,
            "enhance_voice_segid_list": [],
            "audio_noise_segid_list": [],
            "auto_caption": false,
            "auto_caption_segid_list": [],
            "auto_caption_template_id": "",
            "caption_opt": false,
            "caption_opt_segid_list": [],
            "eye_correction": false,
            "eye_correction_segid_list": [],
            "normalize_loudness": false,
            "normalize_loudness_segid_list": [],
            "normalize_loudness_audio_denoise_segid_list": [],
            "auto_adjust_fixed": false,
            "auto_adjust_fixed_value": 50.0,
            "color_correction_fixed": false,
            "color_correction_fixed_value": 50.0,
            "normalize_loudness_fixed": false,
            "enhande_voice_fixed": false,
            "retouch_fixed": false,
            "enhance_quality_fixed": false,
            "smooth_slow_motion_fixed": false,
            "fps": { "num": 0, "den": 1 }
        },
        "uneven_animation_template_info": {
            "composition": "",
            "content": "",
            "order": "",
            "sub_template_info_list": []
        }
    });

    // ─── 15. Шляхи до папки проекту ─────────────────────────────────────────
    // (project_dir визначено вище, на початку функції)

    // draft_root_path — нативні слеші (backslash на Windows, forward slash на macOS)
    let root_path_native = native_path(draft_root);
    // removable_storage_device актуальний лише на Windows (для не-C: дисків)
    let drv = drive_letter(draft_root);
    let removable_device = if !drv.is_empty() && drv.to_uppercase() != "C" {
        format!("{}:", drv)
    } else {
        String::new()
    };

    // ─── 16. Медіапул для draft_meta_info ────────────────────────────────────
    let mut pool: Vec<Value> = vec![json!({
        "ai_group_type": "",
        "create_time": now_secs,
        "duration": 33333,
        "enter_from": 0,
        "extra_info": "",
        "file_Path": "",
        "height": 0,
        "id": uid(),
        "import_time": now_secs,
        "import_time_ms": now_us,
        "item_source": 1,
        "md5": "",
        "metetype": "none",
        "roughcut_time_range": { "duration": 33333, "start": 0 },
        "sub_time_range": { "duration": -1, "start": -1 },
        "type": 0,
        "width": 0
    })];

    if voice_path.is_some() {
        pool.push(json!({
            "ai_group_type": "",
            "create_time": voice_ctime,
            "duration": audio_dur_us,
            "enter_from": 0,
            "extra_info": voice_name,
            "file_Path": voice_path_str,
            "height": 0,
            "id": audio_pool_id,
            "import_time": now_secs,
            "import_time_ms": now_us,
            "item_source": 1,
            "md5": "",
            "metetype": "music",
            "roughcut_time_range": { "duration": audio_dur_us, "start": 0 },
            "sub_time_range": { "duration": -1, "start": -1 },
            "type": 0,
            "width": 0
        }));
    }

    for m in &media_list {
        let metetype = if m.kind == MediaKind::Photo { "photo" } else { "video" };
        let roughcut = if m.kind == MediaKind::Photo {
            json!({ "duration": -1, "start": -1 })
        } else {
            json!({ "duration": m.duration_us, "start": 0 })
        };
        let pool_dur = if m.kind == MediaKind::Photo { secs_to_us(5.0) } else { m.duration_us };
        let fname = m.path.file_name()
            .and_then(|n| n.to_str()).unwrap_or("file").to_string();

        pool.push(json!({
            "ai_group_type": "",
            "create_time": m.create_time,
            "duration": pool_dur,
            "enter_from": 0,
            "extra_info": fname,
            "file_Path": forward_path(&m.path),
            "height": m.height,
            "id": m.pool_id,
            "import_time": now_secs,
            "import_time_ms": now_us,
            "item_source": 1,
            "md5": "",
            "metetype": metetype,
            "roughcut_time_range": roughcut,
            "sub_time_range": { "duration": -1, "start": -1 },
            "type": 0,
            "width": m.width
        }));
    }

    // ─── 17. draft_meta_info.json ────────────────────────────────────────────
    let draft_meta_info = json!({
        "cloud_draft_cover": false,
        "cloud_draft_sync": false,
        "cloud_package_completed_time": "",
        "draft_cloud_capcut_purchase_info": "",
        "draft_cloud_last_action_download": false,
        "draft_cloud_package_type": "",
        "draft_cloud_purchase_info": "",
        "draft_cloud_template_id": "",
        "draft_cloud_tutorial_info": "",
        "draft_cloud_videocut_purchase_info": "",
        "draft_cover": "draft_cover.jpg",
        "draft_deeplink_url": "",
        "draft_enterprise_info": {
            "draft_enterprise_extra": "",
            "draft_enterprise_id": "",
            "draft_enterprise_name": "",
            "enterprise_material": []
        },
        "draft_fold_path": forward_path(&project_dir),
        "draft_id": project_uuid,
        "draft_is_ae_produce": false,
        "draft_is_ai_packaging_used": false,
        "draft_is_ai_shorts": false,
        "draft_is_ai_translate": false,
        "draft_is_article_video_draft": false,
        "draft_is_cloud_temp_draft": false,
        "draft_is_from_deeplink": "false",
        "draft_is_invisible": false,
        "draft_is_pippit_draft": false,
        "draft_is_web_article_video": false,
        "draft_materials": [
            { "type": 0, "value": pool },
            { "type": 1, "value": [] },
            { "type": 2, "value": [] },
            { "type": 3, "value": [] },
            { "type": 6, "value": [] },
            { "type": 7, "value": [] },
            { "type": 8, "value": [] }
        ],
        "draft_materials_copied_info": [],
        "draft_name": project_name,
        "draft_need_rename_folder": false,
        "draft_new_version": "",
        "draft_removable_storage_device": removable_device,
        "draft_root_path": root_path_native,
        "draft_segment_extra_info": [],
        "draft_timeline_materials_size_": 0,
        "draft_type": "",
        "draft_web_article_video_enter_from": "",
        "tm_draft_cloud_completed": "",
        "tm_draft_cloud_entry_id": -1,
        "tm_draft_cloud_modified": 0,
        "tm_draft_cloud_parent_entry_id": -1,
        "tm_draft_cloud_space_id": -1,
        "tm_draft_cloud_user_id": -1,
        "tm_draft_create": now_us,
        "tm_draft_modified": now_us,
        "tm_draft_removed": 0,
        "tm_duration": total_dur_us
    });

    // ─── 18. Timelines/project.json ──────────────────────────────────────────
    let timelines_project = json!({
        "config": {
            "color_space": -1,
            "render_index_track_mode_on": false,
            "use_float_render": false
        },
        "create_time": now_us,
        "id": project_id,
        "main_timeline_id": timeline_id,
        "timelines": [{
            "id": timeline_id,
            "name": "Временная шкала 01",
            "create_time": now_us,
            "update_time": now_us,
            "is_marked_delete": false
        }],
        "update_time": now_us,
        "version": 0
    });

    // ─── 19. Записуємо файли ─────────────────────────────────────────────────
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Не вдалося створити папку проекту: {}", e))?;

    let tl_uuid_dir = project_dir.join("Timelines").join(&timeline_id);
    std::fs::create_dir_all(&tl_uuid_dir)
        .map_err(|e| format!("Не вдалося створити Timelines/: {}", e))?;

    let content_json = serde_json::to_string_pretty(&draft_content)
        .map_err(|e| format!("Помилка серіалізації draft_content: {}", e))?;

    // Windows шукає draft_content.json, macOS — draft_info.json; пишемо обидва
    std::fs::write(project_dir.join("draft_content.json"), &content_json)
        .map_err(|e| format!("draft_content.json: {}", e))?;
    std::fs::write(project_dir.join("draft_info.json"), &content_json)
        .map_err(|e| format!("draft_info.json: {}", e))?;

    std::fs::write(project_dir.join("draft_meta_info.json"),
        serde_json::to_string_pretty(&draft_meta_info)
            .map_err(|e| format!("Помилка серіалізації draft_meta_info: {}", e))?)
        .map_err(|e| format!("draft_meta_info.json: {}", e))?;

    std::fs::write(project_dir.join("Timelines").join("project.json"),
        serde_json::to_string_pretty(&timelines_project)
            .map_err(|e| format!("Помилка серіалізації Timelines/project.json: {}", e))?)
        .map_err(|e| format!("Timelines/project.json: {}", e))?;

    // Timelines/{uuid}/ — теж обидва файли для крос-платформеності
    std::fs::write(tl_uuid_dir.join("draft_content.json"), &content_json)
        .map_err(|e| format!("Timelines/uuid/draft_content.json: {}", e))?;
    std::fs::write(tl_uuid_dir.join("draft_info.json"), &content_json)
        .map_err(|e| format!("Timelines/uuid/draft_info.json: {}", e))?;

    // draft_settings — обов'язковий на macOS; без нього CapCut показує "невірний адрес"
    let draft_settings = format!(
        "[General]\ndraft_create_time={}\ndraft_last_edit_time={}\nreal_edit_keys=1\nreal_edit_seconds=0\n",
        now_secs, now_secs
    );
    std::fs::write(project_dir.join("draft_settings"), draft_settings)
        .map_err(|e| format!("draft_settings: {}", e))?;

    log_fn(&format!("CapCut: проект створено — {}", forward_path(&project_dir)));
    Ok(())
}
