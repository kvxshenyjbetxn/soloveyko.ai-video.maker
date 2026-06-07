use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::localization::{Language, translate};

/// Статус перевірки CLI-інструменту.
#[derive(Clone, PartialEq)]
pub enum ToolStatus {
    Checking,
    Installed(String),
    NotInstalled,
}

/// Стан авто-завантаження бінарника (ffmpeg, whisper тощо).
#[derive(Clone, PartialEq)]
pub enum BinaryDownload {
    Idle,
    Downloading(String),
    Done,
    Failed(String),
}

// Зворотна сумісність для коду, що ще використовує старе ім'я.
pub type FfmpegDownload = BinaryDownload;

/// Асинхронні стани перевірки CLI-інструментів.
pub struct ToolChecks {
    pub gemini: Arc<Mutex<ToolStatus>>,
    pub claude: Arc<Mutex<ToolStatus>>,
    pub ffmpeg: Arc<Mutex<ToolStatus>>,
    pub ffmpeg_download: Arc<Mutex<BinaryDownload>>,
    pub whisper: Arc<Mutex<ToolStatus>>,
    pub whisper_download: Arc<Mutex<BinaryDownload>>,
    pub whisperx: Arc<Mutex<ToolStatus>>,
    pub whisperx_download: Arc<Mutex<BinaryDownload>>,
    /// Опційний AMD-оптимізований whisper — не завантажується автоматично.
    pub whisper_amd_download: Arc<Mutex<BinaryDownload>>,
}

impl ToolChecks {
    pub fn new() -> Self {
        Self {
            gemini: Arc::new(Mutex::new(ToolStatus::Checking)),
            claude: Arc::new(Mutex::new(ToolStatus::Checking)),
            ffmpeg: Arc::new(Mutex::new(ToolStatus::Checking)),
            ffmpeg_download: Arc::new(Mutex::new(BinaryDownload::Idle)),
            whisper: Arc::new(Mutex::new(ToolStatus::Checking)),
            whisper_download: Arc::new(Mutex::new(BinaryDownload::Idle)),
            whisperx: Arc::new(Mutex::new(ToolStatus::Checking)),
            whisperx_download: Arc::new(Mutex::new(BinaryDownload::Idle)),
            whisper_amd_download: Arc::new(Mutex::new(BinaryDownload::Idle)),
        }
    }

    /// Скидає всі статуси і перезапускає перевірки.
    pub fn restart(&self, ctx: egui::Context) {
        *self.gemini.lock().unwrap() = ToolStatus::Checking;
        *self.claude.lock().unwrap() = ToolStatus::Checking;
        *self.ffmpeg.lock().unwrap() = ToolStatus::Checking;
        *self.ffmpeg_download.lock().unwrap() = BinaryDownload::Idle;
        *self.whisper.lock().unwrap() = ToolStatus::Checking;
        *self.whisper_download.lock().unwrap() = BinaryDownload::Idle;
        *self.whisperx.lock().unwrap() = ToolStatus::Checking;
        *self.whisperx_download.lock().unwrap() = BinaryDownload::Idle;
        // whisper_amd — тільки скидаємо до Idle, не авто-скачуємо
        if !crate::bundle::whisper_amd_local_exists() {
            *self.whisper_amd_download.lock().unwrap() = BinaryDownload::Idle;
        }
        self.start(ctx);
    }

    /// Запускає всі перевірки у фонових потоках.
    pub fn start(&self, ctx: egui::Context) {
        Self::check("gemini", "--version", Arc::clone(&self.gemini), ctx.clone());
        Self::check("claude", "--version", Arc::clone(&self.claude), ctx.clone());
        Self::check_ffmpeg(
            Arc::clone(&self.ffmpeg),
            Arc::clone(&self.ffmpeg_download),
            ctx.clone(),
        );
        Self::check_whisper(
            Arc::clone(&self.whisper),
            Arc::clone(&self.whisper_download),
            ctx.clone(),
        );
        Self::check_whisperx(
            Arc::clone(&self.whisperx),
            Arc::clone(&self.whisperx_download),
            ctx,
        );
    }

