//! The speech audio of a media file for whisper.cpp (the `SpeechAudioExtractor` port): its first
//! sound stream as 16 kHz mono 16-bit PCM WAV, written as `<stem>.part.wav` then renamed.

use std::path::{Path, PathBuf};

use se_app::ports::MediaError;

use crate::process::Sidecar;

/// Writes `source`'s first sound stream to `output` with the ffmpeg at `ffmpeg`.
///
/// # Errors
/// [`MediaError`] when the file has no sound or ffmpeg fails.
pub async fn extract(ffmpeg: &Path, source: &Path, output: &Path) -> Result<(), MediaError> {
    let io = |error: std::io::Error| MediaError::Io {
        path: output.to_path_buf(),
        message: error.to_string(),
    };
    if let Some(dir) = output.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(io)?;
    }
    let temporary = temporary_wav(output);
    Sidecar::spawn(ffmpeg, arguments(source, &temporary), None)?
        .finish()
        .await?;
    tokio::fs::rename(&temporary, output).await.map_err(io)
}

/// ffmpeg's arguments: no video, subtitles or data; the first sound stream down-mixed to mono and
/// resampled to 16 kHz (what whisper.cpp reads natively).
#[must_use]
pub fn arguments(source: &Path, output: &Path) -> Vec<String> {
    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-v", "error", "-y", "-i"]
        .map(str::to_owned)
        .to_vec();
    args.push(source.to_string_lossy().into_owned());
    args.extend(
        [
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ]
        .map(str::to_owned),
    );
    args.push(output.to_string_lossy().into_owned());
    args
}

/// `<dir>/<stem>.part.wav`.
fn temporary_wav(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default().to_string_lossy();
    output.with_file_name(format!("{stem}.part.wav"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_for_mono_16_khz_pcm_of_the_first_sound_stream() {
        let args = arguments(Path::new("in.mov"), Path::new("out.part.wav"));
        let line = args.join(" ");
        assert!(line.contains("-map 0:a:0 -vn -sn -dn -ac 1 -ar 16000 -c:a pcm_s16le -f wav"));
        assert_eq!(args.last().map(String::as_str), Some("out.part.wav"));
        assert_eq!(
            temporary_wav(Path::new("dir/speech-16k.wav")),
            Path::new("dir/speech-16k.part.wav")
        );
    }
}
