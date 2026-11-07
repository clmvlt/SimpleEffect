//! Mezzanines, proxies and frames written by real ffmpeg, then probed.

use se_app::ports::{
    AudioEncoding, AudioPlan, Container, FrameExtractor, FrameRequest, MediaJobRunner, MediaTask,
    TranscodePlan, TranscodeRecipe, VideoCodecTarget, VideoEncoding, VideoPlan,
};
use se_domain::{AssetKind, Flicks};
use se_protocol::DerivedKind;

use crate::support::{Progress, generate, probe, runner, spec, tools};

fn encode(
    codec: VideoCodecTarget,
    max_short_side: Option<u32>,
    keyframe_interval: u32,
    fast_decode: bool,
) -> VideoPlan {
    VideoPlan::Encode(VideoEncoding {
        codec,
        quality: 14,
        max_short_side,
        keyframe_interval,
        fast_decode,
    })
}

#[tokio::test]
async fn a_prores_original_becomes_a_10_bit_hevc_mezzanine_with_aac_sound() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let input = generate(
        &tools,
        dir.path(),
        "interview.mov",
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
            "2",
            "-c:v",
            "prores_ks",
            "-profile:v",
            "3",
            "-pix_fmt",
            "yuv422p10le",
            "-c:a",
            "pcm_s16le",
            "-shortest",
        ],
    )
    .await;
    let probed = probe(&tools, &input).await;
    let output = dir.path().join("cache").join("mezzanine.mp4");
    let plan = TranscodePlan {
        derived: DerivedKind::Mezzanine,
        output: output.clone(),
        recipe: TranscodeRecipe {
            container: Container::Mp4,
            video: encode(VideoCodecTarget::HevcMain10, None, 25, false),
            audio: AudioPlan::Encode(AudioEncoding::Aac { bitrate_kbps: 320 }),
        },
    };
    let progress = Progress::default();
    runner(&tools)
        .run(
            &spec(MediaTask::Transcode(plan), &input, dir.path(), &probed),
            &progress,
        )
        .await
        .unwrap();
    assert_eq!(progress.last(), Some(1.0));
    assert!(!dir.path().join("cache").join("mezzanine.part.mp4").exists());

    let mezzanine = probe(&tools, &output).await;
    assert_eq!(mezzanine.kind, AssetKind::Video);
    let video = mezzanine.info.video_stream.unwrap();
    assert_eq!(video.codec, "hevc");
    assert_eq!(video.profile.as_deref(), Some("Main 10"));
    assert_eq!(video.codec_tag.as_deref(), Some("hvc1"));
    assert_eq!(
        (video.width, video.height, video.bit_depth),
        (1280, 720, Some(10))
    );
    assert_eq!(video.chroma_subsampling.as_deref(), Some("4:2:0"));
    assert_eq!(
        video.frame_rate,
        probed.info.video_stream.unwrap().frame_rate
    );
    assert_eq!(mezzanine.info.audio_streams[0].codec, "aac");
}

#[tokio::test]
async fn proxies_are_small_h264_and_keep_the_display_rotation() {
    let Some(tools) = tools() else { return };
    let dir = tempfile::tempdir().unwrap();
    let upright = generate(
        &tools,
        dir.path(),
        "phone.mp4",
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=1920x1080:rate=30",
            "-t",
            "1",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ],
    )
    .await;
    let rotated = generate(
        &tools,
        dir.path(),
        "phone-rotated.mp4",
        &[
            "-display_rotation:v:0",
            "90",
            "-i",
            &upright.to_string_lossy(),
            "-c",
            "copy",
        ],
    )
    .await;
    let probed = probe(&tools, &rotated).await;
    let output = dir.path().join("cache").join("proxy-540.mp4");
    let plan = TranscodePlan {
        derived: DerivedKind::Proxy,
        output: output.clone(),
        recipe: TranscodeRecipe {
            container: Container::Mp4,
            video: encode(VideoCodecTarget::H264High, Some(540), 15, true),
            audio: AudioPlan::Drop,
        },
    };
    runner(&tools)
        .run(
            &spec(MediaTask::Transcode(plan), &rotated, dir.path(), &probed),
            &Progress::default(),
        )
        .await
        .unwrap();
    let proxy = probe(&tools, &output).await.info.video_stream.unwrap();
    assert_eq!(proxy.codec, "h264");
    assert_eq!((proxy.width, proxy.height), (960, 540));
    assert_eq!(proxy.rotation, probed.info.video_stream.unwrap().rotation);
    assert_ne!(proxy.rotation, 0);
}

#[tokio::test]
async fn frames_are_extracted_upright_and_the_end_falls_back_to_the_last_frame() {
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
            "-t",
            "2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ],
    )
    .await;
    let runner = runner(&tools);
    for (name, millis) in [("middle.jpg", 1_000), ("end.jpg", 2_000)] {
        let request = FrameRequest {
            input: input.clone(),
            asset_kind: AssetKind::Video,
            time: Flicks::from_millis(millis),
            max_width: 320,
            output: dir.path().join("frames").join(name),
        };
        runner.extract_frame(&request).await.unwrap();
        let frame = probe(&tools, &request.output).await;
        assert_eq!(frame.kind, AssetKind::Image, "{name}");
        let picture = frame.info.video_stream.unwrap();
        assert_eq!((picture.width, picture.height), (320, 180), "{name}");
    }
}