    fn check(name: &'static str, version_flag: &'static str, status: Arc<Mutex<ToolStatus>>, ctx: egui::Context) {
        std::thread::spawn(move || {
            #[cfg(target_os = "windows")]
            let result = std::process::Command::new("cmd")
                .args(&["/C", name, version_flag])
                .output();

            #[cfg(not(target_os = "windows"))]
            let result = std::process::Command::new(name)
                .arg(version_flag)
                .output();

            let new_status = match result {
                Ok(out) if out.status.success() => {
                    let ver = String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    ToolStatus::Installed(ver)
                }
                _ => ToolStatus::NotInstalled,
            };
            *status.lock().unwrap() = new_status;
            ctx.request_repaint();
        });
    }

    /// Перевіряє ffmpeg (враховує ~/bin/), при відсутності — авто-скачує.
    fn check_ffmpeg(
        ffmpeg_status: Arc<Mutex<ToolStatus>>,
        ffmpeg_download: Arc<Mutex<FfmpegDownload>>,
        ctx: egui::Context,
    ) {
        std::thread::spawn(move || {
            let path = crate::bundle::ffmpeg_path();
            let result = std::process::Command::new(&path)
                .arg("-version")
                .output();

            let installed = matches!(&result, Ok(out) if out.status.success());

            if installed {
                let ver = result.ok()
                    .map(|out| String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("").trim().to_string())
                    .unwrap_or_default();
                *ffmpeg_status.lock().unwrap() = ToolStatus::Installed(ver);
                ctx.request_repaint();
                return;
            }

            // Не знайдено — починаємо авто-завантаження
            *ffmpeg_status.lock().unwrap() = ToolStatus::NotInstalled;
            *ffmpeg_download.lock().unwrap() = FfmpegDownload::Downloading("підготовка...".to_string());
            ctx.request_repaint();

            let dl = Arc::clone(&ffmpeg_download);
            let ctx2 = ctx.clone();
            let result = crate::bundle::download_all(move |label| {
                *dl.lock().unwrap() = FfmpegDownload::Downloading(label);
                ctx2.request_repaint();
            });

            match result {
                Ok(()) => {
                    // Після завантаження — перевіряємо знову
                    let path2 = crate::bundle::ffmpeg_path();
                    let check = std::process::Command::new(&path2).arg("-version").output();
                    let new_status = match check {
                        Ok(out) if out.status.success() => {
                            let ver = String::from_utf8_lossy(&out.stdout)
                                .lines()
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string();
                            ToolStatus::Installed(ver)
                        }
                        _ => ToolStatus::NotInstalled,
                    };
                    *ffmpeg_status.lock().unwrap() = new_status;
                    *ffmpeg_download.lock().unwrap() = BinaryDownload::Done;
                }
                Err(e) => {
                    *ffmpeg_download.lock().unwrap() = BinaryDownload::Failed(e);
                }
            }
            ctx.request_repaint();
        });
    }

