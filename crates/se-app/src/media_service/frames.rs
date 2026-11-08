//! Extracting single pictures of an asset (a service, not a job): AI tools ask for a frame and
//! wait for the JPEG. Results are cached per content, time and width.

use std::path::PathBuf;

use se_domain::{AssetId, AssetKind, Flicks, ProjectId};

use super::{
    MediaService, find_asset,
    media_paths::{frame_file, media_cache_dir},
};
use crate::{
    AppError,
    ports::{FrameRequest, MediaError},
};

/// Narrowest frame width served.
pub const MIN_FRAME_WIDTH: u32 = 16;
/// Widest frame width served (4K).
pub const MAX_FRAME_WIDTH: u32 = 3840;

impl MediaService {
    /// The JPEG of `asset_id`'s picture at source time `time` (ignored for images), at most
    /// `max_width` pixels wide (clamped to 16–3840, never enlarged). Extracted from the original
    /// with ffmpeg once, then served from `<cache>/media/<key>/frames/`.
    ///
    /// # Errors
    /// Not open, unknown asset, missing original, [`AppError::MediaTimeOutOfRange`] for a time
    /// outside the media, [`AppError::DerivedNotApplicable`] for sound files, ffmpeg failures.
    pub async fn extract_frame(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
        time: Flicks,
        max_width: u32,
    ) -> Result<PathBuf, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project, asset_id)?;
        let original = PathBuf::from(&asset.original_path);
        if asset.kind == AssetKind::Audio {
            return Err(AppError::DerivedNotApplicable {
                asset_id,
                derived: "frame",
                reason: "a sound file has no picture".to_owned(),
            });
        }
        if asset.missing || !self.deps.files.is_file(&original).await {
            return Err(MediaError::NotFound { path: original }.into());
        }
        let time = match asset.kind {
            AssetKind::Image => Flicks::ZERO,
            AssetKind::Video | AssetKind::Audio => checked_time(
                asset_id,
                time,
                asset.media_info.as_ref().and_then(|info| info.duration),
            )?,
        };
        let fingerprint = match &asset.fingerprint {
            Some(fingerprint) => fingerprint.clone(),
            None => self.deps.files.fingerprint(&original).await?,
        };
        let max_width = max_width.clamp(MIN_FRAME_WIDTH, MAX_FRAME_WIDTH);
        let output = frame_file(
            &media_cache_dir(handle.path(), &fingerprint),
            time,
            max_width,
        );
        if self.deps.files.is_file(&output).await {
            return Ok(output);
        }
        let request = FrameRequest {
            input: original,
            asset_kind: asset.kind,
            time,
            max_width,
            output: output.clone(),
        };
        self.deps.frames.extract_frame(&request).await?;
        Ok(output)
    }

    /// The bytes of [`MediaService::extract_frame`]'s JPEG (AI read tools return them as images).
    ///
    /// # Errors
    /// Those of [`MediaService::extract_frame`]; reading the cached file.
    pub async fn frame_jpeg(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
        time: Flicks,
        max_width: u32,
    ) -> Result<Vec<u8>, AppError> {
        let path = self
            .extract_frame(project_id, asset_id, time, max_width)
            .await?;
        Ok(self.deps.files.read_file(&path).await?)
    }
}

/// `time` when it lies inside the media (`[0, duration]`; any non-negative time when the duration
/// is unknown).
fn checked_time(
    asset_id: AssetId,
    time: Flicks,
    duration: Option<Flicks>,
) -> Result<Flicks, AppError> {
    let inside = !time.is_negative() && duration.is_none_or(|duration| time <= duration);
    if inside {
        Ok(time)
    } else {
        Err(AppError::MediaTimeOutOfRange {
            asset_id,
            time,
            duration: duration.unwrap_or(Flicks::ZERO),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_must_lie_inside_the_media() {
        let asset = AssetId::from_test_index(1);
        let ten = Flicks::from_millis(10_000);
        assert_eq!(
            checked_time(asset, Flicks::one_second(), Some(ten)),
            Ok(Flicks::one_second())
        );
        assert_eq!(checked_time(asset, ten, Some(ten)), Ok(ten));
        assert!(matches!(
            checked_time(asset, Flicks::from_millis(10_001), Some(ten)),
            Err(AppError::MediaTimeOutOfRange { .. })
        ));
        let before = Flicks::ZERO.checked_sub(Flicks::one_second()).unwrap();
        assert!(checked_time(asset, before, None).is_err());
    }
}
