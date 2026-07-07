use super::media::MediaItem;
use super::types::{ClipKind, PreviewRenderSettings};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Condvar, Mutex, Once, OnceLock};
use std::time::{Duration, Instant};

const MAX_PREVIEW_WORKERS: usize = 2;
const DEMAND_TTL: Duration = Duration::from_secs(2);

type FrameLoadResult = (String, Option<egui::TextureHandle>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PreviewTaskPriority {
    CurrentFrame,
    PlaybackAhead,
    ScrubFallback,
    SharpStill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreviewFrameKind {
    Scrub,
    Sharp,
}

struct FrameTaskReply {
    key: String,
    tx: std::sync::mpsc::Sender<FrameLoadResult>,
    ctx: egui::Context,
}

enum PreviewTaskKind {
    Frame {
        frame_idx: u32,
        frame_kind: PreviewFrameKind,
        out_path: PathBuf,
        reply: FrameTaskReply,
    },
    Chunk {
        start_idx: u32,
        end_idx: u32,
        ctx: egui::Context,
    },
}

struct PreviewTask {
    media: MediaItem,
    media_id: String,
    settings: PreviewRenderSettings,
    priority: PreviewTaskPriority,
    order: u64,
    queue_key: String,
    kind: PreviewTaskKind,
}

struct PreviewDemand {
    center_frame: u32,
    playback_active: bool,
    want_sharp: bool,
    settings: PreviewRenderSettings,
    updated_at: Instant,
}

struct PreviewWorkerState {
    queue: Vec<PreviewTask>,
    demands: HashMap<String, PreviewDemand>,
    running_keys: HashSet<String>,
    running_media_ids: HashSet<String>,
    active_jobs: usize,
    next_order: u64,
}

pub(crate) struct PreviewWorker {
    state: Mutex<PreviewWorkerState>,
    condvar: Condvar,
}

impl PreviewWorker {
    pub(crate) fn get() -> &'static Self {
        static WORKER: OnceLock<PreviewWorker> = OnceLock::new();
        static START_THREADS: Once = Once::new();

        let worker = WORKER.get_or_init(|| PreviewWorker {
            state: Mutex::new(PreviewWorkerState {
                queue: Vec::new(),
                demands: HashMap::new(),
                running_keys: HashSet::new(),
                running_media_ids: HashSet::new(),
                active_jobs: 0,
                next_order: 0,
            }),
            condvar: Condvar::new(),
        });

        START_THREADS.call_once(|| {
            for _ in 0..MAX_PREVIEW_WORKERS {
                worker.spawn_thread();
            }
        });

        worker
    }

    pub(crate) fn update_demand(
        &self,
        media: &MediaItem,
        frame_idx: u32,
        playback_active: bool,
        want_sharp: bool,
        settings: PreviewRenderSettings,
    ) {
        let mut state = self.state.lock().unwrap();
        state.prune_expired_demands();
        state.demands.insert(
            media.id.clone(),
            PreviewDemand {
                center_frame: frame_idx,
                playback_active,
                want_sharp,
                settings,
                updated_at: Instant::now(),
            },
        );
        drop(state);
        self.condvar.notify_all();
    }

    pub(crate) fn enqueue_frame(
        &self,
        ctx: &egui::Context,
        media: MediaItem,
        frame_idx: u32,
        settings: PreviewRenderSettings,
        frame_kind: PreviewFrameKind,
        priority: PreviewTaskPriority,
        out_path: PathBuf,
        key: String,
        tx: std::sync::mpsc::Sender<FrameLoadResult>,
    ) {
        let queue_key = format!(
            "frame:{}:{}:{}:{}",
            media.id,
            settings.fps_tag(),
            match frame_kind {
                PreviewFrameKind::Scrub => "scrub",
                PreviewFrameKind::Sharp => "sharp",
            },
            frame_idx,
        );
        self.enqueue_task(PreviewTask {
            media_id: media.id.clone(),
            media,
            settings,
            priority,
            order: 0,
            queue_key,
            kind: PreviewTaskKind::Frame {
                frame_idx,
                frame_kind,
                out_path,
                reply: FrameTaskReply {
                    key,
                    tx,
                    ctx: ctx.clone(),
                },
            },
        });
    }

    pub(crate) fn enqueue_chunk(
        &self,
        ctx: &egui::Context,
        media: MediaItem,
        start_idx: u32,
        end_idx: u32,
        settings: PreviewRenderSettings,
        priority: PreviewTaskPriority,
    ) {
        let queue_key = format!(
            "chunk:{}:{}:{}:{}",
            media.id,
            settings.fps_tag(),
            start_idx,
            end_idx,
        );
        self.enqueue_task(PreviewTask {
            media_id: media.id.clone(),
            media,
            settings,
            priority,
            order: 0,
            queue_key,
            kind: PreviewTaskKind::Chunk {
                start_idx,
                end_idx,
                ctx: ctx.clone(),
            },
        });
    }

    fn enqueue_task(&self, task: PreviewTask) {
        let mut state = self.state.lock().unwrap();
        state.prune_expired_demands();
        state.upsert_task(task);
        drop(state);
        self.condvar.notify_all();
    }

    fn spawn_thread(&'static self) {
        std::thread::spawn(move || {
            loop {
                let task = {
                    let mut state = self.state.lock().unwrap();
                    loop {
                        state.prune_expired_demands();
                        state.drop_stale_tasks();
                        if let Some(task) = state.take_next_task() {
                            break task;
                        }
                        state = self.condvar.wait(state).unwrap();
                    }
                };

                if !self.task_is_stale(&task) {
                    task.execute();
                } else {
                    task.finish_frame(None);
                }

                let mut state = self.state.lock().unwrap();
                state.active_jobs = state.active_jobs.saturating_sub(1);
                state.running_keys.remove(&task.queue_key);
                state.running_media_ids.remove(&task.media_id);
                drop(state);
                self.condvar.notify_all();
            }
        });
    }

    fn task_is_stale(&self, task: &PreviewTask) -> bool {
        let mut state = self.state.lock().unwrap();
        state.prune_expired_demands();
        state.task_is_stale(task)
    }
}

impl PreviewWorkerState {
    fn prune_expired_demands(&mut self) {
        let now = Instant::now();
        self.demands
            .retain(|_, demand| now.duration_since(demand.updated_at) <= DEMAND_TTL);
    }

    fn upsert_task(&mut self, mut task: PreviewTask) {
        if let Some(existing) = self
            .queue
            .iter_mut()
            .find(|existing| existing.queue_key == task.queue_key)
        {
            task.priority = task.priority.min(existing.priority);
            task.order = existing.order;
            *existing = task;
            return;
        }

        if self.running_keys.contains(&task.queue_key) {
            return;
        }

        self.next_order += 1;
        task.order = self.next_order;
        self.queue.push(task);
    }

    fn drop_stale_tasks(&mut self) {
        let mut idx = 0;
        while idx < self.queue.len() {
            if self.task_is_stale(&self.queue[idx]) {
                self.queue.swap_remove(idx);
            } else {
                idx += 1;
            }
        }
    }

    fn take_next_task(&mut self) -> Option<PreviewTask> {
        if self.active_jobs >= self.max_parallel_jobs() {
            return None;
        }

        let best_idx = self
            .queue
            .iter()
            .enumerate()
            .filter(|(_, task)| !self.running_media_ids.contains(&task.media_id))
            .min_by_key(|(_, task)| (task.priority, task.order))
            .map(|(idx, _)| idx)?;

        let task = self.queue.swap_remove(best_idx);
        self.running_keys.insert(task.queue_key.clone());
        self.running_media_ids.insert(task.media_id.clone());
        self.active_jobs += 1;
        Some(task)
    }

    fn max_parallel_jobs(&self) -> usize {
        self.demands
            .values()
            .map(|demand| {
                demand
                    .settings
                    .preview_ffmpeg_process_limit(demand.playback_active)
            })
            .max()
            .unwrap_or(1)
            .clamp(1, MAX_PREVIEW_WORKERS)
    }

    fn task_is_stale(&self, task: &PreviewTask) -> bool {
        let Some(demand) = self.demands.get(&task.media_id) else {
            return true;
        };

        if demand.settings != task.settings {
            return true;
        }

        match &task.kind {
            PreviewTaskKind::Frame {
                frame_idx,
                frame_kind,
                ..
            } => match frame_kind {
                PreviewFrameKind::Scrub => match task.priority {
                    PreviewTaskPriority::CurrentFrame => *frame_idx != demand.center_frame,
                    PreviewTaskPriority::PlaybackAhead => {
                        !demand.playback_active || *frame_idx <= demand.center_frame
                    }
                    PreviewTaskPriority::ScrubFallback => {
                        demand.playback_active
                            || frame_distance(*frame_idx, demand.center_frame)
                                > demand.settings.fallback_frame_distance()
                    }
                    PreviewTaskPriority::SharpStill => false,
                },
                PreviewFrameKind::Sharp => {
                    demand.playback_active
                        || !demand.want_sharp
                        || *frame_idx != demand.center_frame
                }
            },
            PreviewTaskKind::Chunk {
                start_idx, end_idx, ..
            } => match task.priority {
                PreviewTaskPriority::PlaybackAhead => {
                    !demand.playback_active
                        || *end_idx < demand.center_frame
                        || *start_idx
                            > demand.center_frame.saturating_add(
                                demand
                                    .settings
                                    .cached_frame_warmup()
                                    .max(demand.settings.playback_prefetch_frames() * 4),
                            )
                }
                PreviewTaskPriority::ScrubFallback => {
                    demand.playback_active
                        || demand.center_frame
                            < start_idx.saturating_sub(demand.settings.fallback_frame_distance())
                        || demand.center_frame
                            > end_idx.saturating_add(demand.settings.fallback_frame_distance())
                }
                _ => false,
            },
        }
    }
}

impl PreviewTask {
    fn execute(&self) {
        match &self.kind {
            PreviewTaskKind::Frame {
                frame_idx,
                frame_kind,
                out_path,
                reply,
            } => {
                let ready = if out_path.exists() {
                    true
                } else {
                    match frame_kind {
                        PreviewFrameKind::Scrub => {
                            generate_scrub_frame(&self.media, *frame_idx, self.settings, out_path)
                        }
                        PreviewFrameKind::Sharp => {
                            generate_sharp_frame(&self.media, *frame_idx, self.settings, out_path)
                        }
                    }
                };

                if ready && matches!(frame_kind, PreviewFrameKind::Scrub) {
                    self.media
                        .extraction_complete
                        .store(true, Ordering::Relaxed);
                }

                let texture = if ready {
                    load_frame_from_disk(&reply.ctx, out_path, &reply.key)
                } else {
                    None
                };
                let _ = reply.tx.send((reply.key.clone(), texture));
                reply.ctx.request_repaint();
            }
            PreviewTaskKind::Chunk {
                start_idx,
                end_idx,
                ctx,
            } => {
                let ok = generate_scrub_chunk(&self.media, *start_idx, *end_idx, self.settings);
                if ok {
                    self.media
                        .extraction_complete
                        .store(true, Ordering::Relaxed);
                }
                ctx.request_repaint();
            }
        }
    }

    fn finish_frame(&self, texture: Option<egui::TextureHandle>) {
        if let PreviewTaskKind::Frame { reply, .. } = &self.kind {
            let _ = reply.tx.send((reply.key.clone(), texture));
            reply.ctx.request_repaint();
        }
    }
}

fn frame_distance(a: u32, b: u32) -> u32 {
    if a >= b { a - b } else { b - a }
}

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
    let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba.into_raw());
    Some(ctx.load_texture(key, ci, egui::TextureOptions::LINEAR))
}

