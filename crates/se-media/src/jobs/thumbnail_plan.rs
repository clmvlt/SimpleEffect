//! [`ThumbnailPlan`]: sizes, tile count and sampling of a thumbnails job (pure).

use se_app::ports::MediaError;
use se_domain::{AssetKind, MediaInfo};

/// Height of a sheet tile in pixels.
pub const TILE_HEIGHT: u32 = 90;
/// Largest poster height in pixels (smaller pictures are not enlarged).
pub const POSTER_MAX_HEIGHT: u32 = 360;
/// Most tiles in a sheet.
pub const MAX_TILES: u32 = 40;
/// Tiles per sheet row.
pub const SHEET_COLUMNS: u32 = 8;
/// Clips up to this length are decoded fully for evenly spaced tiles; longer ones decode only
/// keyframes, which is much faster but spaces tiles by the keyframe interval.
const FULL_DECODE_MAX_SECONDS: f64 = 30.0;
/// The poster shows the picture at this fraction of the duration (skipping fades from black).
const POSTER_POSITION: f64 = 0.1;

/// How a thumbnails job samples and sizes its pictures.
#[derive(Debug, Clone, PartialEq)]
pub struct ThumbnailPlan {
    /// Tile size in pixels (display orientation).
    pub tile_width: u32,
    /// Always [`TILE_HEIGHT`].
    pub tile_height: u32,
    /// Poster size in pixels.
    pub poster_width: u32,
    /// Poster height in pixels.
    pub poster_height: u32,
    /// Tiles wanted (fewer when keyframes are sparse).
    pub tiles: u32,
    /// Seconds between tiles.
    pub interval_seconds: f64,
    /// Decode keyframes only (long media).
    pub keyframes_only: bool,
    /// Source time of the poster in seconds.
    pub poster_seconds: f64,
    /// Media duration in seconds, when known.
    pub duration_seconds: Option<f64>,
}

impl ThumbnailPlan {
    /// The plan for an asset with this probe.
    ///
    /// # Errors
    /// [`MediaError::Failed`] when the asset has no picture stream.
    pub fn new(kind: AssetKind, info: &MediaInfo) -> Result<Self, MediaError> {
        let stream = info.video_stream.as_ref().ok_or_else(|| {
            MediaError::Failed("the media has no picture to draw thumbnails from".to_owned())
        })?;
        let (width, height) = if stream.rotation % 180 == 90 {
            (stream.height, stream.width)
        } else {
            (stream.width, stream.height)
        };
        let aspect = f64::from(width.max(1)) / f64::from(height.max(1));
        let poster_height = even(f64::from(height.min(POSTER_MAX_HEIGHT)));
        let duration_seconds = info.duration.map(se_domain::Flicks::to_seconds_f64);
        let (tiles, keyframes_only) = match (kind, duration_seconds) {
            (AssetKind::Video, Some(seconds)) if seconds > 0.0 => {
                // Truncation intended: one tile per whole second, at most MAX_TILES.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let whole_seconds = seconds.floor().min(f64::from(MAX_TILES)) as u32;
                (whole_seconds.max(1), seconds > FULL_DECODE_MAX_SECONDS)
            }
            _ => (1, false),
        };
        let duration = duration_seconds.unwrap_or(0.0);
        Ok(Self {
            tile_width: even(f64::from(TILE_HEIGHT) * aspect),
            tile_height: TILE_HEIGHT,
            poster_width: even(f64::from(poster_height) * aspect),
            poster_height,
            tiles,
            interval_seconds: duration / f64::from(tiles),
            keyframes_only,
            poster_seconds: duration * POSTER_POSITION,
            duration_seconds,
        })
    }

    /// Rows and columns of a sheet holding `tiles` tiles.
    #[must_use]
    pub fn sheet_grid(tiles: u32) -> (u32, u32) {
        let columns = tiles.clamp(1, SHEET_COLUMNS);
        (columns, tiles.max(1).div_ceil(columns))
    }
}

/// The nearest even number of pixels, at least 2 (encoders need even sizes).
fn even(pixels: f64) -> u32 {
    // Sizes are small positive numbers.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rounded = (pixels / 2.0).round().max(1.0) as u32;
    rounded * 2
}

#[cfg(test)]
mod tests {
    use se_domain::{Flicks, test_support::example_project};

    use super::*;

    fn video(seconds: f64, width: u32, height: u32, rotation: u16) -> MediaInfo {
        let mut info = example_project().assets[0].media_info.clone().unwrap();
        info.duration = Flicks::from_seconds_f64(seconds).ok();
        let stream = info.video_stream.as_mut().unwrap();
        stream.width = width;
        stream.height = height;
        stream.rotation = rotation;
        info
    }

    #[test]
    fn short_clips_get_one_tile_per_second_decoded_fully() {
        let plan = ThumbnailPlan::new(AssetKind::Video, &video(12.5, 3840, 2160, 0)).unwrap();
        assert_eq!((plan.tile_width, plan.tile_height), (160, 90));
        assert_eq!((plan.poster_width, plan.poster_height), (640, 360));
        assert_eq!(plan.tiles, 12);
        assert!(!plan.keyframes_only);
        assert!((plan.poster_seconds - 1.25).abs() < 1e-9);
    }

    #[test]
    fn long_media_decode_keyframes_and_cap_the_tiles() {
        let plan = ThumbnailPlan::new(AssetKind::Video, &video(3600.0, 1920, 1080, 0)).unwrap();
        assert_eq!(plan.tiles, MAX_TILES);
        assert!(plan.keyframes_only);
        assert!((plan.interval_seconds - 90.0).abs() < 1e-9);
    }

    #[test]
    fn portrait_phone_footage_uses_its_display_orientation() {
        let plan = ThumbnailPlan::new(AssetKind::Video, &video(5.0, 1920, 1080, 90)).unwrap();
        assert_eq!((plan.tile_width, plan.tile_height), (50, 90));
        assert_eq!((plan.poster_width, plan.poster_height), (202, 360));
    }

    #[test]
    fn small_images_are_not_enlarged() {
        let mut info = video(0.0, 200, 100, 0);
        info.duration = None;
        let plan = ThumbnailPlan::new(AssetKind::Image, &info).unwrap();
        assert_eq!(plan.tiles, 1);
        assert_eq!((plan.poster_width, plan.poster_height), (200, 100));
    }

    #[test]
    fn sheets_have_eight_columns_at_most() {
        assert_eq!(ThumbnailPlan::sheet_grid(1), (1, 1));
        assert_eq!(ThumbnailPlan::sheet_grid(12), (8, 2));
        assert_eq!(ThumbnailPlan::sheet_grid(40), (8, 5));
    }
}
