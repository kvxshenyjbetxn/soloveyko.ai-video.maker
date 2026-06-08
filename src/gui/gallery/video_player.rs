use eframe::egui;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Стан inline-плеєра відео.
/// Кадри накопичуються поступово з фонового потоку через `pending`.
pub struct VideoPlayer {
    pub path: PathBuf,
    /// Готові кадри для відображення.
    pub frames: Vec<egui::TextureHandle>,
    /// Нові кадри від потоку. Дренуються в `frames` кожен UI-тік.
    pub pending: Arc<Mutex<Vec<egui::TextureHandle>>>,
    /// true коли потік завершив витягування.
    pub loading_done: Arc<Mutex<bool>>,
    pub fps: f32,
    pub current_frame: usize,
    pub last_advance: Instant,
    pub playing: bool,
}

impl VideoPlayer {
    pub fn new(path: PathBuf, fps: f32) -> Self {
        Self {
            path,
            frames: Vec::new(),
            pending: Arc::new(Mutex::new(Vec::new())),
            loading_done: Arc::new(Mutex::new(false)),
            fps,
            current_frame: 0,
            last_advance: Instant::now(),
            playing: true,
        }
    }

    /// Переносить нові кадри з `pending` у `frames`.
    pub fn drain_pending(&mut self) {
        let mut pending = self.pending.lock().unwrap();
        if !pending.is_empty() {
            self.frames.extend(pending.drain(..));
        }
    }

}

// ─── Витягування для мініатюри (перший кадр) ────────────────────────────────

/// Запускає фонове витягування першого кадру відео як thumbnail.
pub fn start_thumbnail_extraction(
    path: PathBuf,
    ctx: egui::Context,
    loading: Arc<Mutex<std::collections::HashSet<PathBuf>>>,
    result: Arc<Mutex<Vec<(PathBuf, Option<egui::TextureHandle>)>>>,
) {
    std::thread::spawn(move || {
        loading.lock().unwrap().insert(path.clone());
        ctx.request_repaint();

        let tex = extract_single_frame_pipe(&path, &ctx, 160);
        result.lock().unwrap().push((path.clone(), tex));
        loading.lock().unwrap().remove(&path);
        ctx.request_repaint();
    });
}

// ─── Витягування для hover-анімації ─────────────────────────────────────────

/// Запускає фонове витягування 8 кадрів для hover-анімації мініатюри.
pub fn start_hover_extraction(
    path: PathBuf,
    ctx: egui::Context,
    loading: Arc<Mutex<std::collections::HashSet<PathBuf>>>,
    result: Arc<Mutex<Vec<(PathBuf, Vec<egui::TextureHandle>)>>>,
) {
    std::thread::spawn(move || {
        loading.lock().unwrap().insert(path.clone());
        ctx.request_repaint();

        let frames = extract_frames_file(&path, &ctx, Some(8), 160, 2.0).unwrap_or_default();
        result.lock().unwrap().push((path.clone(), frames));
        loading.lock().unwrap().remove(&path);
        ctx.request_repaint();
    });
}

// ─── Витягування для повноекранного плеєра (streaming) ──────────────────────