    /// Перевіряє whisperx (папка у bin_dir), при відсутності — авто-скачує (macOS та Windows).
    fn check_whisperx(
        whisperx_status: Arc<Mutex<ToolStatus>>,
        whisperx_download: Arc<Mutex<BinaryDownload>>,
        ctx: egui::Context,
    ) {
        std::thread::spawn(move || {
            if crate::bundle::whisperx_local_exists() {
                *whisperx_status.lock().unwrap() = ToolStatus::Installed("bundled".to_string());
                ctx.request_repaint();
                return;
            }

            // Не знайдено — починаємо авто-завантаження (лише macOS та Windows)
            *whisperx_status.lock().unwrap() = ToolStatus::NotInstalled;
            ctx.request_repaint();

            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                *whisperx_download.lock().unwrap() = BinaryDownload::Downloading("підготовка...".to_string());
                ctx.request_repaint();

                let dl = Arc::clone(&whisperx_download);
                let ctx2 = ctx.clone();
                let result = crate::bundle::download_whisperx(move |label| {
                    *dl.lock().unwrap() = BinaryDownload::Downloading(label);
                    ctx2.request_repaint();
                });

                match result {
                    Ok(()) => {
                        let new_status = if crate::bundle::whisperx_local_exists() {
                            ToolStatus::Installed("bundled".to_string())
                        } else {
                            ToolStatus::NotInstalled
                        };
                        *whisperx_status.lock().unwrap() = new_status;
                        *whisperx_download.lock().unwrap() = BinaryDownload::Done;
                    }
                    Err(e) => {
                        *whisperx_download.lock().unwrap() = BinaryDownload::Failed(e);
                    }
                }
            }

            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            {
                let _ = whisperx_download;
            }

            ctx.request_repaint();
        });
    }

    /// Перевіряє whisper (бандлований), при відсутності — авто-скачує.
    fn check_whisper(
        whisper_status: Arc<Mutex<ToolStatus>>,
        whisper_download: Arc<Mutex<BinaryDownload>>,
        ctx: egui::Context,
    ) {
        std::thread::spawn(move || {
            if crate::bundle::whisper_local_exists() {
                *whisper_status.lock().unwrap() = ToolStatus::Installed("bundled".to_string());
                ctx.request_repaint();
                return;
            }

            // Не знайдено — починаємо авто-завантаження
            *whisper_status.lock().unwrap() = ToolStatus::NotInstalled;
            *whisper_download.lock().unwrap() = BinaryDownload::Downloading("підготовка...".to_string());
            ctx.request_repaint();

            let dl = Arc::clone(&whisper_download);
            let ctx2 = ctx.clone();
            let result = crate::bundle::download_whisper(move |label| {
                *dl.lock().unwrap() = BinaryDownload::Downloading(label);
                ctx2.request_repaint();
            });

            match result {
                Ok(()) => {
                    let new_status = if crate::bundle::whisper_local_exists() {
                        ToolStatus::Installed("bundled".to_string())
                    } else {
                        ToolStatus::NotInstalled
                    };
                    *whisper_status.lock().unwrap() = new_status;
                    *whisper_download.lock().unwrap() = BinaryDownload::Done;
                }
                Err(e) => {
                    *whisper_download.lock().unwrap() = BinaryDownload::Failed(e);
                }
            }
            ctx.request_repaint();
        });
    }
}

/// Малює вікно привітання з перевіркою CLI-інструментів.
///
/// Повертає true, якщо щойно було натиснуто кнопку "Закрити".
pub fn draw_welcome_dialog(
    ctx: &egui::Context,
    open: &mut bool,
    dont_show: &mut bool,
    checks: &ToolChecks,
    language: Language,
) -> bool {
    if !*open {
        return false;
    }

    let mut closed = false;

    egui::Window::new(translate(language, "welcome_title"))
        .collapsible(false)
        .resizable(false)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(translate(language, "welcome_desc"));
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let gemini_status = checks.gemini.lock().unwrap().clone();
            let claude_status = checks.claude.lock().unwrap().clone();
            let ffmpeg_status = checks.ffmpeg.lock().unwrap().clone();
            let ffmpeg_download = checks.ffmpeg_download.lock().unwrap().clone();
            let whisper_status = checks.whisper.lock().unwrap().clone();
            let whisper_download = checks.whisper_download.lock().unwrap().clone();
            let whisperx_status = checks.whisperx.lock().unwrap().clone();
            let whisperx_download = checks.whisperx_download.lock().unwrap().clone();

            draw_tool_row(ui, "Gemini CLI", &gemini_status, translate(language, "welcome_gemini_desc"), language);
            ui.add_space(6.0);
            draw_tool_row(ui, "Claude Code", &claude_status, translate(language, "welcome_claude_desc"), language);
            ui.add_space(6.0);
            draw_download_row(ui, "FFmpeg", &ffmpeg_status, &ffmpeg_download, translate(language, "welcome_ffmpeg_desc"), language);
            ui.add_space(6.0);
            draw_download_row(ui, "Whisper", &whisper_status, &whisper_download, translate(language, "welcome_whisper_desc"), language);
            ui.add_space(6.0);
            draw_download_row(ui, "WhisperX", &whisperx_status, &whisperx_download, translate(language, "welcome_whisperx_desc"), language);
            ui.add_space(6.0);

            let whisper_amd_download = checks.whisper_amd_download.lock().unwrap().clone();
            draw_whisper_amd_row(
                ui, language, &whisper_amd_download,
                Arc::clone(&checks.whisper_amd_download), ctx,
            );

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.checkbox(dont_show, translate(language, "welcome_dont_show"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(translate(language, "welcome_close_btn")).clicked() {
                        *open = false;
                        closed = true;
                    }
                    ui.add_space(4.0);
                    if ui.button(translate(language, "welcome_recheck_btn")).clicked() {
                        checks.restart(ctx.clone());
                    }
                });
            });
        });

    closed
}

