use eframe::egui;

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
}

impl Default for VideoMakerApp {
    fn default() -> Self {
        Self {
            active_tab: Tab::Main,
        }
    }
}

impl VideoMakerApp {
    /// Створює новий екземпляр додатку.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Тут в майбутньому можна ініціалізувати стан, завантажувати налаштування тощо.
        Self::default()
    }
}

impl eframe::App for VideoMakerApp {
    /// Викликається кожного разу, коли інтерфейс потребує оновлення та перемальовування.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Верхня панель для навігації між вкладками
        egui::TopBottomPanel::top("navigation_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("🎬 Soloveyko.ai Video Maker");
                ui.separator();
                
                // Рядок вибору вкладок з сучасним виглядом
                ui.selectable_value(&mut self.active_tab, Tab::Main, "Основна");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "Налаштування");
            });
        });

        // Центральна панель для відображення вмісту активної вкладки
        egui::CentralPanel::default().show(ctx, |ui| {
            // Відображаємо контент залежно від обраної вкладки
            match self.active_tab {
                Tab::Main => {
                    ui.heading("Основна вкладка");
                    ui.label("Поки що тут пусто. Ця вкладка буде містити основний інструментарій автоматизації відео.");
                }
                Tab::Settings => {
                    ui.heading("Налаштування");
                    ui.label("Поки що тут пусто. Ця вкладка буде містити конфігурацію додатку та параметри генерації.");
                }
            }
        });
    }
}
