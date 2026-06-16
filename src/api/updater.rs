use eframe::egui;
use std::sync::{Arc, Mutex};

const REPO: &str = "kvxshenyjbetxn/repo.releases";

/// Інформація про доступне оновлення.
#[derive(Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub changelog: String,
    pub download_url: String,
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(serde::Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Порівнює версії у форматі "vX.Y.Z" або "X.Y.Z". Повертає true якщо remote новіша.
fn is_newer(remote: &str, local: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let s = v.trim_start_matches('v');
        let mut parts = s.split('.').filter_map(|p| p.parse::<u32>().ok());
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    };
    parse(remote) > parse(local)
}

/// Відкриває URL у браузері за замовчуванням (крос-платформно).
pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/c", "start", "", url]);
        crate::bundle::set_no_window(&mut cmd);
        let _ = cmd.spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Фоново перевіряє GitHub на нові релізи. Якщо є новіший — записує UpdateInfo в `result`.
pub fn check_for_updates(result: Arc<Mutex<Option<UpdateInfo>>>, ctx: egui::Context) {
    let current = crate::APP_VERSION;
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(15))
            .build();

        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            REPO
        );

        let release: GithubRelease = match agent
            .get(&url)
            .set("User-Agent", "soloveyko-ai-video-maker")
            .call()
        {
            Ok(resp) => match resp.into_json() {
                Ok(r) => r,
                Err(_) => return,
            },
            Err(_) => return,
        };

        if !is_newer(&release.tag_name, current) {
            return;
        }

        #[cfg(target_os = "windows")]
        let download_url = release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".exe"))
            .map(|a| a.browser_download_url.clone())
            .unwrap_or_else(|| format!("https://github.com/{}/releases/latest", REPO));

        #[cfg(target_os = "macos")]
        let download_url = release
            .assets
            .iter()
            .find(|a| !a.name.ends_with(".exe"))
            .map(|a| a.browser_download_url.clone())
            .unwrap_or_else(|| format!("https://github.com/{}/releases/latest", REPO));

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let download_url = format!("https://github.com/{}/releases/latest", REPO);

        *result.lock().unwrap() = Some(UpdateInfo {
            version: release.tag_name,
            changelog: release.body.unwrap_or_default(),
            download_url,
        });

        ctx.request_repaint();
    });
}
