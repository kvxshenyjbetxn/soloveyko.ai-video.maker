use eframe::egui;

/// Куровані шрифти для субтитрів: (відображувана назва, файлові імена для пошуку).
/// Файлові імена перераховані від найбільш вірогідного до резервного.
pub const SUBTITLE_FONTS: &[(&str, &[&str])] = &[
    // Sans-serif
    ("Arial", &["Arial.ttf", "arial.ttf"]),
    ("Arial Bold", &["Arial Bold.ttf", "arialbd.ttf"]),
    ("Arial Narrow", &["Arial Narrow.ttf", "arialn.ttf"]),
    ("Calibri", &["Calibri.ttf", "calibri.ttf"]),
    ("Calibri Bold", &["Calibri Bold.ttf", "calibrib.ttf"]),
    ("Helvetica", &["Helvetica.ttf", "Helvetica.ttc"]),
    (
        "Helvetica Neue",
        &["Helvetica Neue.ttf", "HelveticaNeue.ttc"],
    ),
    ("Gill Sans", &["Gill Sans.ttf", "GillSans.ttc"]),
    ("Optima", &["Optima.ttc", "Optima.ttf"]),
    ("Futura", &["Futura.ttc", "Futura.ttf"]),
    ("Tahoma", &["Tahoma.ttf", "tahoma.ttf"]),
    ("Verdana", &["Verdana.ttf", "verdana.ttf"]),
    ("Trebuchet MS", &["Trebuchet MS.ttf", "trebuc.ttf"]),
    ("Segoe UI", &["segoeui.ttf", "SegoeUI.ttf"]),
    ("Segoe UI Bold", &["segoeuib.ttf", "SegoeUI-Bold.ttf"]),
    ("Franklin Gothic", &["framd.ttf", "FRADM.TTF"]),
    ("Century Gothic", &["GOTHIC.TTF", "gothic.ttf"]),
    // Display / Impact-style
    ("Impact", &["Impact.ttf", "impact.ttf"]),
    ("Oswald", &["Oswald-Regular.ttf", "Oswald.ttf"]),
    ("Anton", &["Anton-Regular.ttf", "Anton.ttf"]),
    // Serif
    ("Georgia", &["Georgia.ttf", "georgia.ttf"]),
    ("Times New Roman", &["Times New Roman.ttf", "times.ttf"]),
    ("Palatino", &["Palatino.ttc", "palat.ttf"]),
    ("Garamond", &["Garamond.ttf", "GARAM.TTF"]),
    ("Book Antiqua", &["BKANT.TTF", "bookant.ttf"]),
    ("Cambria", &["cambria.ttc", "Cambria.ttf"]),
    ("Didot", &["Didot.ttc", "Didot.ttf"]),
    ("Baskerville", &["Baskerville.ttc", "Baskerville.ttf"]),
    // Monospace
    ("Courier New", &["Courier New.ttf", "cour.ttf"]),
    ("Consolas", &["consola.ttf", "Consolas.ttf"]),
    ("Lucida Console", &["lucon.ttf", "LucidaConsole.ttf"]),
    // Handwriting / Decorative
    ("Comic Sans MS", &["Comic Sans MS.ttf", "comic.ttf"]),
    (
        "Brush Script MT",
        &["Brush Script MT Italic.ttf", "brushsci.ttf"],
    ),
    ("Papyrus", &["Papyrus.ttc", "Papyrus.ttf"]),
];

/// Повертає список директорій де шукати системні шрифти (платформо-залежно).
fn system_font_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // macOS 10.15+: шрифти третіх сторін переїхали в Supplemental
        dirs.push("/System/Library/Fonts/Supplemental".into());
        dirs.push("/System/Library/Fonts".into());
        dirs.push("/Library/Fonts".into());
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(std::path::Path::new(&home).join("Library/Fonts"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        dirs.push("C:\\Windows\\Fonts".into());
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(
                std::path::Path::new(&local)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts"),
            );
        }
    }

    dirs
}

/// Шукає перший існуючий файл шрифту у системних директоріях та повертає його байти.
fn try_load_font(file_names: &[&str]) -> Option<Vec<u8>> {
    for dir in system_font_dirs() {
        for name in file_names {
            let path = dir.join(name);
            if let Ok(bytes) = std::fs::read(&path) {
                return Some(bytes);
            }
        }
    }
    None
}

/// Завантажує знайдені системні шрифти у egui FontDefinitions.
/// Повертає список назв шрифтів, які успішно завантажені.
///
/// Важливо: перезаписує FontDefinitions, тому має викликатись один раз при старті
/// до будь-якого іншого `ctx.set_fonts()`.
pub fn load_subtitle_fonts(ctx: &egui::Context) -> Vec<String> {
    let mut font_defs = egui::FontDefinitions::default();
    let mut loaded: Vec<String> = Vec::new();

    for (name, file_names) in SUBTITLE_FONTS {
        if let Some(bytes) = try_load_font(file_names) {
            font_defs.font_data.insert(
                (*name).to_string(),
                egui::FontData::from_owned(bytes).into(),
            );
            font_defs.families.insert(
                egui::FontFamily::Name((*name).into()),
                // Основний шрифт + fallback на стандартний egui щоб кирилиця читалась
                vec![(*name).to_string(), "Hack".to_string()],
            );
            loaded.push((*name).to_string());
        }
    }

    ctx.set_fonts(font_defs);
    loaded
}