fn generate_scrub_frame(
    media: &MediaItem,
    frame_idx: u32,
    settings: PreviewRenderSettings,
    out: &Path,
) -> bool {
    if std::fs::create_dir_all(&media.cache_dir).is_err() {
        return false;
    }
    if !matches!(media.kind, ClipKind::Video) {
        return false;
    }

    let seek = (frame_idx.saturating_sub(1) as f32 / settings.fps)
        .min(media.duration_secs)
        .max(0.0);
    let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
    cmd.args([
        "-y",
        "-v",
        "error",
        "-threads",
        "1",
        "-ss",
        &format!("{seek:.3}"),
        "-i",
        media.path.to_string_lossy().as_ref(),
        "-vframes",
        "1",
        "-vf",
        &format!("scale={}:-2", settings.quality.scrub_width()),
        "-q:v",
        settings.quality.ffmpeg_qscale(),
        out.to_string_lossy().as_ref(),
    ]);
    crate::bundle::set_no_window(&mut cmd);

    let _permit = crate::api::ffmpeg::FfmpegLimiter::get().acquire();
    matches!(crate::api::ffmpeg::run_tracked(&mut cmd), Ok(status) if status.success())
        && out.exists()
}

fn generate_sharp_frame(
    media: &MediaItem,
    frame_idx: u32,
    settings: PreviewRenderSettings,
    out: &Path,
) -> bool {
    if std::fs::create_dir_all(&media.sharp_cache_dir).is_err() {
        return false;
    }
    if !matches!(media.kind, ClipKind::Video) {
        return false;
    }

    let seek = (frame_idx.saturating_sub(1) as f32 / settings.fps)
        .min(media.duration_secs)
        .max(0.0);
    let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
    cmd.args([
        "-y",
        "-v",
        "error",
        "-threads",
        "1",
        "-ss",
        &format!("{seek:.3}"),
        "-i",
        media.path.to_string_lossy().as_ref(),
        "-vframes",
        "1",
        "-vf",
        &format!("scale={}:-2", settings.quality.sharp_width()),
        "-q:v",
        settings.quality.sharp_ffmpeg_qscale(),
        out.to_string_lossy().as_ref(),
    ]);
    crate::bundle::set_no_window(&mut cmd);

    let _permit = crate::api::ffmpeg::FfmpegLimiter::get().acquire();
    matches!(crate::api::ffmpeg::run_tracked(&mut cmd), Ok(status) if status.success())
        && out.exists()
}