/// Запускає streaming-витягування кадрів у `player.pending` через ffmpeg pipe.
/// Перший кадр з'являється протягом ~200–400ms.
pub fn start_fullscreen_extraction(player: &VideoPlayer, path: PathBuf, ctx: egui::Context) {
    const FPS: f32 = 10.0;
    const TARGET_W: u32 = 480;

    let pending = Arc::clone(&player.pending);
    let done = Arc::clone(&player.loading_done);

    std::thread::spawn(move || {
        let (out_w, out_h) =
            get_video_dimensions(&path, TARGET_W).unwrap_or((TARGET_W, TARGET_W * 9 / 16));

        let scale_filter = format!("fps={},scale={}:{}", FPS, out_w, out_h);
        let frame_bytes = (out_w * out_h * 4) as usize;

        let mut ffmpeg_cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
        ffmpeg_cmd.arg("-i").arg(&path)
            .arg("-vf").arg(&scale_filter)
            .arg("-f").arg("rawvideo")
            .arg("-pix_fmt").arg("rgba")
            .arg("-loglevel").arg("error")
            .arg("-")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        crate::bundle::set_no_window(&mut ffmpeg_cmd);
        let child = ffmpeg_cmd.spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(_) => {
                *done.lock().unwrap() = true;
                ctx.request_repaint();
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let mut reader = std::io::BufReader::new(stdout);
            let mut buf = vec![0u8; frame_bytes];
            let mut idx = 0usize;

            loop {
                match reader.read_exact(&mut buf) {
                    Ok(()) => {
                        let ci = egui::ColorImage::from_rgba_unmultiplied(
                            [out_w as usize, out_h as usize],
                            &buf,
                        );
                        let name = format!("vf_stream_{}_{}", idx, path.to_string_lossy());
                        let tex = ctx.load_texture(name, ci, egui::TextureOptions::LINEAR);
                        pending.lock().unwrap().push(tex);
                        ctx.request_repaint();
                        idx += 1;
                    }
                    Err(_) => break,
                }
            }
        }

        let _ = child.wait();
        *done.lock().unwrap() = true;
        ctx.request_repaint();
    });
}

// ─── Внутрішні утиліти ───────────────────────────────────────────────────────

/// Повертає (width, height) для масштабування відео до `target_w` (парна висота).
fn get_video_dimensions(path: &Path, target_w: u32) -> Option<(u32, u32)> {
    let mut ffprobe_cmd = std::process::Command::new(crate::bundle::ffprobe_path());
    ffprobe_cmd.args(["-v", "quiet", "-select_streams", "v:0",
           "-show_entries", "stream=width,height",
           "-of", "csv=p=0"])
        .arg(path);
    crate::bundle::set_no_window(&mut ffprobe_cmd);
    let output = ffprobe_cmd.output().ok()?;

    let s = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = s.trim().split(',').collect();
    if parts.len() < 2 { return None; }

    let w: u32 = parts[0].trim().parse().ok()?;
    let h: u32 = parts[1].trim().parse().ok()?;
    if w == 0 || h == 0 { return None; }

    let scale = target_w as f32 / w as f32;
    let out_h = ((h as f32 * scale) as u32).max(2) & !1; // парне
    Some((target_w, out_h))
}

/// Витягує один кадр через ffmpeg pipe → TextureHandle.
fn extract_single_frame_pipe(
    path: &Path,
    ctx: &egui::Context,
    width: u32,
) -> Option<egui::TextureHandle> {
    let (out_w, out_h) = get_video_dimensions(path, width).unwrap_or((width, width * 9 / 16));
    let frame_bytes = (out_w * out_h * 4) as usize;

    let mut ffmpeg_frame_cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
    ffmpeg_frame_cmd.arg("-i").arg(path)
        .arg("-vf").arg(format!("scale={}:{}", out_w, out_h))
        .arg("-frames:v").arg("1")
        .arg("-f").arg("rawvideo")
        .arg("-pix_fmt").arg("rgba")
        .arg("-loglevel").arg("error")
        .arg("-")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    crate::bundle::set_no_window(&mut ffmpeg_frame_cmd);
    let mut child = ffmpeg_frame_cmd.spawn().ok()?;

    let mut buf = vec![0u8; frame_bytes];
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_exact(&mut buf).ok()?;
    }
    let _ = child.wait();

    let ci = egui::ColorImage::from_rgba_unmultiplied([out_w as usize, out_h as usize], &buf);
    let name = format!("vthumb_{}", path.to_string_lossy());
    Some(ctx.load_texture(name, ci, egui::TextureOptions::LINEAR))
}

