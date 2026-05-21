use crate::gui;
use crate::gui::settings::storage::{AppSettings, load_settings, save_settings};
use eframe::egui;
use crate::theme::AppTheme;
use crate::localization::{Language, translate};

/// Перерахування для представлення доступних вкладок програми.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Основна вкладка
    Main,
    /// Вкладка налаштувань
    Settings,
}

/// Головна структура нашого GUI додатку, що зберігає його поточний стан.
pub struct VideoMakerApp {
    /// Поточна активна вкладка програми.
    active_tab: Tab,
    /// Текст сценарію, який вводить користувач.
    text_input: String,
    /// Поточна обрана тема оформлення додатку.
    theme: AppTheme,
    /// Обраний акцентний колір для елементів інтерфейсу.
    accent_color: egui::Color32,
    /// Поточна збережена ширина бічної панелі пайплайну.
    pipeline_width: f32,
    /// Поточна обрана підвкладка налаштувань.
    active_settings_tab: gui::settings::SettingsSubTab,
    /// Поточна вибрана мова інтерфейсу програми.
    language: Language,
    /// Копія останніх збережених налаштувань на диску для відстеження змін у реальному часі.
    last_saved_settings: AppSettings,
}

impl Default for VideoMakerApp {
    fn default() -> Self {
        Self {
            active_tab: Tab::Main,
            text_input: String::new(),
            theme: AppTheme::Dark, // Сучасна темна тема за замовчуванням
            accent_color: egui::Color32::from_rgb(0, 122, 255), // Синій колір за замовчуванням
            pipeline_width: 450.0,
            active_settings_tab: gui::settings::SettingsSubTab::General,
            language: Language::Uk,
            last_saved_settings: AppSettings::default(),
        }
    }
}

impl VideoMakerApp {
    /// Створює новий екземпляр додатку, завантажуючи збережені налаштування з диска.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Завантажуємо збережені налаштування користувача з файлу settings.json
        let saved = load_settings();
        
        // Конвертуємо назву теми (String) у тип AppTheme
        let theme = match saved.theme.as_str() {
            "Light" => AppTheme::Light,
            "Amoled" => AppTheme::Amoled,
            _ => AppTheme::Dark,
        };
        
        // Відновлюємо колір акценту з масиву [r, g, b, a]
        let accent_color = egui::Color32::from_rgba_unmultiplied(
            saved.accent_color[0],
            saved.accent_color[1],
            saved.accent_color[2],
            saved.accent_color[3],
        );

        let pipeline_width = saved.pipeline_width;

        // Конвертуємо назву мови (String) у тип Language
        let language = match saved.language.as_str() {
            "En" => Language::En,
            "Ru" => Language::Ru,
            _ => Language::Uk,
        };

        Self {
            active_tab: Tab::Main,
            text_input: String::new(),
            theme,
            accent_color,
            pipeline_width,
            active_settings_tab: gui::settings::SettingsSubTab::General,
            language,
            last_saved_settings: saved,
        }
    }
}

impl eframe::App for VideoMakerApp {
    /// Викликається кожного разу, коли інтерфейс потребує оновлення та перемальовування.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Динамічно застосовуємо обрану тему оформлення та акцентний колір до поточного контексту
        crate::theme::apply_theme(ctx, self.theme, self.accent_color);

        // Верхня панель для навігації між вкладками
        egui::TopBottomPanel::top("navigation_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(translate(self.language, "app_title"));
                ui.separator();
                
                // Рядок вибору вкладок з сучасним виглядом
                ui.selectable_value(&mut self.active_tab, Tab::Main, translate(self.language, "tab_main"));
                ui.selectable_value(&mut self.active_tab, Tab::Settings, translate(self.language, "tab_settings"));
            });
        });

        // Відображаємо бічну панель пайплайну ТІЛЬКИ на вкладці "Основна"
        if self.active_tab == Tab::Main {
            let panel_res = egui::SidePanel::right("pipeline_panel")
                .default_width(self.pipeline_width)  // Встановлюємо збережену ширину панелі
                .width_range(350.0..=750.0)       // Збільшений діапазон зміни ширини
                .resizable(true)
                .show(ctx, |ui| {
                    gui::pipeline::draw_pipeline_panel(ui, self.language);
                });

            // Зчитуємо фактичну ширину панелі після її рендерингу/перетягування користувачем
            let actual_width = panel_res.response.rect.width();
            
            // Якщо ширина змінилася більше ніж на 1 піксель, оновлюємо стан у пам'яті
            if (actual_width - self.pipeline_width).abs() > 1.0 {
                self.pipeline_width = actual_width;
            }
        }

        // Конфігуруємо фрейм для центральної панелі.
        // Для редактора (Main) прибираємо відступи (margin), щоб поле було на всю висоту та ширину.
        // Для вкладки налаштувань (Settings) залишаємо стандартні відступи.
        let frame = if self.active_tab == Tab::Main {
            egui::Frame::none()
                .fill(ctx.style().visuals.panel_fill) // Насичуємо фоновим кольором поточної теми
                .inner_margin(egui::Margin::same(0.0))
        } else {
            egui::Frame::central_panel(ctx.style().as_ref())
                .stroke(egui::Stroke::NONE)
        };

        // Central Panel
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                match self.active_tab {
                    Tab::Main => {
                        gui::editor::draw_editor(ui, &mut self.text_input, self.language);
                    }
                    Tab::Settings => {
                        gui::settings::draw_settings(
                            ui,
                            &mut self.theme,
                            &mut self.accent_color,
                            &mut self.active_settings_tab,
                            &mut self.language,
                        );
                    }
                }
            });

        // АВТОЗБЕРЕЖЕННЯ:
        // Перевіряємо, чи користувач наразі не перетягує панель (миша відпущена).
        // Це запобігає надмірному навантаженню на диск та гарантує запис файлу лише після відпускання миші.
        let is_pointer_down = ctx.input(|i| i.pointer.any_down());
        
        if !is_pointer_down {
            let current_theme_str = match self.theme {
                AppTheme::Light => "Light".to_string(),
                AppTheme::Dark => "Dark".to_string(),
                AppTheme::Amoled => "Amoled".to_string(),
            };
            let current_color_arr = self.accent_color.to_array();
            let current_language_str = match self.language {
                Language::En => "En".to_string(),
                Language::Ru => "Ru".to_string(),
                _ => "Uk".to_string(),
            };

            // Перевіряємо зміни значень (з дельтою для ширини панелі)
            if current_theme_str != self.last_saved_settings.theme 
                || current_color_arr != self.last_saved_settings.accent_color 
                || current_language_str != self.last_saved_settings.language
                || (self.pipeline_width - self.last_saved_settings.pipeline_width).abs() > 1.0
            {
                let new_settings = AppSettings {
                    theme: current_theme_str,
                    accent_color: current_color_arr,
                    pipeline_width: self.pipeline_width,
                    language: current_language_str,
                };
                
                // Зберігаємо оновлені налаштування у файл JSON на диску
                save_settings(&new_settings);
                
                // Оновлюємо копію останніх збережених параметрів у пам'яті
                self.last_saved_settings = new_settings;
            }
        }
    }
}
