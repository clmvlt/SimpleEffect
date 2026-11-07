//! The thumbnails job: a sheet of small pictures (90 px high) sampled along the media, its JSON
//! layout and a poster, all in the asset's cache folder.
//!
//! Three ffmpeg runs in a working folder: tiles (`tile-001.jpg`…, with their source times printed
//! by the `metadata` filter), the sheet assembled with the `tile` filter, and the poster. The
//! outputs are moved into place at the end, the JSON last.

use std::path::{Path, PathBuf};

use se_app::ports::{MediaError, MediaJobSpec, ProgressSink};
use se_domain::{AssetKind, Flicks};
use se_protocol::ThumbnailSheet;

use super::{
    ffmpeg_progress::{fraction, out_time_seconds},
    thumbnail_plan::ThumbnailPlan,
};
use crate::{
    cache_layout::{
        THUMBNAIL_INFO, THUMBNAIL_POSTER, THUMBNAIL_SHEET, THUMBNAIL_WORK_DIR, write_atomically,
    },
    process::Sidecar,
};

/// Share of the progress bar taken by the tiles run (the sheet and poster runs are quick).
const TILES_SHARE: f64 = 0.9;
/// File of the tile times printed by the `metadata` filter.
const TIMES_FILE: &str = "tiles.txt";
/// Name pattern of the tiles.
const TILE_PATTERN: &str = "tile-%03d.jpg";
/// Options every ffmpeg run starts with.
const COMMON: [&str; 6] = ["-hide_banner", "-nostdin", "-v", "error", "-y", "-nostats"];

/// Generates the thumbnails of `spec` with the ffmpeg at `ffmpeg`.
///
/// # Errors
/// [`MediaError`] when ffmpeg fails or the cache folder cannot be written.
pub async fn generate(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    progress: &dyn ProgressSink,
) -> Result<(), MediaError> {
    let plan = ThumbnailPlan::new(spec.asset_kind, &spec.media_info)?;
    let work = spec.output_dir.join(THUMBNAIL_WORK_DIR);
    let io = |error: std::io::Error| MediaError::Io {
        path: work.clone(),
        message: error.to_string(),
    };
    if tokio::fs::try_exists(&work).await.unwrap_or(false) {
        tokio::fs::remove_dir_all(&work).await.map_err(io)?;
    }
    tokio::fs::create_dir_all(&work).await.map_err(io)?;
    let mut tile_seconds = render_tiles(ffmpeg, spec, &plan, &work, progress).await?;
    if count_tiles(&work).await.map_err(io)? == 0 {
        // Nothing selected (very short or damaged media): one tile from the first picture.
        render_first_frame_tile(ffmpeg, spec, &plan, &work).await?;
        tile_seconds = vec![0.0];
    }
    let tile_count = count_tiles(&work).await.map_err(io)?.max(1);
    let (columns, rows) = ThumbnailPlan::sheet_grid(tile_count);
    assemble_sheet(ffmpeg, &work, columns, rows).await?;
    render_poster(ffmpeg, spec, &plan, &work).await?;
    let sheet = ThumbnailSheet {
        tile_width: plan.tile_width,
        tile_height: plan.tile_height,
        columns,
        rows,
        times: tile_times(&tile_seconds, tile_count, &plan),
        poster_width: plan.poster_width,
        poster_height: plan.poster_height,
    };
    publish(spec, &work, &sheet).await.map_err(io)?;
    progress.report(1.0);
    Ok(())
}

/// Renders the tiles; returns the source times the `metadata` filter printed (may be empty).
async fn render_tiles(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    plan: &ThumbnailPlan,
    work: &Path,
    progress: &dyn ProgressSink,
) -> Result<Vec<f64>, MediaError> {
    let scale = format!(
        "scale={}:{}:flags=bilinear,setsar=1",
        plan.tile_width, plan.tile_height
    );
    let filter = if spec.asset_kind == AssetKind::Image {
        scale
    } else {
        format!(
            "select='isnan(prev_selected_t)+gte(t-prev_selected_t,{:.3})',{scale},\
             metadata=mode=add:key=se_tile:value=1,metadata=mode=print:file={TIMES_FILE}",
            plan.interval_seconds
        )
    };
    let mut args: Vec<String> = COMMON.iter().map(|arg| (*arg).to_owned()).collect();
    if plan.keyframes_only {
        args.extend(["-skip_frame".into(), "nokey".into()]);
    }
    args.extend(["-i".into(), spec.input.to_string_lossy().into_owned()]);
    args.extend(["-an", "-sn", "-dn", "-vf"].map(str::to_owned));
    args.push(filter);
    args.extend(["-fps_mode", "vfr", "-q:v", "5", "-frames:v"].map(str::to_owned));
    args.push(plan.tiles.to_string());
    args.extend(["-progress".into(), "pipe:1".into(), TILE_PATTERN.into()]);
    Sidecar::spawn(ffmpeg, &args, Some(work))?
        .run_with_lines(|line| {
            if let Some(done) =
                out_time_seconds(line).and_then(|seconds| fraction(seconds, plan.duration_seconds))
            {
                progress.report(done * TILES_SHARE);
            }
        })
        .await?;
    let printed = tokio::fs::read_to_string(work.join(TIMES_FILE))
        .await
        .unwrap_or_default();
    Ok(parse_tile_times(&printed))
}