/// Витягує кадри через тимчасові JPEG-файли (для hover-анімації, max 8 кадрів).
fn extract_frames_file(
    path: &Path,
    ctx: &egui::Context,
    max_frames: Option<usize>,
    width: u32,
    fps: f32,
) -> Option<Vec<egui::TextureHandle>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    let hash = h.finish();

    let tmp_dir = std::env::temp_dir()
        .join(format!("soloveyko_vid_hover_{:x}", hash));
    let _ = std::fs::create_dir_all(&tmp_dir);

    // Очищаємо попередні кадри
    if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
        for e in entries.flatten() {
            let _ = std::fs::remove_file(e.path());
        }
    }

    let out_pattern = tmp_dir.join("frame_%06d.jpg");
    let scale_filter = format!("fps={},scale={}:-2", fps, width);

    let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
    cmd.arg("-i").arg(path)
       .arg("-vf").arg(&scale_filter)
       .arg("-q:v").arg("3")
       .arg("-y")
       .arg("-loglevel").arg("error");

    if let Some(n) = max_frames {
        cmd.arg("-frames:v").arg(n.to_string());
    }

    cmd.arg(&out_pattern);
    crate::bundle::set_no_window(&mut cmd);

    let status = cmd.status().ok()?;
    if !status.success() {
        return None;
    }

    let mut frame_entries: Vec<_> = std::fs::read_dir(&tmp_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("jpg"))
        .collect();
    frame_entries.sort_by_key(|e| e.path());

    let frames: Vec<egui::TextureHandle> = frame_entries
        .iter()
        .filter_map(|entry| {
            let data = std::fs::read(entry.path()).ok()?;
            let img = image::load_from_memory(&data).ok()?;
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
            let name = format!("vhover_{:x}_{}", hash, entry.file_name().to_string_lossy());
            Some(ctx.load_texture(name, ci, egui::TextureOptions::LINEAR))
        })
        .collect();

    if frames.is_empty() { None } else { Some(frames) }
}

// ─── UI ─────────────────────────────────────────────────────────────────────

