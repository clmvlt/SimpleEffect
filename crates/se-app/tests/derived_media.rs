//! Mezzanines, proxies, analyses, settings, the project cache and frames, against in-memory ports
//! through a real `ProjectService` (`MediaTestbed`).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::{Path, PathBuf};

use se_app::{
    media_service::media_paths::media_cache_dir,
    ports::{MediaTask, VideoCodecTarget, VideoPlan},
    test_support::{MediaTestbed, fingerprint, probed_audio, probed_prores, probed_video, settle},
};
use se_domain::{
    AnalysisKind, Asset, AssetId, DerivedFile, DerivedStatus, Flicks, ProjectId,
    StreamDecodability::{Decodable, NoStream, Undecodable},
};
use se_protocol::{
    DecodabilityReport, DecoderCapabilities, DerivedKind, ErrorCode, GenerateDerivedRequest,
    JobKind, MediaSettings, ProtocolError, ProxyResolution,
};

const PARENT: &str = "/films";

fn media_path(name: &str) -> PathBuf {
    Path::new(PARENT).join("rushes").join(name)
}

fn project_path() -> PathBuf {
    Path::new(PARENT).join("Film.seproj")
}

fn code(error: se_app::AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

fn ready(relative_path: &str) -> DerivedFile {
    DerivedFile {
        relative_path: relative_path.to_owned(),
        status: DerivedStatus::Ready,
    }
}

/// A testbed with an open project holding one imported asset.
async fn with_asset(
    name: &str,
    content: u64,
    probed: se_app::ports::ProbedMedia,
) -> (MediaTestbed, ProjectId, AssetId) {
    let testbed = MediaTestbed::new(Path::new(PARENT));
    let project_id = testbed
        .create_project(Path::new(PARENT), "Film")
        .await
        .unwrap();
    testbed.files.add(media_path(name), content, probed);
    let report = testbed
        .media
        .import_files(project_id, &[media_path(name)])
        .await
        .unwrap();
    (testbed, project_id, report.imported[0])
}

async fn report(
    testbed: &MediaTestbed,
    project_id: ProjectId,
    asset_id: AssetId,
    video: se_domain::StreamDecodability,
    audio: se_domain::StreamDecodability,
) {
    let report = DecodabilityReport {
        project_id,
        asset_id,
        video,
        audio,
    };
    testbed.media.report_decodability(&report).await.unwrap();
}

async fn asset(testbed: &MediaTestbed, project_id: ProjectId) -> Asset {
    testbed.assets(project_id).await.unwrap().remove(0)
}

fn cache_file(content: u64, name: &str) -> String {
    format!("cache/media/{}/{name}", fingerprint(content).cache_key())
}

#[tokio::test]
async fn undecodable_prores_gets_a_mezzanine_and_a_proxy_and_becomes_playable() {
    let (testbed, project_id, asset_id) = with_asset("interview.mov", 1, probed_prores()).await;
    let capabilities = DecoderCapabilities {
        hevc_main10: true,
        h264_high: true,
        aac: true,
        flac: true,
    };
    testbed
        .media
        .report_decoder_capabilities(capabilities)
        .await
        .unwrap();
    testbed.runner.hold();
    report(&testbed, project_id, asset_id, Undecodable, Decodable).await;

    let pending = asset(&testbed, project_id).await;
    let mezzanine = pending.derived.mezzanine.unwrap();
    assert_eq!(mezzanine.status, DerivedStatus::Pending);
    assert_eq!(mezzanine.relative_path, cache_file(1, "mezzanine.mp4"));
    assert_eq!(
        pending.derived.proxy.unwrap().relative_path,
        cache_file(1, "proxy-540.mp4")
    );
    let session = testbed.projects.project_session(project_id).await.unwrap();
    assert_eq!(
        session.history.undo_label.as_deref(),
        Some("Import media"),
        "derived states are not undoable"
    );

    testbed.runner.release();
    settle().await;
    let done = asset(&testbed, project_id).await;
    assert_eq!(
        done.derived.mezzanine,
        Some(ready(&cache_file(1, "mezzanine.mp4")))
    );
    assert_eq!(
        done.derived.proxy,
        Some(ready(&cache_file(1, "proxy-540.mp4")))
    );
    let plan = testbed
        .runner
        .runs()
        .into_iter()
        .find_map(|spec| match spec.task {
            MediaTask::Transcode(plan) if plan.job_kind() == JobKind::Mezzanine => Some(plan),
            _ => None,
        })
        .unwrap();
    assert!(
        matches!(plan.recipe.video, VideoPlan::Encode(ref encoding) if encoding.codec == VideoCodecTarget::HevcMain10)
    );
    assert_eq!(
        plan.output,
        project_path()
            .join("cache")
            .join("media")
            .join(fingerprint(1).cache_key())
            .join("mezzanine.mp4")
    );
    let files = testbed.media.asset_files(asset_id).await.unwrap();
    assert_eq!(files.mezzanine, Some(plan.output));
}

#[tokio::test]
async fn decodable_1080p_h264_gets_neither_mezzanine_nor_proxy() {
    let (testbed, project_id, asset_id) = with_asset("clip.mp4", 1, probed_video()).await;
    report(&testbed, project_id, asset_id, Decodable, Decodable).await;
    settle().await;
    let asset = asset(&testbed, project_id).await;
    assert_eq!(asset.derived.mezzanine, None);
    assert_eq!(asset.derived.proxy, None);
    let kinds = testbed.runner.run_kinds();
    assert!(!kinds.contains(&JobKind::Mezzanine) && !kinds.contains(&JobKind::Proxy));
}

#[tokio::test]
async fn large_footage_gets_an_automatic_proxy_unless_turned_off_and_proxies_can_be_requested() {
    let mut uhd = probed_video();
    let video = uhd.info.video_stream.as_mut().unwrap();
    (video.width, video.height) = (3840, 2160);
    let (testbed, project_id, _) = with_asset("uhd.mp4", 1, uhd.clone()).await;
    settle().await;
    assert_eq!(
        asset(&testbed, project_id).await.derived.proxy,
        Some(ready(&cache_file(1, "proxy-540.mp4")))
    );

    let settings = MediaSettings {
        auto_proxies: false,
        proxy_resolution: ProxyResolution::P720,
        ..MediaSettings::default()
    };
    testbed.media.update_media_settings(settings).await.unwrap();
    testbed.files.add(media_path("other.mp4"), 2, uhd);
    let other = testbed
        .media
        .import_files(project_id, &[media_path("other.mp4")])
        .await
        .unwrap()
        .imported[0];
    settle().await;
    assert_eq!(
        testbed.assets(project_id).await.unwrap()[1].derived.proxy,
        None
    );

    let request = GenerateDerivedRequest {
        project_id,
        asset_id: other,
        derived: DerivedKind::Proxy,
    };
    testbed.media.generate_derived(&request).await.unwrap();
    settle().await;
    assert_eq!(
        testbed.assets(project_id).await.unwrap()[1].derived.proxy,
        Some(ready(&cache_file(2, "proxy-720.mp4")))
    );
}

#[tokio::test]
async fn derived_copies_that_do_not_apply_are_refused() {
    let (testbed, project_id, asset_id) = with_asset("voice.wav", 1, probed_audio()).await;
    let proxy = GenerateDerivedRequest {
        project_id,
        asset_id,
        derived: DerivedKind::Proxy,
    };
    assert_eq!(
        code(testbed.media.generate_derived(&proxy).await.unwrap_err()),
        ErrorCode::DerivedNotApplicable
    );
    report(&testbed, project_id, asset_id, NoStream, Decodable).await;
    let mezzanine = GenerateDerivedRequest {
        derived: DerivedKind::Mezzanine,
        ..proxy
    };
    assert_eq!(
        code(
            testbed
                .media
                .generate_derived(&mezzanine)
                .await
                .unwrap_err()
        ),
        ErrorCode::DerivedNotApplicable
    );
}

#[tokio::test]
async fn a_failed_mezzanine_keeps_its_reason_and_is_retried_on_request_and_on_reopening() {
    let (testbed, project_id, asset_id) = with_asset("interview.mov", 1, probed_prores()).await;
    settle().await;
    let settings = MediaSettings {
        auto_proxies: false,
        ..MediaSettings::default()
    };
    testbed.media.update_media_settings(settings).await.unwrap();
    testbed.runner.fail_next(2);
    report(&testbed, project_id, asset_id, Undecodable, Decodable).await;
    settle().await;
    let failed = asset(&testbed, project_id).await.derived.mezzanine.unwrap();
    assert!(
        matches!(failed.status, DerivedStatus::Failed { ref message } if message.contains("ffmpeg"))
    );

    testbed.runner.fail_next(2);
    let request = GenerateDerivedRequest {
        project_id,
        asset_id,
        derived: DerivedKind::Mezzanine,
    };
    testbed.media.generate_derived(&request).await.unwrap();
    settle().await;
    assert!(matches!(
        asset(&testbed, project_id)
            .await
            .derived
            .mezzanine
            .unwrap()
            .status,
        DerivedStatus::Failed { .. }
    ));

    testbed.projects.close_project(project_id).await.unwrap();
    testbed
        .projects
        .open_project(&project_path())
        .await
        .unwrap();
    settle().await;
    assert_eq!(
        asset(&testbed, project_id).await.derived.mezzanine,
        Some(ready(&cache_file(1, "mezzanine.mp4")))
    );
}

#[tokio::test]
async fn analyses_are_stored_once_per_content_with_a_summary_in_the_asset() {
    let (testbed, project_id, asset_id) = with_asset("interview.mp4", 1, probed_video()).await;
    settle().await;
    let analysed = asset(&testbed, project_id).await;
    let audio = analysed.analysis.audio.unwrap();
    assert_eq!(audio.integrated_lufs, Some(-16.0));
    assert_eq!(audio.silence_count, 1);
    assert_eq!(audio.silence_duration, Flicks::one_second());
    assert_eq!(analysed.analysis.scenes.unwrap().change_count, 2);
    let stored = testbed.repository.folder(project_path()).unwrap().analyses;
    let keys: Vec<_> = stored.keys().copied().collect();
    assert_eq!(
        keys,
        [
            (asset_id, AnalysisKind::AudioAnalysis),
            (asset_id, AnalysisKind::SceneDetection)
        ]
    );
    assert!(
        stored
            .values()
            .all(|analysis| analysis.content_key == fingerprint(1).cache_key())
    );

    testbed.projects.close_project(project_id).await.unwrap();
    testbed
        .projects
        .open_project(&project_path())
        .await
        .unwrap();
    settle().await;
    let analyses_run = testbed
        .runner
        .run_kinds()
        .iter()
        .filter(|kind| **kind == JobKind::AudioAnalysis)
        .count();
    assert_eq!(analyses_run, 1, "a stored analysis is not computed again");

    testbed
        .files
        .add(media_path("other.mp4"), 2, probed_video());
    testbed
        .media
        .relink_asset(project_id, asset_id, &media_path("other.mp4"), true)
        .await
        .unwrap();
    assert_eq!(
        asset(&testbed, project_id).await.analysis.audio,
        None,
        "relinking to other content clears summaries"
    );
    settle().await;
    assert!(asset(&testbed, project_id).await.analysis.audio.is_some());
}

#[tokio::test]
async fn cleaning_the_cache_deletes_derived_files_and_regenerates_them() {
    let (testbed, project_id, asset_id) = with_asset("interview.mov", 1, probed_prores()).await;
    report(&testbed, project_id, asset_id, Undecodable, Decodable).await;
    settle().await;
    let mezzanine = project_path()
        .join("cache")
        .join("media")
        .join(fingerprint(1).cache_key())
        .join("mezzanine.mp4");
    assert!(testbed.files.written().contains(&mezzanine));
    let before = testbed.media.project_cache(project_id).await.unwrap();
    assert_eq!(before.size_bytes, 2 * se_app::test_support::FAKE_FILE_BYTES);
    assert_eq!(Path::new(&before.path), project_path().join("cache"));
    let runs_before = testbed.runner.runs().len();

    testbed.runner.hold();
    testbed.runner.clear_outputs();
    let after = testbed.media.clear_project_cache(project_id).await.unwrap();
    assert_eq!(after.size_bytes, 0);
    assert_eq!(
        asset(&testbed, project_id)
            .await
            .derived
            .mezzanine
            .unwrap()
            .status,
        DerivedStatus::Pending
    );
    testbed.runner.release();
    settle().await;
    assert!(testbed.files.written().contains(&mezzanine));
    let rerun: Vec<JobKind> = testbed.runner.run_kinds().split_off(runs_before);
    assert!(rerun.contains(&JobKind::Thumbnails) && rerun.contains(&JobKind::Mezzanine));
    assert!(
        !rerun.contains(&JobKind::AudioAnalysis),
        "analyses are kept"
    );
    assert_eq!(
        asset(&testbed, project_id).await.derived.mezzanine,
        Some(ready(&cache_file(1, "mezzanine.mp4")))
    );
}

#[tokio::test]
async fn settings_are_validated_saved_and_point_the_tools_at_the_chosen_folder() {
    let testbed = MediaTestbed::new(Path::new(PARENT));
    assert_eq!(
        testbed.media.media_settings().await.unwrap(),
        MediaSettings::default()
    );
    let too_low = MediaSettings {
        mezzanine_quality: 2,
        ..MediaSettings::default()
    };
    let error = testbed
        .media
        .update_media_settings(too_low)
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::InvalidRequest);
    let tools_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .to_string_lossy()
        .into_owned();
    let chosen = MediaSettings {
        mezzanine_quality: 18,
        proxy_resolution: ProxyResolution::P720,
        auto_proxies: false,
        tools_dir: Some(tools_dir.clone()),
    };
    assert_eq!(
        testbed
            .media
            .update_media_settings(chosen.clone())
            .await
            .unwrap(),
        chosen
    );
    assert_eq!(testbed.media.media_settings().await.unwrap(), chosen);
    assert_eq!(testbed.tools.dirs(), [Some(PathBuf::from(tools_dir))]);
    let relative = MediaSettings {
        tools_dir: Some("bin".to_owned()),
        ..chosen
    };
    assert_eq!(
        code(
            testbed
                .media
                .update_media_settings(relative)
                .await
                .unwrap_err()
        ),
        ErrorCode::InvalidRequest
    );
    assert_eq!(testbed.media.decoder_capabilities().await.unwrap(), None);
}

#[tokio::test]
async fn frames_are_extracted_once_and_times_outside_the_media_are_refused() {
    let (testbed, project_id, asset_id) = with_asset("interview.mp4", 1, probed_video()).await;
    let time = Flicks::from_millis(2_500);
    let path = testbed
        .media
        .extract_frame(project_id, asset_id, time, 640)
        .await
        .unwrap();
    assert_eq!(
        path,
        media_cache_dir(&project_path(), &fingerprint(1))
            .join("frames")
            .join(format!("{}-640.jpg", time.get()))
    );
    testbed
        .media
        .extract_frame(project_id, asset_id, time, 640)
        .await
        .unwrap();
    assert_eq!(testbed.frames.requests().len(), 1);
    assert_eq!(
        testbed.frames.requests()[0].input,
        media_path("interview.mp4")
    );

    let error = testbed
        .media
        .extract_frame(project_id, asset_id, Flicks::from_millis(60_000), 640)
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::MediaTimeOutOfRange);
}