fn draw_tool_row(
    ui: &mut egui::Ui,
    name: &str,
    status: &ToolStatus,
    description: &str,
    language: Language,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);

        match status {
            ToolStatus::Checking => { ui.spinner(); }
            ToolStatus::Installed(_) => {
                ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(46, 204, 113)).size(16.0).strong());
            }
            ToolStatus::NotInstalled => {
                ui.label(egui::RichText::new("✗").color(egui::Color32::from_rgb(231, 76, 60)).size(16.0).strong());
            }
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).strong());
                match status {
                    ToolStatus::Installed(ver) => {
                        ui.label(egui::RichText::new(ver).color(egui::Color32::from_rgb(46, 204, 113)).size(11.0));
                    }
                    ToolStatus::NotInstalled => {
                        ui.label(egui::RichText::new(translate(language, "welcome_not_installed")).color(egui::Color32::from_rgb(231, 76, 60)).size(11.0));
                    }
                    ToolStatus::Checking => {
                        ui.label(egui::RichText::new(translate(language, "welcome_checking")).weak().size(11.0));
                    }
                }
            });
            ui.label(egui::RichText::new(description).weak().size(11.0));
        });
    });
}

/// Рядок для опційного встановлення Whisper AMD (тільки Windows, без авто-завантаження).
fn draw_whisper_amd_row(
    ui: &mut egui::Ui,
    language: Language,
    download: &BinaryDownload,
    _download_arc: Arc<Mutex<BinaryDownload>>,
    _ctx: &egui::Context,
) {
    let is_installed = crate::bundle::whisper_amd_local_exists();

    ui.horizontal(|ui| {
        ui.set_min_height(30.0);

        // Іконка статусу
        match download {
            BinaryDownload::Downloading(_) => { ui.spinner(); }
            _ if is_installed => {
                ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(46, 204, 113)).size(16.0).strong());
            }
            BinaryDownload::Failed(_) => {
                ui.label(egui::RichText::new("✗").color(egui::Color32::from_rgb(231, 76, 60)).size(16.0).strong());
            }
            _ => {
                ui.label(egui::RichText::new("○").color(egui::Color32::GRAY).size(16.0));
            }
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Whisper AMD").strong());

                match download {
                    BinaryDownload::Downloading(label) => {
                        ui.label(egui::RichText::new(label).color(egui::Color32::from_rgb(255, 200, 0)).size(11.0));
                    }
                    BinaryDownload::Failed(err) => {
                        ui.label(egui::RichText::new(err).color(egui::Color32::from_rgb(231, 76, 60)).size(11.0));
                    }
                    _ if is_installed => {
                        ui.label(egui::RichText::new("встановлено").color(egui::Color32::from_rgb(46, 204, 113)).size(11.0));
                    }
                    _ => {
                        ui.label(egui::RichText::new(translate(language, "welcome_whisper_amd_optional")).weak().size(11.0));
                    }
                }
            });

            ui.label(egui::RichText::new(translate(language, "welcome_whisper_amd_desc")).weak().size(11.0));

            // Кнопка встановлення або повтору — тільки на Windows, якщо не встановлено
            #[cfg(target_os = "windows")]
            if !is_installed {
                match download {
                    BinaryDownload::Idle | BinaryDownload::Done => {
                        ui.add_space(2.0);
                        if ui.small_button(translate(language, "welcome_whisper_amd_install_btn")).clicked() {
                            start_whisper_amd_download(_download_arc, _ctx.clone());
                        }
                    }
                    BinaryDownload::Failed(_) => {
                        ui.add_space(2.0);
                        if ui.small_button(translate(language, "welcome_recheck_btn")).clicked() {
                            start_whisper_amd_download(_download_arc, _ctx.clone());
                        }
                    }
                    _ => {}
                }
            }
        });
    });
}

