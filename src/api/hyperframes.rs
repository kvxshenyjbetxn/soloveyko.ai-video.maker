use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
const NPX_COMMAND: &str = "npx.cmd";
#[cfg(not(windows))]
const NPX_COMMAND: &str = "npx";

fn new_npx_command() -> std::process::Command {
    crate::bundle::new_direct_cli_command(NPX_COMMAND)
}

/// Детерміновано збирає preview-проєкт із готових standalone HyperFrames-кліпів.
/// Агент не бере участі: скрипт переносить style, markup і inline script у sub-composition.
pub fn rebuild_preview_all(save_dir: &Path) -> Result<usize, String> {
    let segments_path = save_dir.join("segments.json");
    let content = std::fs::read_to_string(&segments_path)
        .map_err(|error| format!("Не вдалося прочитати segments.json: {}", error))?;
    let timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|error| format!("Невалідний segments.json: {}", error))?;

    let preview_dir = save_dir.join("preview-all");
    let compositions_dir = preview_dir.join("compositions");
    std::fs::create_dir_all(&compositions_dir)
        .map_err(|error| format!("Не вдалося створити preview-all: {}", error))?;

    let mut slots = Vec::new();
    let mut start_secs = 0.0;
    for (position, segment) in timeline.segments.iter().enumerate() {
        if segment.media_type
            != crate::core::pipeline::timeline::sync::SegmentMediaType::Hyperframes
        {
            continue;
        }
        let source_path = segment
            .media
            .as_deref()
            .filter(|media| media.to_ascii_lowercase().ends_with(".html"))
            .map(|media| save_dir.join(media))
            // Під час генерації segments.json ще не записано. Шлях детермінований
            // за індексом, тому preview доступний одразу після появи HTML-файлу.
            .unwrap_or_else(|| {
                save_dir.join(format!("clips/{:04}-scene/index.html", position + 1))
            });
        if !source_path.is_file() {
            continue;
        }
        let source = std::fs::read_to_string(&source_path).map_err(|error| {
            format!(
                "Не вдалося прочитати HyperFrames-кліп {}: {}",
                source_path.display(),
                error
            )
        })?;
        let composition_id = extract_composition_id(&source).ok_or_else(|| {
            format!(
                "HyperFrames-кліп {} не містить data-composition-id",
                source_path.display()
            )
        })?;
        let body = extract_tag_inner(&source, "body")
            .ok_or_else(|| format!("HyperFrames-кліп {} не містить body", source_path.display()))?;
        let styles = extract_tag_blocks(&source, "style");
        // Зовнішні runtime-скрипти зазвичай лежать у head standalone-кліпу.
        // Для sub-composition вони мусять бути всередині template разом з inline timeline.
        let runtime_scripts = extract_tag_blocks(&source, "script")
            .into_iter()
            .filter(|script| script.contains("src="))
            .collect::<Vec<_>>();
        let file_name = format!("{:04}-scene.html", position + 1);
        let composition = wrap_subcomposition(&runtime_scripts, &styles, body);
        std::fs::write(compositions_dir.join(&file_name), composition).map_err(|error| {
            format!(
                "Не вдалося записати preview composition {}: {}",
                file_name, error
            )
        })?;

        slots.push(PreviewSlot {
            composition_id,
            file_name,
            start_secs,
            duration_secs: segment.duration_secs.max(0.1),
        });
        start_secs += segment.duration_secs.max(0.1);
    }

    let index = build_preview_index(&slots, start_secs.max(0.1));
    std::fs::write(preview_dir.join("index.html"), index)
        .map_err(|error| format!("Не вдалося записати preview-all/index.html: {}", error))?;
    Ok(slots.len())
}

struct PreviewSlot {
    composition_id: String,
    file_name: String,
    start_secs: f64,
    duration_secs: f64,
}

fn extract_composition_id(html: &str) -> Option<String> {
    let marker = "data-composition-id";
    let after_marker = html.get(html.find(marker)? + marker.len()..)?.trim_start();
    let after_equals = after_marker.strip_prefix('=')?.trim_start();
    let quote = after_equals.chars().next()?;
    if quote != '\"' && quote != '\'' {
        return None;
    }
    let value = &after_equals[quote.len_utf8()..];
    Some(value.get(..value.find(quote)?)?.to_string())
}