fn generate_scrub_chunk(
    media: &MediaItem,
    start_idx: u32,
    end_idx: u32,
    settings: PreviewRenderSettings,
) -> bool {
    if std::fs::create_dir_all(&media.cache_dir).is_err() {
        return false;
    }
    if !matches!(media.kind, ClipKind::Video) {
        return false;
    }

    let fps = settings.fps.max(1.0);
    let start_secs = (start_idx.saturating_sub(1) as f32 / fps).max(0.0);
    let duration_secs = (end_idx.saturating_sub(start_idx) as f32 / fps)
        .min((media.duration_secs - start_secs).max(0.1));
    let out_pattern = media.cache_dir.join("%06d.jpg");

    let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
    cmd.args([
        "-y",
        "-v",
        "error",
        "-threads",
        "1",
        "-ss",
        &format!("{start_secs:.3}"),
        "-i",
        media.path.to_string_lossy().as_ref(),
        "-t",
        &format!("{duration_secs:.3}"),
        "-vf",
        &format!("fps={:.4},scale={}:-2", fps, settings.quality.scrub_width()),
        "-q:v",
        settings.quality.ffmpeg_qscale(),
        "-start_number",
        &start_idx.to_string(),
        out_pattern.to_string_lossy().as_ref(),
    ]);
    crate::bundle::set_no_window(&mut cmd);

    let _permit = crate::api::ffmpeg::FfmpegLimiter::get().acquire();
    let status_ok =
        matches!(crate::api::ffmpeg::run_tracked(&mut cmd), Ok(status) if status.success());
    // Біля кінця відео фільтр fps= може віддати на кілька кадрів менше за теоретичний
    // розрахунок (округлення тайм-кодів останнього реального кадру джерела) — це не
    // провал, якщо взагалі щось з'явилось близько до очікуваного кінця. Якщо вимагати
    // рівно end_idx, такий chunk ніколи не позначиться готовим і буде перезапускатись
    // на кожному тіку програвання назавжди.
    let tail_tolerance = 5;
    status_ok
        && (end_idx.saturating_sub(tail_tolerance)..=end_idx)
            .rev()
            .any(|idx| media.cache_dir.join(format!("{idx:06}.jpg")).exists())
}