/// Повноекранний відеоплеєр. Повертає `false` якщо потрібно закрити.
pub fn draw_video_player(ctx: &egui::Context, player: &mut VideoPlayer) -> bool {
    // Просуваємо кадр
    if player.playing && !player.frames.is_empty() {
        let frame_dur = Duration::from_secs_f32(1.0 / player.fps);
        if player.last_advance.elapsed() >= frame_dur {
            player.current_frame = (player.current_frame + 1) % player.frames.len();
            player.last_advance = Instant::now();
        }
        ctx.request_repaint_after(frame_dur.saturating_sub(player.last_advance.elapsed()));
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return false;
    }

    let screen = ctx.screen_rect();
    let mut should_close = false;

    egui::Area::new(egui::Id::new("video_player_overlay"))
        .fixed_pos(screen.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let bg_resp = ui.allocate_rect(screen, egui::Sense::click());
            ui.painter().rect_filled(screen, 0.0, egui::Color32::from_black_alpha(240));

            if player.frames.is_empty() {
                // Ще немає жодного кадру — spinner
                ui.put(screen, egui::Spinner::new().size(48.0));
            } else {
                let ctrl_h = 56.0;

                // Поточний кадр
                if let Some(frame) = player.frames.get(player.current_frame) {
                    let max_w = screen.width() - 80.0;
                    let max_h = screen.height() - ctrl_h - 80.0;
                    let sz = frame.size_vec2();
                    let scale = (max_w / sz.x).min(max_h / sz.y);
                    let disp = sz * scale;
                    let center_y = screen.top() + (screen.height() - ctrl_h) / 2.0;
                    let img_rect = egui::Rect::from_center_size(
                        egui::pos2(screen.center().x, center_y), disp,
                    );
                    ui.put(img_rect, egui::Image::from_texture(frame).fit_to_exact_size(disp));
                }

                // Індикатор завантаження у правому верхньому куті поруч із X
                if !*player.loading_done.lock().unwrap() {
                    ui.put(
                        egui::Rect::from_center_size(
                            egui::pos2(screen.right() - 66.0, screen.top() + 22.0),
                            egui::vec2(20.0, 20.0),
                        ),
                        egui::Spinner::new().size(16.0),
                    );
                }

                // Прогрес-бар
                let bar_y = screen.bottom() - ctrl_h;
                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(screen.left() + 40.0, bar_y),
                    egui::vec2(screen.width() - 80.0, 4.0),
                );
                ui.painter().rect_filled(bar_rect, 2.0, egui::Color32::from_gray(70));
                let progress = if !player.frames.is_empty() {
                    player.current_frame as f32 / player.frames.len() as f32
                } else {
                    0.0
                };
                let fill = egui::Rect::from_min_size(
                    bar_rect.min,
                    egui::vec2(bar_rect.width() * progress, 4.0),
                );
                ui.painter().rect_filled(fill, 2.0, egui::Color32::WHITE);

                // Seek по кліку/перетяганню на прогрес-барі
                let seek_resp = ui.interact(
                    bar_rect.expand(8.0),
                    egui::Id::new("vp_seek"),
                    egui::Sense::click_and_drag(),
                );
                if seek_resp.is_pointer_button_down_on() {
                    if let Some(pos) = seek_resp.interact_pointer_pos() {
                        let t = ((pos.x - bar_rect.left()) / bar_rect.width()).clamp(0.0, 1.0);
                        player.current_frame = ((t * player.frames.len() as f32) as usize)
                            .min(player.frames.len().saturating_sub(1));
                        player.last_advance = Instant::now();
                    }
                }

                // Play/Pause
                let play_c = egui::pos2(screen.center().x, bar_y + 26.0);
                let play_rect = egui::Rect::from_center_size(play_c, egui::vec2(32.0, 32.0));
                let play_resp = ui.interact(play_rect, egui::Id::new("vp_play"), egui::Sense::click());
                let btn_col = if play_resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_gray(200) };

                if player.playing {
                    let bw = 4.0;
                    let bh = 14.0;
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(play_c + egui::vec2(-bw * 1.2, 0.0), egui::vec2(bw, bh)),
                        1.0, btn_col,
                    );
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(play_c + egui::vec2(bw * 1.2, 0.0), egui::vec2(bw, bh)),
                        1.0, btn_col,
                    );
                } else {
                    let s = 9.0_f32;
                    ui.painter().add(egui::Shape::convex_polygon(
                        vec![
                            egui::pos2(play_c.x - s * 0.4, play_c.y - s * 0.8),
                            egui::pos2(play_c.x + s * 0.8, play_c.y),
                            egui::pos2(play_c.x - s * 0.4, play_c.y + s * 0.8),
                        ],
                        btn_col,
                        egui::Stroke::NONE,
                    ));
                }

                if play_resp.clicked() {
                    player.playing = !player.playing;
                    if player.playing {
                        player.last_advance = Instant::now();
                    }
                }

                if bg_resp.clicked() {
                    should_close = true;
                }
            }

            // Кнопка ✕ завжди доступна
            let close_c = egui::pos2(screen.right() - 22.0, screen.top() + 22.0);
            let close_rect = egui::Rect::from_center_size(close_c, egui::vec2(36.0, 36.0));
            let close_resp = ui.interact(close_rect, egui::Id::new("vp_close"), egui::Sense::click());
            let cc = if close_resp.hovered() { egui::Color32::WHITE } else { egui::Color32::from_gray(160) };
            let r = 8.0;
            ui.painter().line_segment(
                [close_c + egui::vec2(-r, -r), close_c + egui::vec2(r, r)],
                egui::Stroke::new(2.0, cc),
            );
            ui.painter().line_segment(
                [close_c + egui::vec2(r, -r), close_c + egui::vec2(-r, r)],
                egui::Stroke::new(2.0, cc),
            );
            if close_resp.clicked() {
                should_close = true;
            }
        });

    !should_close
}
