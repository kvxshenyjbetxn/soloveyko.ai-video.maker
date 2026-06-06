use std::path::{Path, PathBuf};

pub struct AudioPlayer {
    _stream: rodio::OutputStream,
    // Sink зберігається щоб утримувати відтворення живим; дроп зупиняє аудіо
    _sink: rodio::Sink,
}

impl AudioPlayer {
    /// Відкриває аудіо-файл і починає відтворення з позиції `start_secs`.
    pub fn start(path: &Path, start_secs: f32) -> Option<Self> {
        let (stream, handle) = rodio::OutputStream::try_default().ok()?;
        let sink = rodio::Sink::try_new(&handle).ok()?;
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