fn extract_tag_inner<'a>(html: &'a str, tag: &str) -> Option<&'a str> {
    let open_start = html.find(&format!("<{}", tag))?;
    let after_open = html.get(open_start..)?;
    let content_start = open_start + after_open.find('>')? + 1;
    let close_start = html.get(content_start..)?.find(&format!("</{}>", tag))? + content_start;
    html.get(content_start..close_start)
}

fn extract_tag_blocks(html: &str, tag: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut offset = 0;
    let close = format!("</{}>", tag);
    while let Some(relative_open) = html
        .get(offset..)
        .and_then(|source| source.find(&format!("<{}", tag)))
    {
        let open_start = offset + relative_open;
        let Some(relative_end) = html
            .get(open_start..)
            .and_then(|source| source.find(&close))
        else {
            break;
        };
        let end = open_start + relative_end + close.len();
        if let Some(block) = html.get(open_start..end) {
            blocks.push(block.to_string());
        }
        offset = end;
    }
    blocks
}

fn wrap_subcomposition(runtime_scripts: &[String], styles: &[String], body: &str) -> String {
    format!(
        "<!doctype html>\n<html>\n<body>\n<template>\n{}\n{}\n{}\n</template>\n</body>\n</html>\n",
        runtime_scripts.join("\n"),
        styles.join("\n"),
        body.trim()
    )
}

fn build_preview_index(slots: &[PreviewSlot], duration_secs: f64) -> String {
    let slots_html = slots
        .iter()
        .enumerate()
        .map(|(index, slot)| {
            format!(
                "    <div id=\"scene-{index}\" class=\"clip\" data-composition-id=\"{}\" data-composition-src=\"compositions/{}\" data-start=\"{:.3}\" data-duration=\"{:.3}\" data-track-index=\"1\" data-width=\"1920\" data-height=\"1080\"></div>",
                escape_html_attribute(&slot.composition_id),
                slot.file_name,
                slot.start_secs,
                slot.duration_secs,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<!doctype html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=1920, height=1080\" />\n  <script src=\"https://cdn.jsdelivr.net/npm/gsap@3.14.2/dist/gsap.min.js\"></script>\n  <style>\n    html, body {{ margin: 0; width: 100%; height: 100%; overflow: hidden; background: #000; }}\n    #preview-all {{ position: relative; width: 1920px; height: 1080px; overflow: hidden; }}\n    .clip {{ position: absolute; inset: 0; width: 100%; height: 100%; overflow: hidden; }}\n  </style>\n</head>\n<body>\n  <div id=\"preview-all\" data-composition-id=\"preview-all\" data-width=\"1920\" data-height=\"1080\" data-duration=\"{duration_secs:.3}\" data-start=\"0\">\n{slots_html}\n  </div>\n  <script>\n    window.__timelines = window.__timelines || {{}};\n    window.__timelines['preview-all'] = gsap.timeline({{ paused: true }});\n  </script>\n</body>\n</html>\n"
    )
}

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('\"', "&quot;")
        .replace('\'', "&#39;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn open_preview(preview_dir: &Path, job_id: u64, job_name: &str) -> Result<(), String> {
    let save_dir = preview_dir.parent().ok_or_else(|| {
        format!(
            "Не вдалося визначити папку задачі для preview: {}",
            preview_dir.display()
        )
    })?;
    let clips = rebuild_preview_all(save_dir)?;
    if clips == 0 {
        return Err("HyperFrames preview: немає готових HTML-кліпів.".to_string());
    }

    let entry_path = preview_dir.join("index.html");
    if !entry_path.is_file() {
        return Err(format!(
            "HyperFrames preview не містить index.html: {}",
            entry_path.display()
        ));
    }

    let entry_html = std::fs::read_to_string(&entry_path)
        .map_err(|e| format!("Не вдалося прочитати HyperFrames preview: {}", e))?;
    for required in [
        "data-composition-id",
        "data-width",
        "data-height",
        "window.__timelines",
    ] {
        if !entry_html.contains(required) {
            return Err(format!(
                "Некоректний HyperFrames preview у {}: відсутній {}. Створіть валідну кореневу композицію, а не HTML-галерею.",
                entry_path.display(),
                required
            ));
        }
    }

    crate::logger::log_job(
        job_id,
        job_name,
        &format!(
            "HyperFrames: запускаємо preview у {}",
            preview_dir.display()
        ),
    );

    // На Windows npx доступний як npx.cmd; запускаємо його без cmd /C.
    let mut cmd = new_npx_command();
    cmd.current_dir(preview_dir)
        .args(["hyperframes", "preview"]);

    let child = crate::api::process::spawn_tracked(&mut cmd, Some(job_id))
        .map_err(|e| format!("Не вдалося запустити HyperFrames preview: {}", e))?;

    std::thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(())
}

pub fn render_pending_segments_async(
    save_dir: PathBuf,
    job_id: u64,
    job_name: String,
    ctx: egui::Context,
    timeline_rebuild_requested: Arc<Mutex<bool>>,
) {
    std::thread::spawn(
        move || match render_pending_segments(&save_dir, job_id, &job_name) {
            Ok(rendered) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("HyperFrames: render завершено, кліпів: {}", rendered),
                );
                *timeline_rebuild_requested.lock().unwrap() = true;
                ctx.request_repaint();
            }
            Err(e) => {
                crate::logger::log_job(
                    job_id,
                    &job_name,
                    &format!("HyperFrames render error: {}", e),
                );
                ctx.request_repaint();
            }
        },
    );
}