/// One tile from the first decoded picture.
async fn render_first_frame_tile(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    plan: &ThumbnailPlan,
    work: &Path,
) -> Result<(), MediaError> {
    let mut args: Vec<String> = COMMON.iter().map(|arg| (*arg).to_owned()).collect();
    args.extend(["-i".into(), spec.input.to_string_lossy().into_owned()]);
    args.extend(["-an", "-sn", "-dn", "-frames:v", "1", "-q:v", "5", "-vf"].map(str::to_owned));
    args.push(format!(
        "scale={}:{},setsar=1",
        plan.tile_width, plan.tile_height
    ));
    args.push("tile-001.jpg".to_owned());
    Sidecar::spawn(ffmpeg, &args, Some(work))?.finish().await
}

/// Lays the tiles out in one JPEG.
async fn assemble_sheet(
    ffmpeg: &Path,
    work: &Path,
    columns: u32,
    rows: u32,
) -> Result<(), MediaError> {
    let mut args: Vec<String> = COMMON.iter().map(|arg| (*arg).to_owned()).collect();
    args.extend(["-i", TILE_PATTERN, "-vf"].map(str::to_owned));
    args.push(format!("tile={columns}x{rows}"));
    args.extend(["-frames:v", "1", "-q:v", "4", THUMBNAIL_SHEET].map(str::to_owned));
    Sidecar::spawn(ffmpeg, &args, Some(work))?.finish().await
}

/// The poster: one larger picture at [`ThumbnailPlan::poster_seconds`] (input seeking is fast).
async fn render_poster(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    plan: &ThumbnailPlan,
    work: &Path,
) -> Result<(), MediaError> {
    let mut args: Vec<String> = COMMON.iter().map(|arg| (*arg).to_owned()).collect();
    if spec.asset_kind == AssetKind::Video && plan.poster_seconds > 0.0 {
        args.extend(["-ss".into(), format!("{:.3}", plan.poster_seconds)]);
    }
    args.extend(["-i".into(), spec.input.to_string_lossy().into_owned()]);
    args.extend(["-an", "-sn", "-dn", "-frames:v", "1", "-q:v", "3", "-vf"].map(str::to_owned));
    args.push(format!(
        "scale={}:{}:flags=bicubic,setsar=1",
        plan.poster_width, plan.poster_height
    ));
    args.push(THUMBNAIL_POSTER.to_owned());
    Sidecar::spawn(ffmpeg, &args, Some(work))?.finish().await
}

/// Number of `tile-NNN.jpg` files in `work`.
async fn count_tiles(work: &Path) -> std::io::Result<u32> {
    let mut entries = tokio::fs::read_dir(work).await?;
    let mut count = 0_u32;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("tile-") && name.ends_with(".jpg") {
            count += 1;
        }
    }
    Ok(count)
}

/// Moves the sheet and poster into the cache folder, then writes the JSON layout, and removes
/// the working folder.
async fn publish(spec: &MediaJobSpec, work: &Path, sheet: &ThumbnailSheet) -> std::io::Result<()> {
    let destination = |name: &str| -> PathBuf { spec.output_dir.join(name) };
    tokio::fs::rename(work.join(THUMBNAIL_SHEET), destination(THUMBNAIL_SHEET)).await?;
    tokio::fs::rename(work.join(THUMBNAIL_POSTER), destination(THUMBNAIL_POSTER)).await?;
    let json = serde_json::to_vec_pretty(sheet).map_err(std::io::Error::other)?;
    write_atomically(&destination(THUMBNAIL_INFO), &json).await?;
    tokio::fs::remove_dir_all(work).await
}

/// Source times printed by `metadata=mode=print`: lines like `frame:3 pts:90090 pts_time:3.003`.
#[must_use]
pub fn parse_tile_times(printed: &str) -> Vec<f64> {
    printed
        .lines()
        .filter_map(|line| {
            let (_, rest) = line.split_once("pts_time:")?;
            rest.split_whitespace().next()?.parse::<f64>().ok()
        })
        .collect()
}

/// One time per tile: the printed ones when they match the tiles, else evenly spaced ones.
fn tile_times(printed: &[f64], tiles: u32, plan: &ThumbnailPlan) -> Vec<Flicks> {
    let seconds: Vec<f64> = if printed.len() == tiles as usize {
        printed.to_vec()
    } else {
        (0..tiles)
            .map(|index| f64::from(index) * plan.interval_seconds)
            .collect()
    };
    seconds
        .into_iter()
        .map(|seconds| Flicks::from_seconds_f64(seconds.max(0.0)).unwrap_or(Flicks::ZERO))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printed_metadata_gives_the_tile_times() {
        let printed = "frame:0    pts:0       pts_time:0\nse_tile=1\nframe:1    pts:90090   pts_time:3.003\nse_tile=1\n";
        assert_eq!(parse_tile_times(printed), [0.0, 3.003]);
        assert_eq!(parse_tile_times(""), Vec::<f64>::new());
    }
}
