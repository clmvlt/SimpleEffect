//! Pictures sampled for face detection (prompt 26): one upright BGR frame per interval, from the
//! first frame, in time order, stopping when the consumer does.

use std::path::Path;

use se_app::ports::{FrameSampleRequest, FrameSampler, SampledFrame};
use se_domain::Flicks;
use tokio::sync::mpsc;

use crate::support::{generate, runner, tools};

/// A 3 s video whose left half is white for the first 2 s and black after (so a frame's time can
/// be read from its pixels).
async fn timed_video(tools: &se_media::ToolLocator, dir: &Path, name: &str) -> std::path::PathBuf {
    generate(
        tools,
        dir,
        name,
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=black:size=320x240:rate=25:duration=3",
            "-vf",
            "drawbox=x=0:y=0:w=160:h=240:color=white:t=fill:enable='lt(t,2)'",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ],
    )
    .await
}

fn request(source: &Path, width: u32, height: u32) -> FrameSampleRequest {
    FrameSampleRequest {
        source: source.to_path_buf(),
        interval: Flicks::from_millis(200),
        width,
        height,
        hardware_decoding: true,
    }
}

async fn collect(sampler: &impl FrameSampler, request: &FrameSampleRequest) -> Vec<SampledFrame> {
    let (sender, mut receiver) = mpsc::channel(4);
    let sampling = sampler.sample_frames(request, sender);
    let reading = async {
        let mut frames = Vec::new();
        while let Some(frame) = receiver.recv().await {
            frames.push(frame);
        }
        frames
    };
    let (result, frames) = tokio::join!(sampling, reading);
    result.unwrap();
    frames
}

/// Mean value of the frame's left half.
fn left_brightness(frame: &SampledFrame) -> f64 {
    let width = frame.width as usize;
    let (sum, count) = frame
        .pixels
        .chunks_exact(width * 3)
        .flat_map(|row| row[..width / 2 * 3].iter())
        .fold((0_u32, 0_u32), |(sum, count), &byte| {
            (sum + u32::from(byte), count + 1)
        });
    f64::from(sum) / f64::from(count.max(1))
}

#[tokio::test]
async fn one_frame_per_interval_from_the_first_with_its_time() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let video = timed_video(&tools, dir.path(), "timed.mp4").await;
    let frames = collect(&runner(&tools), &request(&video, 160, 120)).await;
    assert_eq!(frames.len(), 15, "3 s at 5 samples per second");
    for (index, frame) in (0_u32..).zip(&frames) {
        assert_eq!(frame.time, Flicks::from_millis(200 * index));
        assert_eq!((frame.width, frame.height), (160, 120));
        assert_eq!(frame.pixels.len(), 160 * 120 * 3);
        let white = left_brightness(frame) > 128.0;
        assert_eq!(white, index < 10, "frame {index} at {:?}", frame.time);
    }
}

#[tokio::test]
async fn rotated_videos_are_sampled_upright() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let video = timed_video(&tools, dir.path(), "timed.mp4").await;
    let rotated = generate(
        &tools,
        dir.path(),
        "rotated.mp4",
        &[
            "-display_rotation:v:0",
            "90",
            "-i",
            &video.to_string_lossy(),
            "-c",
            "copy",
        ],
    )
    .await;
    let frames = collect(&runner(&tools), &request(&rotated, 120, 160)).await;
    assert_eq!(frames.len(), 15);
    // Upright, the white half of the first frames is the top or bottom half, not the left one.
    let first = &frames[0];
    let brightness = left_brightness(first);
    assert!((60.0..200.0).contains(&brightness), "{brightness}");
}

#[tokio::test]
async fn sampling_stops_when_the_consumer_stops() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let video = timed_video(&tools, dir.path(), "timed.mp4").await;
    let (sender, mut receiver) = mpsc::channel(1);
    let sampler = runner(&tools);
    let request = request(&video, 160, 120);
    let sampling = sampler.sample_frames(&request, sender);
    let reading = async move {
        let first = receiver.recv().await;
        drop(receiver);
        first
    };
    let (result, first) = tokio::join!(sampling, reading);
    result.unwrap();
    assert_eq!(first.map(|frame| frame.time), Some(Flicks::ZERO));
}
