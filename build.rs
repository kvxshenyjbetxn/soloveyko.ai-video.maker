fn main() {
    // Вбудовуємо іконку в .exe для Windows (відображається в Explorer та Taskbar)
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("video.maker.ico");
        res.compile().expect("Не вдалося скомпілювати ресурси Windows");
    }
}
