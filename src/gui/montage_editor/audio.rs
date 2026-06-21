use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct AudioPlayer {
    _stream: rodio::OutputStream,
    // Sink зберігається щоб утримувати відтворення живим; дроп зупиняє аудіо
    _sink: rodio::Sink,
}

impl AudioPlayer {
    /// Відкриває аудіо-файл і починає відтворення з позиції `start_secs`.
    /// `volume` — коефіцієнт гучності (1.0 = норма, 0.0 = тиша, 2.0 = +100%).
    pub fn start(path: &Path, start_secs: f32, volume: f32) -> Option<Self> {
        let (stream, handle) = rodio::OutputStream::try_default().ok()?;
        let sink = rodio::Sink::try_new(&handle).ok()?;
        sink.set_volume(volume.max(0.0));
        let file = std::fs::File::open(path).ok()?;
        let decoder = rodio::Decoder::new(std::io::BufReader::new(file)).ok()?;
        use rodio::Source;
        if start_secs > 0.05 {
            sink.append(decoder.skip_duration(std::time::Duration::from_secs_f32(start_secs)));
        } else {
            sink.append(decoder);
        }
        Some(Self { _stream: stream, _sink: sink })
    }
}

#[allow(dead_code)]
pub struct PlayingAudio {
    pub path: PathBuf,
    pub start_secs: f32,
    pub duration: f32,
    pub player: AudioPlayer,
}

// ─── Кеш аудіо вбудованого у відеофайли ─────────────────────────────────────

/// Шлях до WAV-кешу для вбудованого аудіо відеофайлу.
pub fn embedded_audio_cache_path(video_path: &Path, save_path: &Path) -> PathBuf {
    let mut h = DefaultHasher::new();
    video_path.hash(&mut h);
    save_path.join(".audio_cache").join(format!("{:x}.wav", h.finish()))
}

/// Асинхронно витягує аудіо з відеофайлу у WAV (pcm_s16le, 44100 Hz, stereo).
/// Якщо кеш вже існує — нічого не робить.
pub fn extract_embedded_audio_async(video_path: PathBuf, save_path: PathBuf) {
    let out_path = embedded_audio_cache_path(&video_path, &save_path);
    if out_path.exists() { return; }
    std::thread::spawn(move || {
        std::fs::create_dir_all(save_path.join(".audio_cache")).ok();
        let mut cmd = std::process::Command::new(crate::bundle::ffmpeg_path());
        cmd.args(["-y", "-v", "error", "-i"])
            .arg(&video_path)
            .args(["-vn", "-acodec", "pcm_s16le", "-ar", "44100", "-ac", "2"])
            .arg(&out_path);
        crate::bundle::set_no_window(&mut cmd);
        let _ = crate::api::ffmpeg::run_tracked(&mut cmd);
    });
}
