mod audio;
mod frame_cache;
mod inspector;
mod media;
mod media_pool;
mod preview;
mod state;
mod timeline;
mod topbar;
mod types;
mod utils;

pub use audio::{
    AudioPlayer, PlayingAudio, embedded_audio_cache_path, extract_embedded_audio_async,
};
pub use frame_cache::FrameCache;
pub use media::MediaItem;
pub use state::MontageEditorState;
pub use types::{
    ClipKind, MontageEditorActions, MontagePreviewSettings, PreviewQuality, PreviewRenderSettings,
};

use crate::localization::{Language, translate};
use eframe::egui;
use egui::{Color32, Frame, Sense, Stroke, Vec2};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// ─── Головне вікно редактора ──────────────────────────────────────────────────

pub fn draw_montage_editor_window(
    ctx: &egui::Context,
    language: Language,
    open_job: &mut Option<u64>,
    state: &mut Option<MontageEditorState>,
    jobs: &[crate::queue::PipelineJob],
    anim_loading: &Arc<Mutex<HashSet<PathBuf>>>,
    regen_paths: &HashSet<PathBuf>,
    preview_render: PreviewRenderSettings,
) -> MontageEditorActions {
    let job_id = match *open_job {
        Some(id) => id,
        None => return MontageEditorActions::default(),
    };

    if state.is_none() {
        if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
            *state = Some(MontageEditorState::load(
                std::path::Path::new(&job.settings.save_path),
                &job.name,
                preview_render,
            ));
        } else {
            *open_job = None;
            return MontageEditorActions::default();
        }
    }

    // Синхронізуємо налаштування ефектів превью з поточним JobSettings
    if let Some(ref mut s) = *state {
        if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
            s.preview_settings = MontagePreviewSettings {
                zoom_enabled: job.settings.montage_image_zoom_enabled,
                zoom_mode: job.settings.montage_image_zoom_mode.clone(),
                zoom_scale: job.settings.montage_image_zoom_scale,
                shake_enabled: job.settings.montage_image_shake_enabled,
                shake_intensity: job.settings.montage_image_shake_intensity,
                transition: job.settings.montage_transition.clone(),
                transition_duration: job.settings.montage_transition_duration,
            };
        }
    }

    let editor = match state {
        Some(s) => s,
        None => return MontageEditorActions::default(),
    };

    // Очищаємо накопичені дії попереднього кадру
    editor.pending_animate_paths.clear();
    editor.pending_regen = None;

    // Оновлюємо duration_secs/has_audio для медіа, у яких фоновий ffprobe щойно завершився
    {
        let mut probe_updated = false;
        for media in editor.media_pool.iter_mut() {
            if media.refresh_probe() {
                probe_updated = true;
            }
        }
        if probe_updated {
            ctx.request_repaint();
        }
    }

    // Оновлюємо плейсхолдери якщо підтверджено вибір стоку.
    // needs_stock_refresh залишається true поки є незавантажені файли.
    if editor.needs_stock_refresh {
        let still_pending = state::refresh_placeholder_clips(editor);
        editor.needs_stock_refresh = still_pending;
    }

    // Поки є медіа з незавершеною екстракцією кадрів — продовжуємо опитування,
    // щоб UI автоматично підхопив новий кадр як тільки ffmpeg його запише.
    if editor
        .media_pool
        .iter()
        .any(|m| !m.is_extraction_complete())
    {
        ctx.request_repaint_after(std::time::Duration::from_millis(300));
    }

    // Після завершення оживлення (.jpg → .mp4): оновлюємо пул та кліпи
    {
        let loading = anim_loading.lock().unwrap();
        let mut replacements: Vec<(PathBuf, PathBuf)> = vec![];
        let mut to_remove: Vec<usize> = vec![];
        for (i, m) in editor.media_pool.iter().enumerate() {
            if !m.path.exists() && !loading.contains(&m.path) {
                let mp4 = m.path.with_extension("mp4");
                if mp4.exists() {
                    replacements.push((m.path.clone(), mp4));
                } else {
                    to_remove.push(i);
                }
            }
        }
        drop(loading);

        for (old, new) in replacements {
            let save_path = editor.save_path.clone();
            for clip in &mut editor.clips {
                if clip.path.as_deref() == Some(old.as_path()) {
                    clip.path = Some(new.clone());
                    clip.kind = ClipKind::Video;
                    clip.name = new
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
            if let Some(m) = editor.media_pool.iter_mut().find(|m| m.path == old) {
                let old_id = m.id.clone();
                *m = MediaItem::new(new.clone(), &save_path, editor.preview_render);
                m.id = old_id; // зберігаємо ID щоб кліпи знаходили медіа по media_id
            }
            if editor.pool_preview.as_deref() == Some(old.as_path()) {
                editor.pool_preview = Some(new);
                editor.pool_preview_texture = None;
            }
        }
        for &i in to_remove.iter().rev() {
            editor.media_pool.remove(i);
        }

        // Другий прохід: кліпи що вказують на неіснуючий файл, але в пулі вже є відповідний .mp4
        let clip_fixes: Vec<(String, PathBuf, String)> = editor
            .clips
            .iter()
            .filter_map(|clip| {
                clip.path.as_ref().and_then(|p| {
                    if !p.exists() {
                        let mp4 = p.with_extension("mp4");
                        editor
                            .media_pool
                            .iter()
                            .find(|m| m.path == mp4)
                            .map(|pm| (clip.id.clone(), pm.path.clone(), pm.name.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect();
        for (id, new_path, new_name) in clip_fixes {
            if let Some(clip) = editor.clips.iter_mut().find(|c| c.id == id) {
                clip.path = Some(new_path);
                clip.kind = ClipKind::Video;
                clip.name = new_name;
            }
        }
    }

    // Пробіл: пауза/відтворення (не перехоплюємо коли фокус у текстовому полі)
    if !ctx.wants_keyboard_input() && ctx.input(|i| i.key_pressed(egui::Key::Space)) {
        editor.is_playing = !editor.is_playing;
        editor.last_frame_time = Instant::now();
        if !editor.is_playing {
            editor.active_audios.clear();
        }
    }

    // Ctrl/Cmd+Z — скасування, Ctrl/Cmd+Y або Ctrl/Cmd+Shift+Z — повторення
    if !ctx.wants_keyboard_input() {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift) {
            editor.undo();
        }
        if ctx.input(|i| {
            (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
        }) {
            editor.redo();
        }
    }

    if editor.is_playing {
        let elapsed = editor.last_frame_time.elapsed().as_secs_f32();
        editor.playhead = (editor.playhead + elapsed).min(editor.total_dur());
        editor.last_frame_time = Instant::now();
        if editor.playhead >= editor.total_dur() {
            editor.is_playing = false;
            editor.playhead = 0.0;
            editor.active_audios.clear();
        } else {
            struct TargetAudio {
                path: PathBuf,
                start: f32,
                duration: f32,
                volume: f32,
                trim_start: f32,
            }
            let mut targets = Vec::new();
            for clip in &editor.clips {
                if clip.kind == ClipKind::Audio {
                    if let Some(ref cp) = clip.path {
                        // Вбудоване аудіо відеофайлу (.mp4) — rodio не декодує mp4,
                        // тому грає з попередньо витягнутого WAV-кешу
                        let play_path = if clip.is_embedded_audio {
                            let cached = embedded_audio_cache_path(cp, &editor.save_path);
                            if cached.exists() {
                                cached
                            } else {
                                continue;
                            }
                        } else {
                            cp.clone()
                        };
                        let vol = editor
                            .track_volumes
                            .get(clip.track_idx)
                            .copied()
                            .unwrap_or(1.0);
                        targets.push(TargetAudio {
                            path: play_path,
                            start: clip.start_secs,
                            duration: clip.duration,
                            volume: vol,
                            trim_start: clip.trim_start,
                        });
                    }
                }
            }

            let playhead = editor.playhead;

            // 1. Зупиняємо ті, що вже не мають грати
            editor.active_audios.retain(|active| {
                targets.iter().any(|t| {
                    t.path == active.path
                        && (t.start - active.start_secs).abs() < 0.001
                        && playhead >= t.start
                        && playhead < t.start + t.duration
                })
            });

            // 2. Запускаємо нові
            for t in targets {
                let should_play = playhead >= t.start && playhead < t.start + t.duration;
                if should_play {
                    let already_playing = editor.active_audios.iter().any(|active| {
                        active.path == t.path && (active.start_secs - t.start).abs() < 0.001
                    });
                    if !already_playing {
                        let offset = playhead - t.start + t.trim_start;
                        if let Some(player) = AudioPlayer::start(&t.path, offset, t.volume) {
                            editor.active_audios.push(PlayingAudio {
                                path: t.path,
                                start_secs: t.start,
                                duration: t.duration,
                                player,
                            });
                        }
                    }
                }
            }
        }
        ctx.request_repaint();
    }

    let title = format!(
        "{}: {} #{}",
        translate(language, "montage_editor_title"),
        editor.job_name,
        job_id + 1
    );
    let mut is_open = true;
    let mut close_after = false;

    let is_awaiting = jobs
        .iter()
        .find(|j| j.id == job_id)
        .map(|j| *j.status.lock().unwrap() == crate::queue::JobStatus::AwaitingMontageControl)
        .unwrap_or(false);

    let win_id = if editor.maximized {
        "montage_editor_window_maximized"
    } else {
        "montage_editor_window"
    };

    let mut window = egui::Window::new(&title)
        .id(egui::Id::new(win_id))
        .open(&mut is_open)
        .resizable(!editor.maximized)
        .movable(!editor.maximized)
        .collapsible(true);

    if editor.maximized {
        let screen = ctx.screen_rect();
        // Враховуємо тіні та рамки вікна, щоб воно не вилазило за межі програми
        let margin = 6.0;
        let rect = egui::Rect::from_min_max(
            egui::pos2(screen.min.x + margin, screen.min.y + margin),
            egui::pos2(screen.max.x - margin, screen.max.y - margin),
        );
        window = window.fixed_rect(rect);
    } else {
        window = window
            .default_size([1100.0, 680.0])
            .min_size([700.0, 480.0]);
    }

    let window_response = window.show(ctx, |ui| {
        if topbar::draw_topbar(ui, language, editor, is_awaiting, job_id, jobs) {
            close_after = true;
        }
        ui.separator();

        egui::TopBottomPanel::bottom("montage_editor_timeline_panel")
            .resizable(false)
            .exact_height(editor.timeline_height)
            .frame(
                Frame::none()
                    .fill(Color32::from_rgb(14, 14, 17))
                    .inner_margin(egui::Margin::symmetric(4.0, 4.0)),
            )
            .show_inside(ui, |ui| {
                timeline::draw_timeline(ui, language, editor, anim_loading, regen_paths);
            });

        egui::CentralPanel::default()
            .frame(Frame::none())
            .show_inside(ui, |ui| {
                egui::SidePanel::left("editor_media_pool")
                    .resizable(true)
                    .default_width(220.0)
                    .min_width(160.0)
                    .frame(
                        Frame::none()
                            .fill(Color32::from_rgb(18, 18, 20))
                            .inner_margin(6.0),
                    )
                    .show_inside(ui, |ui| {
                        media_pool::draw_media_pool(
                            ui,
                            language,
                            editor,
                            anim_loading,
                            regen_paths,
                        );
                    });

                egui::SidePanel::right("editor_inspector")
                    .resizable(true)
                    .default_width(240.0)
                    .min_width(180.0)
                    .frame(
                        Frame::none()
                            .fill(Color32::from_rgb(18, 18, 20))
                            .inner_margin(6.0),
                    )
                    .show_inside(ui, |ui| {
                        inspector::draw_inspector(ui, language, editor);
                    });

                egui::CentralPanel::default()
                    .frame(
                        Frame::none()
                            .fill(Color32::from_rgb(10, 10, 12))
                            .inner_margin(6.0),
                    )
                    .show_inside(ui, |ui| {
                        preview::draw_preview(ui, editor);
                    });
            });
    });

    if let Some(ref inner_resp) = window_response {
        if inner_resp.response.double_clicked() {
            editor.maximized = !editor.maximized;
        }
    }

    // Збираємо дії з pending полів editor
    let animate_paths = std::mem::take(&mut editor.pending_animate_paths);
    let regen_opt = editor.pending_regen.take();
    let open_stock_picker = editor.pending_open_stock_picker.take();
    let preview_render_changed = editor.pending_preview_render.take();
    let regen_action = regen_opt.and_then(|(path, is_custom)| {
        jobs.iter().find(|j| j.id == job_id).map(|job| {
            (
                path,
                job.settings.clone(),
                is_custom,
                job_id,
                job.name.clone(),
            )
        })
    });

    if !is_open || close_after {
        *open_job = None;
        *state = None;
        return MontageEditorActions {
            animate_paths,
            regen_action,
            open_stock_picker: None,
            preview_render_changed,
        };
    }

    // Fullscreen preview (подвійний клік на медіа в пулі або кліп у таймлінії)
    if let Some(ref preview_path) = editor.pool_preview.clone() {
        // Стейл-кадр: стара текстура щойно стала застарілою після перегенерації.
        // Скидаємо її і показуємо спінер один кадр — щоб GPU-бекенд встиг
        // звільнити старий слот до завантаження нового.
        if editor.preview_stale_path.as_deref() == Some(preview_path.as_path()) {
            editor.pool_preview_texture = None;
            editor.preview_stale_path = None;
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                editor.pool_preview = None;
            } else {
                let screen = ctx.screen_rect();
                egui::Area::new(egui::Id::new("montage_preview_stale"))
                    .fixed_pos(egui::Pos2::ZERO)
                    .order(egui::Order::Tooltip)
                    .interactable(true)
                    .show(ctx, |ui| {
                        ui.allocate_rect(screen, Sense::hover());
                        ui.painter()
                            .rect_filled(screen, 0.0, Color32::from_black_alpha(215));
                        ui.put(
                            egui::Rect::from_center_size(screen.center(), Vec2::splat(40.0)),
                            egui::Spinner::new().size(40.0),
                        );
                    });
                ctx.request_repaint();
            }
        } else {
            let need_load = editor
                .pool_preview_texture
                .as_ref()
                .map(|(p, _)| p != preview_path)
                .unwrap_or(true);
            if need_load {
                if let Some(tex) = load_preview_texture(ctx, preview_path, editor) {
                    editor.pool_preview_texture = Some((preview_path.clone(), tex));
                }
            }

            let texture = editor.pool_preview_texture.as_ref().and_then(|(p, t)| {
                if p == preview_path {
                    Some(t.clone())
                } else {
                    None
                }
            });

            if let Some(texture) = texture {
                let is_anim = anim_loading.lock().unwrap().contains(preview_path)
                    || regen_paths.contains(preview_path);
                let (keep_open, regen_kind) = draw_montage_media_preview(ctx, &texture, is_anim);
                if !keep_open {
                    editor.pool_preview = None;
                    editor.pool_preview_texture = None;
                }
                if let Some(is_custom) = regen_kind {
                    editor.pending_regen = Some((preview_path.clone(), is_custom));
                }
            } else {
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    editor.pool_preview = None;
                } else {
                    draw_preview_loading_overlay(ctx, "montage_preview_loading");
                }
            }
        }
    }

    MontageEditorActions {
        animate_paths,
        regen_action,
        open_stock_picker,
        preview_render_changed,
    }
}

/// Завантажує текстуру для fullscreen preview: зображення читає напряму,
/// для відео просить перший scrub-кадр ліниво, без масової передобробки всього пулу.
fn load_preview_texture(
    ctx: &egui::Context,
    path: &Path,
    editor: &mut MontageEditorState,
) -> Option<egui::TextureHandle> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp") {
        crate::gui::gallery::preview::load_image_texture(ctx, path)
    } else if matches!(ext.as_str(), "mp4" | "mov" | "webm") {
        let media = editor.media_pool.iter().find(|m| m.path == path).cloned()?;
        editor
            .frame_cache
            .get_frame(ctx, &media, 0.0, false, editor.preview_render)
    } else {
        None
    }
}

fn draw_preview_loading_overlay(ctx: &egui::Context, id: &str) {
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new(id))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Tooltip)
        .interactable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(screen, Sense::hover());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(215));
            ui.put(
                egui::Rect::from_center_size(screen.center(), Vec2::splat(40.0)),
                egui::Spinner::new().size(40.0),
            );
        });
    ctx.request_repaint();
}

