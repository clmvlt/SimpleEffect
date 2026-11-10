//! Export outputs (prompt 27): a ProRes master encoded from RGBA frames with its WAV, AAC sound
//! joined to a muxed picture and alone, partial files removed when an output is dropped, and a
//! missing folder refused.

use std::{path::Path, time::Duration};

use se_app::ports::{ExportOutputPlan, ExportOutputs, ExportStream};
use se_domain::FrameRate;
use se_media::FsExportOutputs;
use se_protocol::{
    AudioEncoderChoice, AudioExportCodec, AudioExportSettings, ExportContainer, ExportPipeline,
    ExportQuality, VideoExportCodec, VideoExportSettings, VideoRate,
};

use crate::support::{generate, probe, tools};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 36;

fn master_video() -> VideoExportSettings {
    VideoExportSettings {
        codec: VideoExportCodec::Prores422Hq,
        width: WIDTH,
        height: HEIGHT,
        rate: VideoRate::Quality {
            level: ExportQuality::VeryHigh,
        },
    }
}

fn audio(codec: AudioExportCodec, encoder: AudioEncoderChoice) -> AudioExportSettings {
    AudioExportSettings {
        codec,
        bits_per_second: 128_000,
        encoder,
    }
}

fn master_plan(path: &Path) -> ExportOutputPlan {
    ExportOutputPlan {
        path: path.to_path_buf(),
        container: ExportContainer::Mov,
        pipeline: ExportPipeline::FfmpegFrames { with_audio: true },
        video: Some(master_video()),
        audio: Some(audio(AudioExportCodec::Pcm, AudioEncoderChoice::WebCodecs)),
        frame_rate: FrameRate::FPS_25,
    }
}

fn frame(shade: u8) -> Vec<u8> {
    [shade, 255 - shade, 128, 255].repeat((WIDTH * HEIGHT) as usize)
}

fn leftovers(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("part"))
        })
        .collect()
}

/// Writes `bytes` in two chunks, the second first (muxers patch earlier bytes).
async fn write_in_two(
    output: &mut dyn se_app::ports::ExportOutput,
    stream: ExportStream,
    bytes: &[u8],
) {
    let middle = bytes.len() / 2;
    output
        .write(stream, middle as u64, &bytes[middle..])
        .await
        .unwrap();
    output.write(stream, 0, &bytes[..middle]).await.unwrap();
}

#[tokio::test]
async fn prores_masters_are_encoded_from_frames_with_their_wav() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let wav = generate(
        &tools,
        dir.path(),
        "tone.wav",
        &[
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1:sample_rate=48000",
            "-c:a",
            "pcm_s16le",
        ],
    )
    .await;
    let path = dir.path().join("Master.mov");
    let outputs = FsExportOutputs::new(tools.clone());
    let mut output = outputs.open(master_plan(&path)).await.unwrap();
    write_in_two(
        output.as_mut(),
        ExportStream::Audio,
        &std::fs::read(wav).unwrap(),
    )
    .await;
    for index in 0..25u8 {
        output.write_frame(&frame(index * 10)).await.unwrap();
    }
    assert!(output.write_frame(&[0; 12]).await.is_err());
    let size = output.finish().await.unwrap();

    assert_eq!(size, std::fs::metadata(&path).unwrap().len());
    let info = probe(&tools, &path).await.info;
    let video = info.video_stream.unwrap();
    assert_eq!(video.codec, "prores");
    assert_eq!((video.width, video.height), (WIDTH, HEIGHT));
    assert_eq!(info.audio_streams[0].codec, "pcm_s24le");
    let seconds = info.duration.unwrap().to_seconds_f64();
    assert!((seconds - 1.0).abs() < 0.05, "{seconds}");
    assert_eq!(leftovers(dir.path()), Vec::<String>::new());
}

#[tokio::test]
async fn aac_sound_is_joined_to_the_picture_or_kept_alone() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let picture = generate(
        &tools,
        dir.path(),
        "picture.mp4",
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=64x36:rate=25:duration=1",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ],
    )
    .await;
    let wav = generate(
        &tools,
        dir.path(),
        "tone.wav",
        &[
            "-f",
            "lavfi",
            "-i",
            "sine=duration=1:sample_rate=48000",
            "-c:a",
            "pcm_s16le",
        ],
    )
    .await;
    let outputs = FsExportOutputs::new(tools.clone());
    let path = dir.path().join("Film.mp4");
    let mut output = outputs
        .open(ExportOutputPlan {
            path: path.clone(),
            container: ExportContainer::Mp4,
            pipeline: ExportPipeline::AacRemux { with_video: true },
            video: Some(VideoExportSettings {
                codec: VideoExportCodec::H264,
                ..master_video()
            }),
            audio: Some(audio(AudioExportCodec::Aac, AudioEncoderChoice::Ffmpeg)),
            frame_rate: FrameRate::FPS_25,
        })
        .await
        .unwrap();
    write_in_two(
        output.as_mut(),
        ExportStream::Data,
        &std::fs::read(picture).unwrap(),
    )
    .await;
    write_in_two(
        output.as_mut(),
        ExportStream::Audio,
        &std::fs::read(&wav).unwrap(),
    )
    .await;
    output.finish().await.unwrap();
    let info = probe(&tools, &path).await.info;
    assert_eq!(info.video_stream.unwrap().codec, "h264");
    assert_eq!(info.audio_streams[0].codec, "aac");

    let sound_path = dir.path().join("Son.m4a");
    let mut sound = outputs
        .open(ExportOutputPlan {
            path: sound_path.clone(),
            container: ExportContainer::M4a,
            pipeline: ExportPipeline::AacRemux { with_video: false },
            video: None,
            audio: Some(audio(AudioExportCodec::Aac, AudioEncoderChoice::Ffmpeg)),
            frame_rate: FrameRate::FPS_25,
        })
        .await
        .unwrap();
    assert!(sound.write(ExportStream::Data, 0, b"x").await.is_err());
    sound
        .write(ExportStream::Audio, 0, &std::fs::read(&wav).unwrap())
        .await
        .unwrap();
    sound.finish().await.unwrap();
    let info = probe(&tools, &sound_path).await.info;
    assert!(info.video_stream.is_none());
    assert_eq!(info.audio_streams[0].codec, "aac");
    assert_eq!(leftovers(dir.path()), Vec::<String>::new());
}

#[tokio::test]
async fn dropping_an_unfinished_output_removes_its_partial_files() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Cancelled.mov");
    let outputs = FsExportOutputs::new(tools.clone());
    let mut output = outputs.open(master_plan(&path)).await.unwrap();
    output
        .write(ExportStream::Audio, 0, &[0; 64])
        .await
        .unwrap();
    output.write_frame(&frame(0)).await.unwrap();
    output.write_frame(&frame(50)).await.unwrap();
    assert_ne!(leftovers(dir.path()), Vec::<String>::new());
    drop(output);
    for _ in 0..50 {
        if leftovers(dir.path()).is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(leftovers(dir.path()), Vec::<String>::new());
    assert!(!path.exists());
}

#[tokio::test]
async fn a_missing_folder_is_refused() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let outputs = FsExportOutputs::new(tools);
    let Err(error) = outputs
        .open(master_plan(&dir.path().join("missing").join("Film.mov")))
        .await
    else {
        panic!("a missing folder was accepted");
    };
    assert!(error.to_string().contains("does not exist"), "{error}");
}
