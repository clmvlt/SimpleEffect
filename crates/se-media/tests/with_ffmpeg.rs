//! The adapter against real ffmpeg and ffprobe: media are generated with `testsrc2` and `sine`.
//! Every test is skipped (with a message) when the binaries are not installed
//! (`pnpm fetch-binaries`).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use se_app::ports::{MediaJobRunner, MediaJobSpec, MediaProber, ProgressSink};
use se_domain::AssetKind;
use se_media::{
    FfmpegJobRunner, FfprobeProber, Tool, ToolLocator,
    cache_layout::{THUMBNAIL_INFO, THUMBNAIL_POSTER, THUMBNAIL_SHEET, peaks_file},
    peaks::{PEAK_LEVELS, Peaks},
    process::Sidecar,
};
use se_protocol::{JobKind, ThumbnailSheet};

/// The repository's tools (`resources/bin/<platform>/`), or `None` with a message.
fn tools() -> Option<ToolLocator> {
    let locator = ToolLocator {
        override_dir: None,
        exe_dir: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        search_path: false,
    };
    if locator.locate(Tool::Ffmpeg).is_err() || locator.locate(Tool::Ffprobe).is_err() {
        eprintln!("skipped: ffmpeg/ffprobe not installed (run `pnpm fetch-binaries`)");
        return None;
    }
    Some(locator)
}

/// Generates a media file with ffmpeg's lavfi sources.
async fn generate(tools: &ToolLocator, dir: &Path, name: &str, args: &[&str]) -> PathBuf {
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

#[derive(Default)]
struct Progress(Mutex<Vec<f64>>);

impl ProgressSink for Progress {
    fn report(&self, fraction: f64) {
        self.0.lock().unwrap().push(fraction);
    }
}

#[tokio::test]
async fn a_video_gets_a_thumbnail_sheet_a_poster_and_peaks() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let input = generate(
        &tools,
        dir.path(),
        "clip.mp4",
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=1280x720:rate=25",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000",
            "-t",
            "4",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
        ],
    )
    .await;
    let probed = FfprobeProber::new(tools.clone())
        .probe(&input)
        .await
        .unwrap();
    assert_eq!(probed.kind, AssetKind::Video);
    let runner = FfmpegJobRunner::new(tools);
    let cache = dir.path().join("cache");
    for kind in [JobKind::Thumbnails, JobKind::WaveformPeaks] {
        let spec = MediaJobSpec {
            kind,
            input: input.clone(),
            output_dir: cache.clone(),
            asset_kind: probed.kind,
            media_info: probed.info.clone(),
        };
        let progress = Progress::default();
        runner.run(&spec, &progress).await.unwrap();
        assert!(runner.outputs_ready(kind, &cache).await, "{kind:?} outputs");
        assert_eq!(progress.0.lock().unwrap().last(), Some(&1.0));
    }
    let sheet: ThumbnailSheet =
        serde_json::from_slice(&std::fs::read(cache.join(THUMBNAIL_INFO)).unwrap()).unwrap();
    assert_eq!((sheet.tile_width, sheet.tile_height), (160, 90));
    assert_eq!(sheet.times.len(), 4, "one tile per second of a 4 s clip");
    assert_eq!((sheet.columns, sheet.rows), (4, 1));
    assert!(sheet.times.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(cache.join(THUMBNAIL_SHEET).is_file());
    assert!(cache.join(THUMBNAIL_POSTER).is_file());
    assert!(!cache.join("thumbnails.tmp").exists());

    let finest =
        Peaks::decode(&std::fs::read(cache.join(peaks_file(PEAK_LEVELS[0]))).unwrap()).unwrap();
    assert_eq!(finest.channels, 1);
    assert_eq!(finest.sample_rate, 48_000);
    // 4 s at 48 kHz = 192 000 samples = 750 blocks of 256.
    assert!(
        (749..=752).contains(&finest.peak_count()),
        "{}",
        finest.peak_count()
    );
    let loudest = finest.values.iter().map(|(_, max)| *max).max().unwrap();
    assert!(loudest > 3000, "a sine has a visible waveform ({loudest})");
}

#[tokio::test]
async fn images_get_one_tile_and_files_that_are_not_media_are_refused() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let image = generate(
        &tools,
        dir.path(),
        "still.png",
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=320x240",
            "-frames:v",
            "1",
        ],
    )
    .await;
    let ffprobe = FfprobeProber::new(tools.clone());
    let probed = ffprobe.probe(&image).await.unwrap();
    assert_eq!(probed.kind, AssetKind::Image);
    let spec = MediaJobSpec {
        kind: JobKind::Thumbnails,
        input: image,
        output_dir: dir.path().join("cache"),
        asset_kind: probed.kind,
        media_info: probed.info,
    };
    FfmpegJobRunner::new(tools)
        .run(&spec, &Progress::default())
        .await
        .unwrap();
    let sheet: ThumbnailSheet =
        serde_json::from_slice(&std::fs::read(spec.output_dir.join(THUMBNAIL_INFO)).unwrap())
            .unwrap();
    assert_eq!(sheet.times.len(), 1);
    assert_eq!((sheet.poster_width, sheet.poster_height), (320, 240));

    let text = dir.path().join("notes.txt");
    std::fs::write(&text, "not media at all\n").unwrap();
    let error = ffprobe.probe(&text).await.unwrap_err();
    assert!(error.to_string().contains("notes.txt"), "{error}");
}

#[tokio::test]
async fn dropping_a_job_stops_ffmpeg_and_leaves_nothing_complete() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    // A long 4K clip: the job cannot finish within the timeout.
    let input = generate(
        &tools,
        dir.path(),
        "long.mkv",
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=3840x2160:rate=25",
            "-t",
            "20",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-g",
            "1",
        ],
    )
    .await;
    let probed = FfprobeProber::new(tools.clone())
        .probe(&input)
        .await
        .unwrap();
    let spec = MediaJobSpec {
        kind: JobKind::Thumbnails,
        input,
        output_dir: dir.path().join("cache"),
        asset_kind: probed.kind,
        media_info: probed.info,
    };
    let runner = FfmpegJobRunner::new(tools);
    let progress = Progress::default();
    let outcome =
        tokio::time::timeout(Duration::from_millis(300), runner.run(&spec, &progress)).await;
    assert!(
        outcome.is_err(),
        "the job must still be running when cancelled"
    );
    assert!(
        !runner
            .outputs_ready(JobKind::Thumbnails, &spec.output_dir)
            .await
    );
    // The killed ffmpeg released its files: the working folder can be removed at once.
    tokio::time::sleep(Duration::from_millis(200)).await;
    std::fs::remove_dir_all(&spec.output_dir).unwrap();
}
