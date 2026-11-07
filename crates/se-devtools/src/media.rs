//! Test media made with ffmpeg: a few short videos with sound and two sound files, enough to
//! give a large project real thumbnails and waveforms.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::DevError;

/// One file to generate: its name and the ffmpeg arguments after `-y`.
struct MediaRecipe {
    name: &'static str,
    inputs: Vec<String>,
    outputs: Vec<&'static str>,
}

/// A test-pattern video of `seconds` with a tone at `frequency` Hz, its hue turned by `hue`.
fn video(name: &'static str, seconds: u32, frequency: u32, hue: u32) -> MediaRecipe {
    MediaRecipe {
        name,
        inputs: vec![
            "-f".into(),
            "lavfi".into(),
            "-i".into(),
            format!("testsrc2=size=1280x720:rate=25:duration={seconds},hue=h={hue}"),
            "-f".into(),
            "lavfi".into(),
            "-i".into(),
            format!("aevalsrc='0.5*sin(2*PI*{frequency}*t)*abs(sin(PI*t/2))':s=48000:d={seconds}"),
        ],
        outputs: vec![
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
        ],
    }
}

/// A sound file of `seconds` from the lavfi source `source`.
fn sound(name: &'static str, source: String) -> MediaRecipe {
    MediaRecipe {
        name,
        inputs: vec!["-f".into(), "lavfi".into(), "-i".into(), source],
        outputs: vec!["-ar", "48000", "-ac", "2"],
    }
}

fn recipes() -> Vec<MediaRecipe> {
    vec![
        video("Plage.mp4", 30, 220, 0),
        video("Forêt.mp4", 24, 330, 90),
        video("Ville.mp4", 40, 440, 180),
        video("Interview.mp4", 60, 180, 270),
        sound(
            "Musique.wav",
            "anoisesrc=color=pink:amplitude=0.25:duration=90,volume='0.4+0.6*abs(sin(PI*t/8))':eval=frame"
                .to_owned(),
        ),
        sound(
            "Voix off.wav",
            "aevalsrc='0.7*sin(2*PI*160*t)*gt(sin(PI*t*1.3),0)*abs(sin(PI*t*5))':s=48000:d=45"
                .to_owned(),
        ),
    ]
}

/// Generates the test media in `dir` (skipping files already there) and returns their paths.
///
/// # Errors
/// ffmpeg cannot be run or fails.
pub fn generate_media(ffmpeg: &Path, dir: &Path) -> Result<Vec<PathBuf>, DevError> {
    std::fs::create_dir_all(dir).map_err(|error| DevError::io(dir, &error))?;
    let mut paths = Vec::new();
    for recipe in recipes() {
        let path = dir.join(recipe.name);
        if !path.is_file() {
            let status = Command::new(ffmpeg)
                .args(["-hide_banner", "-loglevel", "error", "-y"])
                .args(&recipe.inputs)
                .args(&recipe.outputs)
                .arg(&path)
                .status()
                .map_err(|error| DevError::io(ffmpeg, &error))?;
            if !status.success() {
                return Err(DevError::Ffmpeg {
                    file: recipe.name.to_owned(),
                });
            }
        }
        paths.push(path);
    }
    Ok(paths)
}
