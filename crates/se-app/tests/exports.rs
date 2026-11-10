//! Exports over the example project (prompt 27): the snapshot taken at launch, uploads written
//! in place, progress, completion checked by ffprobe, one export at a time, cancellation and
//! failures leaving no partial file, a silent or closed editor window failing the export,
//! refusals, a closing project cancelling its exports, and the dialog's preferences.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use se_app::{
    AppError,
    exports::is_finished,
    ports::ExportStream,
    test_support::{MediaTestbed, TESTBED_EXPORTS_DIR, probed_video, settle},
};
use se_domain::{
    ExportId, Flicks, ProjectId, SequenceId, TimeRange, test_support::example_project,
};
use se_protocol::{
    AudioEncoderChoice, AudioExportCodec, AudioExportSettings, CoreEvent, ExportContainer,
    ExportInfo, ExportPipeline, ExportPreferences, ExportPreset, ExportPresetVideo, ExportQuality,
    ExportRenderRequest, ExportSettings, ExportState, JobKind, StartExportRequest,
    VideoExportCodec, VideoExportSettings, VideoRate,
};

const PROJECT_PATH: &str = "/films/Example.seproj";

struct Bed {
    services: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
}

async fn bed() -> Bed {
    let services = MediaTestbed::new(Path::new("/films"));
    let project = example_project();
    for (content, asset) in (1..).zip(&project.assets) {
        services
            .files
            .add(asset.original_path.clone(), content, probed_video());
    }
    services
        .repository
        .insert(PathBuf::from(PROJECT_PATH), project.clone(), Vec::new());
    let session = services
        .projects
        .open_project(Path::new(PROJECT_PATH))
        .await
        .unwrap();
    settle().await;
    services.events.take();
    Bed {
        project_id: session.snapshot.project.id,
        sequence_id: project.sequences[0].id,
        services,
    }
}

fn output_path(name: &str) -> PathBuf {
    Path::new(TESTBED_EXPORTS_DIR).join(name)
}

fn mp4() -> ExportSettings {
    ExportSettings {
        container: ExportContainer::Mp4,
        video: Some(VideoExportSettings {
            codec: VideoExportCodec::H264,
            width: 1280,
            height: 720,
            rate: VideoRate::Quality {
                level: ExportQuality::High,
            },
        }),
        audio: Some(AudioExportSettings {
            codec: AudioExportCodec::Aac,
            bits_per_second: 192_000,
            encoder: AudioEncoderChoice::WebCodecs,
        }),
    }
}

fn prores() -> ExportSettings {
    ExportSettings {
        container: ExportContainer::Mov,
        video: Some(VideoExportSettings {
            codec: VideoExportCodec::Prores422Hq,
            width: 64,
            height: 36,
            rate: VideoRate::Quality {
                level: ExportQuality::VeryHigh,
            },
        }),
        audio: None,
    }
}

impl Bed {
    fn request(&self, name: &str, settings: ExportSettings) -> StartExportRequest {
        StartExportRequest {
            project_id: self.project_id,
            sequence_id: self.sequence_id,
            settings,
            range: None,
            path: output_path(name).to_string_lossy().into_owned(),
            preset_name: "Test".to_owned(),
        }
    }

    async fn start(&self, name: &str, settings: ExportSettings) -> ExportInfo {
        let export = self
            .services
            .exports
            .start(self.request(name, settings))
            .await
            .unwrap();
        settle().await;
        export
    }

    fn state(&self, export_id: ExportId) -> ExportState {
        self.services
            .exports
            .list()
            .into_iter()
            .find(|export| export.id == export_id)
            .unwrap()
            .state
    }

    fn render_requests(events: &[CoreEvent]) -> Vec<ExportRenderRequest> {
        events
            .iter()
            .filter_map(|event| match event {
                CoreEvent::ExportRenderRequested(request) => Some(request.clone()),
                _ => None,
            })
            .collect()
    }
}