/// Fullscreen preview поверх всього UI з повним блокуванням кліків.
/// Повертає (keep_open, regen_kind): keep_open=false → закрити;
/// regen_kind: Some(false)=ті ж налаштування, Some(true)=кастомні.
fn draw_montage_media_preview(
    ctx: &egui::Context,
    texture: &egui::TextureHandle,
    is_animating: bool,
) -> (bool, Option<bool>) {
    let mut keep_open = true;
    let mut regen_kind: Option<bool> = None;

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        return (false, None);
    }

    let screen = ctx.screen_rect();
    let pad = 40.0;
    let img_sz = texture.size_vec2();
    let scale =
        ((screen.width() - pad * 2.0) / img_sz.x).min((screen.height() - pad * 2.0) / img_sz.y);
    let disp = img_sz * scale;
    let img_rect = egui::Rect::from_center_size(screen.center(), disp);

    // Order::Tooltip перехоплює всі події поверх Window
    egui::Area::new(egui::Id::new("montage_editor_preview_area"))
        .fixed_pos(egui::Pos2::ZERO)
        .order(egui::Order::Tooltip)
        .interactable(true)
        .show(ctx, |ui| {
            let bg = ui.allocate_rect(screen, Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(215));

            ui.put(
                img_rect,
                egui::Image::from_texture(texture).fit_to_exact_size(disp),
            );

            if is_animating {
                ui.painter()
                    .rect_filled(img_rect, 0.0, Color32::from_black_alpha(120));
                ui.put(img_rect, egui::Spinner::new().size(32.0));
            }

            let btn_sz = egui::vec2(36.0, 36.0);
            let top_y = screen.top() + 22.0;

            let close_c = egui::pos2(screen.right() - 22.0, top_y);
            let close_r = ui.interact(
                egui::Rect::from_center_size(close_c, btn_sz),
                egui::Id::new("mep_close"),
                Sense::click(),
            );
            let cc = if close_r.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(160)
            };
            let r = 8.0;
            let st = Stroke::new(2.0, cc);
            ui.painter()
                .line_segment([close_c + Vec2::new(-r, -r), close_c + Vec2::new(r, r)], st);
            ui.painter()
                .line_segment([close_c + Vec2::new(r, -r), close_c + Vec2::new(-r, r)], st);

            let cust_c = egui::pos2(screen.right() - 66.0, top_y);
            let cust_r = ui.interact(
                egui::Rect::from_center_size(cust_c, btn_sz),
                egui::Id::new("mep_custom"),
                Sense::click(),
            );
            let cc = if cust_r.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(160)
            };
            crate::gui::gallery::icons::draw_menu_icon(
                ui.painter(),
                cust_c,
                8.0,
                Stroke::new(2.0, cc),
            );

            let same_c = egui::pos2(screen.right() - 110.0, top_y);
            let same_r = ui.interact(
                egui::Rect::from_center_size(same_c, btn_sz),
                egui::Id::new("mep_same"),
                Sense::click(),
            );
            let cc = if same_r.hovered() {
                Color32::WHITE
            } else {
                Color32::from_gray(160)
            };
            crate::gui::gallery::icons::draw_refresh_icon(
                ui.painter(),
                same_c,
                9.0,
                Stroke::new(2.0, cc),
            );

            if close_r.clicked() {
                keep_open = false;
            } else if !is_animating && same_r.clicked() {
                regen_kind = Some(false);
            } else if !is_animating && cust_r.clicked() {
                regen_kind = Some(true);
            } else if bg.clicked() {
                if let Some(pos) = bg.interact_pointer_pos() {
                    if !img_rect.contains(pos) {
                        keep_open = false;
                    }
                }
            }
        });

    (keep_open, regen_kind)
}
