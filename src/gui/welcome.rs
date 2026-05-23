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

/// Асинхронні стани перевірки CLI-інструментів.
pub struct ToolChecks {
    pub npm: Arc<Mutex<ToolStatus>>,
    pub gemini: Arc<Mutex<ToolStatus>>,
    pub claude: Arc<Mutex<ToolStatus>>,
    pub ffmpeg: Arc<Mutex<ToolStatus>>,
}

impl ToolChecks {
    pub fn new() -> Self {
        Self {
            npm: Arc::new(Mutex::new(ToolStatus::Checking)),
            gemini: Arc::new(Mutex::new(ToolStatus::Checking)),
            claude: Arc::new(Mutex::new(ToolStatus::Checking)),
            ffmpeg: Arc::new(Mutex::new(ToolStatus::Checking)),
        }
    }

    /// Запускає всі перевірки у фонових потоках.
    pub fn start(&self, ctx: egui::Context) {
        Self::check("npm", "--version", Arc::clone(&self.npm), ctx.clone());
        Self::check("gemini", "--version", Arc::clone(&self.gemini), ctx.clone());
        Self::check("claude", "--version", Arc::clone(&self.claude), ctx.clone());
        Self::check("ffmpeg", "-version", Arc::clone(&self.ffmpeg), ctx.clone());
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
                    // Беремо лише перший рядок, щоб уникнути багатострокових виводів
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
}

/// Малює вікно привітання з перевіркою CLI-інструментів.
///
/// Повертає true, якщо щойно було натиснуто кнопку "Закрити" (щоб зовнішній код міг зберегти налаштування).
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

    let is_macos = cfg!(target_os = "macos");
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

            let npm_status = checks.npm.lock().unwrap().clone();
            let gemini_status = checks.gemini.lock().unwrap().clone();
            let claude_status = checks.claude.lock().unwrap().clone();
            let ffmpeg_status = checks.ffmpeg.lock().unwrap().clone();

            // npm
            let npm_cmds: &[&str] = if is_macos {
                &["brew install node"]
            } else {
                &["winget install OpenJS.NodeJS"]
            };
            draw_tool_row(ui, "npm", &npm_status, translate(language, "welcome_npm_desc"), language, npm_cmds);

            ui.add_space(6.0);

            // Gemini CLI
            let gemini_cmds: &[&str] = if is_macos {
                &["sudo npm install -g @google/gemini-cli"]
            } else {
                &["npm install -g @google/gemini-cli"]
            };
            draw_tool_row(ui, "Gemini CLI", &gemini_status, translate(language, "welcome_gemini_desc"), language, gemini_cmds);

            ui.add_space(6.0);

            // Claude Code
            draw_tool_row(
                ui,
                "Claude Code",
                &claude_status,
                translate(language, "welcome_claude_desc"),
                language,
                &["curl -fsSL https://claude.ai/install.sh | bash"],
            );

            ui.add_space(6.0);

            // FFmpeg
            let ffmpeg_cmds: &[&str] = if is_macos {
                &["brew install ffmpeg"]
            } else {
                &["winget install Gyan.FFmpeg"]
            };
            draw_tool_row(ui, "FFmpeg", &ffmpeg_status, translate(language, "welcome_ffmpeg_desc"), language, ffmpeg_cmds);

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.checkbox(dont_show, translate(language, "welcome_dont_show"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(translate(language, "welcome_close_btn")).clicked() {
                        *open = false;
                        closed = true;
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
    install_cmds: &[&str],
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);

        // Кольоровий індикатор встановлення
        match status {
            ToolStatus::Checking => {
                ui.spinner();
            }
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
                        ui.label(
                            egui::RichText::new(ver)
                                .color(egui::Color32::from_rgb(46, 204, 113))
                                .size(11.0),
                        );
                    }
                    ToolStatus::NotInstalled => {
                        ui.label(
                            egui::RichText::new(translate(language, "welcome_not_installed"))
                                .color(egui::Color32::from_rgb(231, 76, 60))
                                .size(11.0),
                        );
                    }
                    ToolStatus::Checking => {
                        ui.label(
                            egui::RichText::new(translate(language, "welcome_checking"))
                                .weak()
                                .size(11.0),
                        );
                    }
                }
            });

            ui.label(egui::RichText::new(description).weak().size(11.0));

            // Показуємо команди встановлення лише коли інструмент відсутній
            if *status == ToolStatus::NotInstalled {
                ui.add_space(2.0);
                ui.label(egui::RichText::new(translate(language, "welcome_install_label")).size(11.0));
                for cmd in install_cmds {
                    ui.label(
                        egui::RichText::new(format!("  $ {}", cmd))
                            .monospace()
                            .size(11.0),
                    );
                }
            }
        });
    });
}
