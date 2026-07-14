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

pub fn open_preview(preview_dir: &Path, job_id: u64, job_name: &str) -> Result<(), String> {
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
