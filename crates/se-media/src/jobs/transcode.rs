//! The mezzanine and proxy job: one ffmpeg run writing the plan's output under a temporary name
//! (`<stem>.part.<ext>`, so ffmpeg still picks the format from the extension), renamed when
//! complete. A hardware encoder is tried first when one works for the target codec; if it fails
//! (driver, resolution or depth it does not support), the job encodes in software.

use std::path::{Path, PathBuf};

use se_app::ports::{MediaError, MediaJobSpec, ProgressSink, TranscodePlan, VideoPlan};

use super::{
    ffmpeg_progress::{fraction, out_time_seconds},
    transcode_args::transcode_args,
};
use crate::{
    encoders::{EncoderSupport, HardwareEncoder},
    process::Sidecar,
};

/// Writes `plan.output` from `spec.input` with the ffmpeg at `ffmpeg`.
///
/// # Errors
/// [`MediaError`] when ffmpeg fails in software too, or the output cannot be written.
pub async fn generate(
    ffmpeg: &Path,
    support: EncoderSupport,
    spec: &MediaJobSpec,
    plan: &TranscodePlan,
    progress: &dyn ProgressSink,
) -> Result<(), MediaError> {
    let io = |error: std::io::Error| MediaError::Io {
        path: plan.output.clone(),
        message: error.to_string(),
    };
    if let Some(dir) = plan.output.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(io)?;
    }
    let temporary = temporary_output(&plan.output);
    let hardware = match &plan.recipe.video {
        VideoPlan::Encode(encoding) => support.for_codec(encoding.codec),
        VideoPlan::Drop | VideoPlan::Copy | VideoPlan::Still => None,
    };
    let mut done = false;
    if let Some(encoder) = hardware {
        match run(ffmpeg, spec, plan, Some(encoder), &temporary, progress).await {
            Ok(()) => done = true,
            Err(error) => tracing::warn!(
                %error,
                encoder = encoder.name(),
                "hardware encoding failed; encoding in software"
            ),
        }
    }
    if !done {
        run(ffmpeg, spec, plan, None, &temporary, progress).await?;
    }
    tokio::fs::rename(&temporary, &plan.output)
        .await
        .map_err(io)?;
    progress.report(1.0);
    Ok(())
}

async fn run(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    plan: &TranscodePlan,
    hardware: Option<HardwareEncoder>,
    temporary: &Path,
    progress: &dyn ProgressSink,
) -> Result<(), MediaError> {
    let args = transcode_args(spec, plan, hardware, temporary);
    let duration = spec
        .media_info
        .duration
        .map(se_domain::Flicks::to_seconds_f64);
    Sidecar::spawn(ffmpeg, &args, None)?
        .run_with_lines(|line| {
            if let Some(done) =
                out_time_seconds(line).and_then(|seconds| fraction(seconds, duration))
            {
                progress.report(done.min(0.99));
            }
        })
        .await
}

/// `<dir>/<stem>.part.<ext>`: a name no reader takes for the finished file.
fn temporary_output(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default().to_string_lossy();
    let name = match output.extension() {
        Some(extension) => format!("{stem}.part.{}", extension.to_string_lossy()),
        None => format!("{stem}.part"),
    };
    output.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_outputs_keep_the_extension() {
        assert_eq!(
            temporary_output(Path::new("cache").join("mezzanine.mp4").as_path()),
            Path::new("cache").join("mezzanine.part.mp4")
        );
    }
}