/// Запускає завантаження whisper-amd у фоновому потоці (тільки Windows).
#[cfg(target_os = "windows")]
fn start_whisper_amd_download(download: Arc<Mutex<BinaryDownload>>, ctx: egui::Context) {
    *download.lock().unwrap() = BinaryDownload::Downloading("підготовка...".to_string());
    ctx.request_repaint();

    std::thread::spawn(move || {
        let dl = Arc::clone(&download);
        let ctx2 = ctx.clone();

        let result = crate::bundle::download_whisper_amd(move |label| {
            *dl.lock().unwrap() = BinaryDownload::Downloading(label);
            ctx2.request_repaint();
        });

        match result {
            Ok(()) => *download.lock().unwrap() = BinaryDownload::Done,
            Err(e) => *download.lock().unwrap() = BinaryDownload::Failed(e),
        }
        ctx.request_repaint();
    });
}

/// Рядок для бінарника з авто-завантаженням — відображає прогрес скачування.
fn draw_download_row(
    ui: &mut egui::Ui,
    name: &str,
    status: &ToolStatus,
    download: &BinaryDownload,
    description: &str,
    language: Language,
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);

        match (status, download) {
            (ToolStatus::Installed(_), _) => {
                ui.label(egui::RichText::new("✓").color(egui::Color32::from_rgb(46, 204, 113)).size(16.0).strong());
            }
            (_, BinaryDownload::Downloading(_)) => {
                ui.spinner();
            }
            (_, BinaryDownload::Failed(_)) => {
                ui.label(egui::RichText::new("✗").color(egui::Color32::from_rgb(231, 76, 60)).size(16.0).strong());
            }
            (ToolStatus::Checking, _) => {
                ui.spinner();
            }
            _ => {
                ui.label(egui::RichText::new("✗").color(egui::Color32::from_rgb(231, 76, 60)).size(16.0).strong());
            }
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).strong());

                match (status, download) {
                    (ToolStatus::Installed(ver), _) => {
                        ui.label(egui::RichText::new(ver).color(egui::Color32::from_rgb(46, 204, 113)).size(11.0));
                    }
                    (_, BinaryDownload::Downloading(label)) => {
                        ui.label(egui::RichText::new(label).color(egui::Color32::from_rgb(255, 200, 0)).size(11.0));
                    }
                    (_, BinaryDownload::Done) => {
                        ui.label(egui::RichText::new(translate(language, "welcome_checking")).weak().size(11.0));
                    }
                    (_, BinaryDownload::Failed(err)) => {
                        ui.label(egui::RichText::new(err).color(egui::Color32::from_rgb(231, 76, 60)).size(11.0));
                    }
                    (ToolStatus::Checking, _) => {
                        ui.label(egui::RichText::new(translate(language, "welcome_checking")).weak().size(11.0));
                    }
                    _ => {
                        ui.label(egui::RichText::new(translate(language, "welcome_not_installed")).color(egui::Color32::from_rgb(231, 76, 60)).size(11.0));
                    }
                }
            });
            ui.label(egui::RichText::new(description).weak().size(11.0));
        });
    });
}
