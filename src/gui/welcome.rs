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
    pub gemini: Arc<Mutex<ToolStatus>>,
    pub claude: Arc<Mutex<ToolStatus>>,
    pub ffmpeg: Arc<Mutex<ToolStatus>>,
    /// Перевіряється тільки на macOS, на Windows — завжди Installed.
    pub brew: Arc<Mutex<ToolStatus>>,
    /// Перевіряється тільки на Windows, на macOS — завжди Installed.
    pub npm: Arc<Mutex<ToolStatus>>,
}

impl ToolChecks {
    pub fn new() -> Self {
        Self {
            gemini: Arc::new(Mutex::new(ToolStatus::Checking)),
            claude: Arc::new(Mutex::new(ToolStatus::Checking)),
            ffmpeg: Arc::new(Mutex::new(ToolStatus::Checking)),
            brew: Arc::new(Mutex::new(ToolStatus::Checking)),
            npm: Arc::new(Mutex::new(ToolStatus::Checking)),
        }
    }

    /// Скидає всі статуси і перезапускає перевірки.
    pub fn restart(&self, ctx: egui::Context) {
        *self.gemini.lock().unwrap() = ToolStatus::Checking;
        *self.claude.lock().unwrap() = ToolStatus::Checking;
        *self.ffmpeg.lock().unwrap() = ToolStatus::Checking;
        *self.brew.lock().unwrap() = ToolStatus::Checking;
        *self.npm.lock().unwrap() = ToolStatus::Checking;
        self.start(ctx);
    }

