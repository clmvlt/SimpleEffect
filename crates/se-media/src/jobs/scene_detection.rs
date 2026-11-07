//! The scene detection job: ffmpeg's scene score on a small copy of the picture; frames above the
//! threshold are printed by `showinfo` to stderr and parsed ([`parse_scene_changes`]).
//!
//! The filter graph splits the picture: one branch selects and prints shot changes, the other
//! feeds every frame to the null output, so `-progress` advances smoothly.

use std::path::Path;

use se_app::ports::{MediaError, MediaJobSpec, ProgressSink};
use se_domain::SceneDetection;

use super::ffmpeg_progress::{fraction, out_time_seconds};
use crate::{analysis::parse_scene_changes, process::Sidecar};

/// Width of the picture the scene score is computed on (fast, and enough to see a cut).
const ANALYSIS_WIDTH: u32 = 320;

/// The `-filter_complex` graph of a detection at `threshold`.
#[must_use]
pub fn scene_filter(threshold: f64) -> String {
    format!(
        "[0:v:0]scale={ANALYSIS_WIDTH}:-2,split[scenes][all];\
         [scenes]select='gt(scene,{threshold:.3})',showinfo,nullsink;[all]null[out]"
    )
}

/// Detects the shot changes of `spec` with the ffmpeg at `ffmpeg`.
///
/// # Errors
/// [`MediaError`] when ffmpeg fails.
pub async fn detect(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    threshold: f64,
    progress: &dyn ProgressSink,
) -> Result<SceneDetection, MediaError> {
    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-nostats", "-v", "info"]
        .map(str::to_owned)
        .to_vec();
    args.extend(["-progress".to_owned(), "pipe:1".to_owned(), "-i".to_owned()]);
    args.push(spec.input.to_string_lossy().into_owned());
    args.push("-filter_complex".to_owned());
    args.push(scene_filter(threshold));
    args.extend(["-map", "[out]", "-an", "-sn", "-dn", "-f", "null", "-"].map(str::to_owned));
    let duration = spec
        .media_info
        .duration
        .map(se_domain::Flicks::to_seconds_f64);
    let stderr = Sidecar::spawn(ffmpeg, &args, None)?
        .run_with_lines_keeping_stderr(|line| {
            if let Some(done) =
                out_time_seconds(line).and_then(|seconds| fraction(seconds, duration))
            {
                progress.report(done.min(0.99));
            }
        })
        .await?;
    progress.report(1.0);
    Ok(SceneDetection {
        threshold,
        changes: parse_scene_changes(&stderr),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_graph_scores_a_small_copy_and_keeps_every_frame_for_progress() {
        assert_eq!(
            scene_filter(0.3),
            "[0:v:0]scale=320:-2,split[scenes][all];[scenes]select='gt(scene,0.300)',showinfo,nullsink;[all]null[out]"
        );
    }
}