fn render_pending_segments(save_dir: &Path, job_id: u64, job_name: &str) -> Result<usize, String> {
    let segments_path = save_dir.join("segments.json");
    let content = std::fs::read_to_string(&segments_path)
        .map_err(|e| format!("Не вдалося прочитати segments.json: {}", e))?;
    let mut timeline =
        serde_json::from_str::<crate::core::pipeline::timeline::sync::Timeline>(&content)
            .map_err(|e| format!("Невалідний segments.json: {}", e))?;

    let media_dir = save_dir.join("media");
    std::fs::create_dir_all(&media_dir)
        .map_err(|e| format!("Не вдалося створити media/: {}", e))?;

    let mut rendered = 0usize;

    for segment in timeline.segments.iter_mut() {
        if segment.media_type
            != crate::core::pipeline::timeline::sync::SegmentMediaType::Hyperframes
        {
            continue;
        }

        let Some(source_rel) = segment.media.clone() else {
            continue;
        };
        if !source_rel.to_ascii_lowercase().ends_with(".html") {
            continue;
        }

        let source_path = save_dir.join(&source_rel);
        if !source_path.exists() {
            return Err(format!(
                "HyperFrames HTML не знайдено для сегмента {}: {}",
                segment.index + 1,
                source_path.display()
            ));
        }
        let clip_dir = source_path.parent().ok_or_else(|| {
            format!(
                "Не вдалося визначити папку кліпу для сегмента {}",
                segment.index + 1
            )
        })?;

        let output_rel = format!("media/{:04}.mp4", segment.index + 1);
        let output_path = media_dir.join(format!("{:04}.mp4", segment.index + 1));

        crate::logger::log_job(
            job_id,
            job_name,
            &format!(
                "HyperFrames: render сегмента {} з {}",
                segment.index + 1,
                clip_dir.display()
            ),
        );

        let mut cmd = new_npx_command();
        cmd.current_dir(clip_dir)
            .args(["hyperframes", "render", "--quality", "high", "--output"])
            .arg(&output_path);

        let output = crate::api::process::output_tracked(&mut cmd, Some(job_id))
            .map_err(|e| format!("HyperFrames render spawn error: {}", e))?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "HyperFrames render failed for segment {}. stdout: {} stderr: {}",
                segment.index + 1,
                stdout.trim(),
                stderr.trim()
            ));
        }
        if !output_path.exists() {
            return Err(format!(
                "HyperFrames render не створив файл для сегмента {}: {}",
                segment.index + 1,
                output_path.display()
            ));
        }

        segment.media = Some(output_rel);
        rendered += 1;
    }

    if rendered == 0 {
        return Ok(0);
    }

    let json = serde_json::to_string_pretty(&timeline).map_err(|e| format!("JSON error: {}", e))?;
    std::fs::write(&segments_path, json).map_err(|e| format!("Write error: {}", e))?;

    Ok(rendered)
}