    /// Запускає всі перевірки у фонових потоках.
    pub fn start(&self, ctx: egui::Context) {
        Self::check("gemini", "--version", Arc::clone(&self.gemini), ctx.clone());
        Self::check("claude", "--version", Arc::clone(&self.claude), ctx.clone());
        Self::check("ffmpeg", "-version", Arc::clone(&self.ffmpeg), ctx.clone());

        // brew — тільки macOS
        if cfg!(target_os = "macos") {
            Self::check("brew", "--version", Arc::clone(&self.brew), ctx.clone());
        } else {
            *self.brew.lock().unwrap() = ToolStatus::Installed(String::new());
        }

        // npm — тільки Windows
        if cfg!(target_os = "windows") {
            Self::check("npm", "--version", Arc::clone(&self.npm), ctx.clone());
        } else {
            *self.npm.lock().unwrap() = ToolStatus::Installed(String::new());
        }
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
}

/// Відкриває Terminal (macOS) або PowerShell (Windows) і виконує команди через тимчасовий скрипт.
fn run_in_terminal(commands: &[&str]) {
    #[cfg(target_os = "macos")]
    {
        let script = format!("#!/bin/bash\n{}\n", commands.join("\n"));
        let tmp = std::env::temp_dir().join("soloveyko_install.sh");
        if std::fs::write(&tmp, &script).is_ok() {
            let path = tmp.to_string_lossy().to_string();
            let _ = std::process::Command::new("chmod").args(["+x", &path]).status();
            let _ = std::process::Command::new("osascript")
                .args([
                    "-e", "tell application \"Terminal\" to activate",
                    "-e", &format!("tell application \"Terminal\" to do script \"bash '{}'\"", path),
                ])
                .spawn();
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script = commands.join("\r\n");
        let tmp = std::env::temp_dir().join("soloveyko_install.ps1");
        if std::fs::write(&tmp, &script).is_ok() {
            let path = tmp.to_string_lossy().to_string();
            let _ = std::process::Command::new("powershell")
                .args(["-NoExit", "-ExecutionPolicy", "Bypass", "-File", &path])
                .spawn();
        }
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

            let gemini_status = checks.gemini.lock().unwrap().clone();
            let claude_status = checks.claude.lock().unwrap().clone();
            let ffmpeg_status = checks.ffmpeg.lock().unwrap().clone();
            let brew_status = checks.brew.lock().unwrap().clone();
            let npm_status = checks.npm.lock().unwrap().clone();

            // macOS: показуємо Homebrew першим якщо brew відсутній і хоч один залежний інструмент відсутній
            let brew_needed = is_macos
                && brew_status == ToolStatus::NotInstalled
                && (gemini_status == ToolStatus::NotInstalled || ffmpeg_status == ToolStatus::NotInstalled);
            if brew_needed {
                draw_tool_row(
                    ui,
                    "Homebrew",
                    &brew_status,
                    translate(language, "welcome_brew_desc"),
                    language,
                    &[r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#],
                );
                ui.add_space(6.0);
            }

            // Windows: показуємо npm перед Gemini якщо npm відсутній
            if !is_macos && gemini_status == ToolStatus::NotInstalled {
                draw_tool_row(
                    ui,
                    "npm",
                    &npm_status,
                    translate(language, "welcome_npm_desc"),
                    language,
                    &[r#"powershell -c "irm https://community.chocolatey.org/install.ps1|iex""#],
                );
                ui.add_space(6.0);
            }

            // Gemini CLI
            // macOS: якщо brew відсутній — додаємо його встановлення першим кроком
            // Windows: якщо npm відсутній — додаємо його встановлення першим кроком
            let gemini_cmds_vec: Vec<&str> = if is_macos {
                let mut cmds = Vec::new();
                if brew_status == ToolStatus::NotInstalled {
                    cmds.push(r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#);
                }
                cmds.push("brew install gemini-cli");
                cmds
            } else {
                let mut cmds = Vec::new();
                if npm_status == ToolStatus::NotInstalled {
                    cmds.push(r#"powershell -c "irm https://community.chocolatey.org/install.ps1|iex""#);
                }
                cmds.push("npm install -g @google/gemini-cli");
                cmds
            };
            draw_tool_row(ui, "Gemini CLI", &gemini_status, translate(language, "welcome_gemini_desc"), language, &gemini_cmds_vec);

            ui.add_space(6.0);

            // Claude Code
            let claude_cmds: &[&str] = if is_macos {
                &["curl -fsSL https://claude.ai/install.sh | bash"]
            } else {
                &["irm https://claude.ai/install.ps1 | iex"]
            };
            draw_tool_row(ui, "Claude Code", &claude_status, translate(language, "welcome_claude_desc"), language, claude_cmds);

            ui.add_space(6.0);

            // FFmpeg
            // macOS: якщо brew відсутній — додаємо його встановлення першим кроком
            let ffmpeg_cmds_vec: Vec<&str> = if is_macos {
                let mut cmds = Vec::new();
                if brew_status == ToolStatus::NotInstalled {
                    cmds.push(r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#);
                }
                cmds.push("brew install ffmpeg");
                cmds
            } else {
                vec!["winget install Gyan.FFmpeg"]
            };
            draw_tool_row(ui, "FFmpeg", &ffmpeg_status, translate(language, "welcome_ffmpeg_desc"), language, &ffmpeg_cmds_vec);

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
    install_cmds: &[&str],
) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);

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

            if *status == ToolStatus::NotInstalled {
                ui.add_space(2.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(translate(language, "welcome_install_label")).size(11.0));

                    // Кнопка встановлення через Terminal/PowerShell
                    let has_terminal = cfg!(target_os = "macos") || cfg!(target_os = "windows");
                    if has_terminal {
                        let btn = egui::Button::new(
                            egui::RichText::new(translate(language, "welcome_install_btn")).size(11.0),
                        )
                        .small();
                        if ui.add(btn).clicked() {
                            run_in_terminal(install_cmds);
                        }
                    }
                });

                for cmd in install_cmds {
                    let resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!("  {}", cmd))
                                .monospace()
                                .size(11.0),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if resp.clicked() {
                        ui.output_mut(|o| o.copied_text = cmd.to_string());
                    }
                    resp.on_hover_text(translate(language, "welcome_copy_hint"));
                }
            }
        });
    });
}