fn cancelled_renders(events: &[CoreEvent]) -> Vec<ExportId> {
    events
        .iter()
        .filter_map(|event| match event {
            CoreEvent::ExportRenderCancelled { export_id } => Some(*export_id),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn an_export_renders_its_snapshot_and_completes_the_file() {
    let bed = bed().await;
    let project = bed.services.projects.project(bed.project_id).await.unwrap();
    let content_end = project.sequences[0].content_end();
    let export = bed.start("film.mp4", mp4()).await;
    // Later edits never reach the export.
    bed.services
        .projects
        .rename_sequence(bed.project_id, bed.sequence_id, "Renamed")
        .await
        .unwrap();
    let events = bed.services.events.take();
    let [request] = Bed::render_requests(&events).try_into().unwrap();
    assert_eq!(request.export_id, export.id);
    assert_eq!(request.pipeline, ExportPipeline::Muxed);
    assert_eq!((request.start, request.end), (Flicks::ZERO, content_end));
    assert_eq!(request.snapshot.sequences.len(), 1);
    assert_eq!(
        request.snapshot.sequences[0].name,
        project.sequences[0].name
    );
    assert_eq!(export.sequence_name, project.sequences[0].name);
    assert_eq!(export.duration, content_end);
    assert!(matches!(
        bed.state(export.id),
        ExportState::Rendering { .. }
    ));

    let exports = &bed.services.exports;
    exports
        .write(export.id, ExportStream::Data, 3, b"def".to_vec())
        .await
        .unwrap();
    exports
        .write(export.id, ExportStream::Data, 0, b"abc".to_vec())
        .await
        .unwrap();
    exports.report_progress(export.id, 50, 100).await.unwrap();
    settle().await;
    assert!(matches!(
        bed.state(export.id),
        ExportState::Rendering { progress, .. } if (progress - 0.5).abs() < 1e-9
    ));
    exports
        .set_thumbnail(export.id, vec![0xFF, 0xD8, 0xFF, 0xE0])
        .unwrap();
    bed.services
        .files
        .add(output_path("film.mp4"), 99, probed_video());
    exports.finish_render(export.id).await.unwrap();
    settle().await;

    let [file] = bed.services.export_outputs.finished().try_into().unwrap();
    assert_eq!(file.path, output_path("film.mp4"));
    assert_eq!(file.data, b"abcdef");
    let ExportState::Completed {
        size_bytes, check, ..
    } = bed.state(export.id)
    else {
        panic!("not completed: {:?}", bed.state(export.id));
    };
    assert_eq!(size_bytes, 6);
    assert_eq!(check.unwrap().video_codec.as_deref(), Some("h264"));
    assert!(exports.thumbnail(export.id).is_some());
    let events = bed.services.events.take();
    assert!(events.iter().any(|event| matches!(
        event,
        CoreEvent::JobCompleted { job } if job.kind == JobKind::Export
    )));
    assert!(matches!(
        exports
            .write(export.id, ExportStream::Data, 0, vec![1])
            .await,
        Err(AppError::ExportNotRendering { .. })
    ));
}

#[tokio::test]
async fn exports_run_one_at_a_time_and_cancel_without_leftovers() {
    let bed = bed().await;
    let first = bed.start("one.mp4", mp4()).await;
    let second = bed.start("two.mp4", mp4()).await;
    assert!(matches!(bed.state(first.id), ExportState::Rendering { .. }));
    assert_eq!(bed.state(second.id), ExportState::Queued);

    bed.services.exports.cancel(second.id).unwrap();
    bed.services
        .exports
        .write(first.id, ExportStream::Data, 0, vec![7; 16])
        .await
        .unwrap();
    bed.services.exports.cancel(first.id).unwrap();
    settle().await;

    assert_eq!(bed.state(first.id), ExportState::Cancelled);
    assert_eq!(bed.state(second.id), ExportState::Cancelled);
    assert_eq!(
        bed.services.export_outputs.discarded(),
        vec![output_path("one.mp4")]
    );
    // The cancelled second export never opened its file.
    assert_eq!(bed.services.export_outputs.opened().len(), 1);
    let events = bed.services.events.take();
    assert!(cancelled_renders(&events).contains(&first.id));
    assert!(matches!(
        bed.services.exports.report_progress(first.id, 1, 2).await,
        Err(AppError::ExportNotRendering { .. })
    ));
    bed.services.exports.clear_finished();
    assert_eq!(bed.services.exports.list(), Vec::<ExportInfo>::new());
}

#[tokio::test]
async fn a_failed_render_is_reported_and_never_retried() {
    let bed = bed().await;
    let export = bed.start("broken.mp4", mp4()).await;
    bed.services
        .exports
        .fail_render(export.id, "the H.264 encoder stopped".to_owned())
        .await
        .unwrap();
    settle().await;
    let ExportState::Failed { message } = bed.state(export.id) else {
        panic!("not failed: {:?}", bed.state(export.id));
    };
    assert!(message.contains("the H.264 encoder stopped"), "{message}");
    assert_eq!(bed.services.export_outputs.opened().len(), 1);
    assert_eq!(
        bed.services.export_outputs.discarded(),
        vec![output_path("broken.mp4")]
    );
    let events = bed.services.events.take();
    assert!(events.iter().any(|event| matches!(
        event,
        CoreEvent::JobFailed { job } if job.kind == JobKind::Export && job.attempt == 1
    )));
}

#[tokio::test]
async fn files_that_cannot_be_prepared_or_completed_fail_the_export() {
    let bed = bed().await;
    bed.services
        .export_outputs
        .fail_next_open("the folder is read-only");
    let unopened = bed.start("locked.mp4", mp4()).await;
    let ExportState::Failed { message } = bed.state(unopened.id) else {
        panic!("not failed");
    };
    assert!(message.contains("read-only"), "{message}");

    bed.services
        .export_outputs
        .fail_next_finish("the disk is full");
    let unfinished = bed.start("full.mp4", mp4()).await;
    bed.services
        .exports
        .finish_render(unfinished.id)
        .await
        .unwrap();
    settle().await;
    let ExportState::Failed { message } = bed.state(unfinished.id) else {
        panic!("not failed");
    };
    assert!(message.contains("disk is full"), "{message}");
    assert_eq!(
        bed.services.export_outputs.discarded(),
        vec![output_path("full.mp4")]
    );
}

#[tokio::test]
async fn masters_receive_frames_of_their_size() {
    let bed = bed().await;
    let export = bed.start("master.mov", prores()).await;
    let events = bed.services.events.take();
    let [request] = Bed::render_requests(&events).try_into().unwrap();
    assert_eq!(
        request.pipeline,
        ExportPipeline::FfmpegFrames { with_audio: false }
    );
    let frame = vec![0; 64 * 36 * 4];
    bed.services
        .exports
        .write_frame(export.id, frame.clone())
        .await
        .unwrap();
    bed.services
        .exports
        .write_frame(export.id, frame)
        .await
        .unwrap();
    assert!(matches!(
        bed.services
            .exports
            .write_frame(export.id, vec![0; 10])
            .await,
        Err(AppError::Media(_))
    ));
    settle().await;
    assert!(matches!(bed.state(export.id), ExportState::Failed { .. }));
}

#[tokio::test(start_paused = true)]
async fn a_silent_or_closed_editor_window_fails_the_export() {
    let bed = bed().await;
    let silent = bed.start("silent.mp4", mp4()).await;
    tokio::time::sleep(Duration::from_secs(91)).await;
    settle().await;
    let ExportState::Failed { message } = bed.state(silent.id) else {
        panic!("not failed");
    };
    assert!(message.contains("sent nothing"), "{message}");

    let closed = bed.start("closed.mp4", mp4()).await;
    bed.services.renderer.set_connected(false);
    tokio::time::sleep(Duration::from_secs(3)).await;
    settle().await;
    let ExportState::Failed { message } = bed.state(closed.id) else {
        panic!("not failed");
    };
    assert!(message.contains("closed during the export"), "{message}");
}

#[tokio::test]
async fn bad_requests_are_refused_with_what_to_change() {
    let bed = bed().await;
    let exports = &bed.services.exports;
    let refused = |result: Result<ExportInfo, AppError>| match result {
        Err(AppError::InvalidExport { reason }) => reason,
        other => panic!("not refused: {other:?}"),
    };
    let wrong_extension = refused(exports.start(bed.request("film.mov", mp4())).await);
    assert!(wrong_extension.contains(".mp4"), "{wrong_extension}");
    let mut relative = bed.request("film.mp4", mp4());
    relative.path = "film.mp4".to_owned();
    assert!(refused(exports.start(relative).await).contains("absolute"));
    let mut empty = bed.request("film.mp4", mp4());
    empty.range =
        Some(TimeRange::from_start_end(Flicks::one_second(), Flicks::one_second()).unwrap());
    assert!(refused(exports.start(empty).await).contains("empty"));

    let first = bed.start("film.mp4", mp4()).await;
    let busy = refused(exports.start(bed.request("film.mp4", mp4())).await);
    assert!(busy.contains("another export"), "{busy}");
    exports.cancel(first.id).unwrap();
    settle().await;

    bed.services.renderer.set_connected(false);
    assert!(matches!(
        exports.start(bed.request("film.mp4", mp4())).await,
        Err(AppError::RendererUnavailable { .. })
    ));
}

#[tokio::test]
async fn closing_a_project_cancels_its_exports() {
    let bed = bed().await;
    let running = bed.start("running.mp4", mp4()).await;
    let queued = bed.start("queued.mp4", mp4()).await;
    bed.services
        .projects
        .close_project(bed.project_id)
        .await
        .unwrap();
    settle().await;
    assert_eq!(bed.state(running.id), ExportState::Cancelled);
    assert_eq!(bed.state(queued.id), ExportState::Cancelled);
    assert_eq!(
        bed.services.export_outputs.discarded(),
        vec![output_path("running.mp4")]
    );
    assert!(
        bed.services
            .exports
            .list()
            .iter()
            .all(|export| is_finished(&export.state))
    );
}

#[tokio::test]
async fn the_dialog_remembers_its_preferences() {
    let bed = bed().await;
    let exports = &bed.services.exports;
    let defaults = exports.preferences().await.unwrap();
    assert_eq!(defaults.directory, TESTBED_EXPORTS_DIR);
    assert_eq!(defaults.file_name_template, "{sequence}");
    assert_eq!(defaults.presets, Vec::<ExportPreset>::new());

    let saved = ExportPreferences {
        directory: output_path("clients").to_string_lossy().into_owned(),
        file_name_template: "{project} - {preset}".to_owned(),
        presets: vec![ExportPreset {
            id: "p1".to_owned(),
            name: "Client 4K".to_owned(),
            container: ExportContainer::Mp4,
            video: Some(ExportPresetVideo {
                codec: VideoExportCodec::Hevc,
                short_side: 2160,
                rate: VideoRate::Bitrate {
                    bits_per_second: 40_000_000,
                },
            }),
            audio: mp4().audio,
        }],
    };
    exports.update_preferences(&saved).await.unwrap();
    assert_eq!(exports.preferences().await.unwrap(), saved);
    let mut broken = saved;
    broken.file_name_template = "a/b".to_owned();
    assert!(matches!(
        exports.update_preferences(&broken).await,
        Err(AppError::InvalidExport { .. })
    ));
}
