//! Shared helpers: the repository's tools, media generation, specs and a progress recorder.

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use se_app::ports::{MediaJobSpec, MediaProber, MediaTask, ProbedMedia, ProgressSink};
use se_media::{FfmpegJobRunner, FfmpegTools, FfprobeProber, Tool, ToolLocator, process::Sidecar};

/// The repository's tools (`resources/bin/<platform>/`), or `None` with a message.
pub fn tools() -> Option<ToolLocator> {
    let locator = ToolLocator::new(None, Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))), false);
    if locator.locate(Tool::Ffmpeg).is_err() || locator.locate(Tool::Ffprobe).is_err() {
        eprintln!("skipped: ffmpeg/ffprobe not installed (run `pnpm fetch-binaries`)");
        return None;
    }
    Some(locator)
}

/// A job runner over `tools`.
pub fn runner(tools: &ToolLocator) -> FfmpegJobRunner {
    FfmpegJobRunner::new(FfmpegTools::new(tools.clone()))
}

/// Generates a media file with ffmpeg's lavfi sources.
pub async fn generate(tools: &ToolLocator, dir: &Path, name: &str, args: &[&str]) -> PathBuf {
    let output = dir.join(name);
    let ffmpeg = tools.locate(Tool::Ffmpeg).unwrap();
    let mut all: Vec<String> = ["-hide_banner", "-v", "error", "-y"]
        .map(str::to_owned)
        .to_vec();
    all.extend(args.iter().map(|arg| (*arg).to_owned()));
    all.push(output.to_string_lossy().into_owned());
    Sidecar::spawn(&ffmpeg, &all, None)
        .unwrap()
        .finish()
        .await
        .unwrap();
    output
}

/// Probes `path` with the repository's ffprobe.
pub async fn probe(tools: &ToolLocator, path: &Path) -> ProbedMedia {
    FfprobeProber::new(tools.clone()).probe(path).await.unwrap()
}

/// The spec of `task` on `input`, whose cache folder is `<dir>/cache`.
pub fn spec(task: MediaTask, input: &Path, dir: &Path, probed: &ProbedMedia) -> MediaJobSpec {
    MediaJobSpec {
        task,
        input: input.to_path_buf(),
        output_dir: dir.join("cache"),
        asset_kind: probed.kind,
        media_info: probed.info.clone(),
    }
}

/// Records every progress report.
#[derive(Default)]
pub struct Progress(pub Mutex<Vec<f64>>);

impl Progress {
    /// The last fraction reported.
    pub fn last(&self) -> Option<f64> {
        self.0.lock().unwrap().last().copied()
    }
}

impl ProgressSink for Progress {
    fn report(&self, fraction: f64) {
        self.0.lock().unwrap().push(fraction);
    }
}
