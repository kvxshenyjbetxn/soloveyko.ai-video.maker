use std::path::{Path, PathBuf};
use std::time::Instant;

/// Стабільний хеш шляху → ім'я папки кешу (не змінюється між запусками)
pub fn path_hash(path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    format!("{:x}", h.finish())
}

fn clean_windows_path(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy();
        if s.starts_with(r"\\?\") {
            return PathBuf::from(&s[4..]);
        }
    }
    path.to_path_buf()
}

/// Отримує тривалість медіафайлу через ffprobe
pub fn probe_duration(path: &Path) -> Option<f32> {
    let clean_path = clean_windows_path(path);
    let mut ffprobe_cmd = std::process::Command::new(crate::bundle::ffprobe_path());
    ffprobe_cmd.args(["-v", "error", "-show_entries", "format=duration",
           "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(&clean_path);
    crate::bundle::set_no_window(&mut ffprobe_cmd);
    let out = ffprobe_cmd.output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<f32>().ok()
}

pub fn uuid_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}-{:x}", t.as_nanos(), rand_u32())
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    Instant::now().hash(&mut h);
    h.finish() as u32
}
