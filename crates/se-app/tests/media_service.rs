//! The media use cases against in-memory ports, through a real `ProjectService`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_app::{
    JobQueue, JobQueueConfig, MediaService, MediaServiceDeps, ProjectService, ProjectServiceConfig,
    ProjectServiceDeps,
    media_service::media_paths::media_cache_dir,
    test_support::{
        FakeJobRunner, FakeMedia, FixedClock, InMemoryAppLibrary, InMemoryProjectRepository,
        RecordingEventSink, SequentialIds, fingerprint, probed_audio, probed_video,
    },
};
use se_domain::{
    Asset, AssetKind, Decodability, Format, FormatPreset, FrameRate, ProjectId, StreamDecodability,
};
use se_protocol::{
    CoreEvent, DecodabilityReport, ErrorCode, JobKind, ProtocolError, RelinkOutcome,
};

const PARENT: &str = "/films";

struct Harness {
    projects: ProjectService,
    media: Arc<MediaService>,
    files: Arc<FakeMedia>,
    runner: Arc<FakeJobRunner>,
    repository: InMemoryProjectRepository,
    events: Arc<RecordingEventSink>,
}

fn harness() -> Harness {
    let repository = InMemoryProjectRepository::default();
    let events = Arc::new(RecordingEventSink::default());
    let ids = Arc::new(SequentialIds::default());
    let files = Arc::new(FakeMedia::default());
    let runner = Arc::new(FakeJobRunner::default());
    let jobs = JobQueue::new(events.clone(), ids.clone(), JobQueueConfig::default());
    let media = Arc::new(MediaService::new(MediaServiceDeps {
        prober: files.clone(),
        files: files.clone(),
        runner: runner.clone(),
        jobs,
        ids: ids.clone(),
    }));
    let deps = ProjectServiceDeps {
        repository: Arc::new(repository.clone()),
        library: Arc::new(InMemoryAppLibrary::default()),
        events: events.clone(),
        clock: Arc::new(FixedClock::default()),
        ids,
    };
    let config = ProjectServiceConfig {
        default_parent_dir: PathBuf::from(PARENT),
        save_delay: Duration::from_secs(2),
    };
    Harness {
        projects: ProjectService::new(deps, config).with_observer(media.clone()),
        media,
        files,
        runner,
        repository,
        events,
    }
}

fn media_path(name: &str) -> PathBuf {
    Path::new(PARENT).join("rushes").join(name)
}

fn project_path() -> PathBuf {
    Path::new(PARENT).join("Film.seproj")
}

fn code(error: se_app::AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

impl Harness {
    async fn create(&self) -> ProjectId {
        let format = Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25).unwrap();
        let session = self
            .projects
            .create_project("Film", Path::new(PARENT), format)
            .await
            .unwrap();
        session.snapshot.project.id
    }

    async fn assets(&self, project_id: ProjectId) -> Vec<Asset> {
        let session = self.projects.project_session(project_id).await.unwrap();
        session.snapshot.assets
    }
}

/// Lets queued jobs run.
async fn settle() {
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn importing_adds_probed_assets_in_one_undoable_transaction_and_queues_jobs() {
    let harness = harness();
    let project_id = harness.create().await;
    harness.files.add(media_path("clip.mp4"), 1, probed_video());
    harness
        .files
        .add(media_path("voice.wav"), 2, probed_audio());
    harness.files.add_unreadable(media_path("notes.txt"), 3);
    let paths = [
        media_path("clip.mp4"),
        media_path("voice.wav"),
        media_path("notes.txt"),
        media_path("gone.mov"),
    ];

    let report = harness
        .media
        .import_files(project_id, &paths)
        .await
        .unwrap();

    assert_eq!(report.imported.len(), 2);
    let codes: Vec<ErrorCode> = report.failures.iter().map(|failure| failure.code).collect();
    assert_eq!(
        codes,
        [ErrorCode::MediaNotFound, ErrorCode::MediaUnsupported]
    );
    let assets = harness.assets(project_id).await;
    let clip = &assets[0];
    assert_eq!(clip.kind, AssetKind::Video);
    assert_eq!(clip.relative_hint.as_deref(), Some("../rushes/clip.mp4"));
    assert_eq!(clip.fingerprint, Some(fingerprint(1)));
    assert_eq!(
        clip.decodability,
        Decodability {
            video: StreamDecodability::Unknown,
            audio: StreamDecodability::Unknown
        }
    );
    assert_eq!(assets[1].decodability.video, StreamDecodability::NoStream);
    let session = harness.projects.project_session(project_id).await.unwrap();
    assert_eq!(session.history.undo_label.as_deref(), Some("Import media"));

    settle().await;
    let mut runs: Vec<(JobKind, PathBuf)> = harness
        .runner
        .runs()
        .into_iter()
        .map(|spec| (spec.kind, spec.input))
        .collect();
    runs.sort();
    assert_eq!(
        runs,
        [
            (JobKind::Thumbnails, media_path("clip.mp4")),
            (JobKind::WaveformPeaks, media_path("clip.mp4")),
            (JobKind::WaveformPeaks, media_path("voice.wav")),
        ]
    );
    let spec = &harness.runner.runs()[0];
    assert_eq!(
        spec.output_dir,
        media_cache_dir(&project_path(), &fingerprint(1))
    );
    let jobs = harness.media.list_jobs(project_id).unwrap();
    assert_eq!(jobs.len(), 3);
}

#[tokio::test]
async fn contents_already_imported_are_reported_as_duplicates() {
    let harness = harness();
    let project_id = harness.create().await;
    harness.files.add(media_path("clip.mp4"), 1, probed_video());
    harness
        .files
        .add(media_path("copy of clip.mp4"), 1, probed_video());
    harness
        .files
        .add(media_path("voice.wav"), 2, probed_audio());
    harness
        .files
        .add(media_path("voice again.wav"), 2, probed_audio());
    let first = harness
        .media
        .import_files(project_id, &[media_path("clip.mp4")])
        .await
        .unwrap();

    let report = harness
        .media
        .import_files(
            project_id,
            &[
                media_path("copy of clip.mp4"),
                media_path("voice.wav"),
                media_path("voice again.wav"),
            ],
        )
        .await
        .unwrap();

    assert_eq!(report.imported.len(), 1);
    let duplicates: Vec<_> = report
        .duplicates
        .iter()
        .map(|duplicate| duplicate.asset_id)
        .collect();
    assert_eq!(duplicates, [first.imported[0], report.imported[0]]);
    assert_eq!(harness.assets(project_id).await.len(), 2);
}

#[tokio::test]
async fn cached_outputs_are_not_generated_again() {
    let harness = harness();
    let project_id = harness.create().await;
    harness
        .files
        .add(media_path("voice.wav"), 2, probed_audio());
    let cache = media_cache_dir(&project_path(), &fingerprint(2));
    harness.runner.add_output(JobKind::WaveformPeaks, &cache);
    harness
        .media
        .import_files(project_id, &[media_path("voice.wav")])
        .await
        .unwrap();
    settle().await;
    assert_eq!(harness.runner.runs(), []);
}

#[tokio::test]
async fn a_renamed_original_is_flagged_missing_on_reopening_and_relinking_repairs_it() {
    let harness = harness();
    let project_id = harness.create().await;
    harness.files.add(media_path("clip.mp4"), 1, probed_video());
    let report = harness
        .media
        .import_files(project_id, &[media_path("clip.mp4")])
        .await
        .unwrap();
    let asset_id = report.imported[0];
    settle().await;
    harness.projects.close_project(project_id).await.unwrap();
    harness.files.remove(&media_path("clip.mp4"));
    harness
        .files
        .add(media_path("renamed.mp4"), 1, probed_video());

    harness
        .projects
        .open_project(&project_path())
        .await
        .unwrap();
    let asset = harness.assets(project_id).await.remove(0);
    assert!(asset.missing);
    let session = harness.projects.project_session(project_id).await.unwrap();
    assert!(!session.history.can_undo, "missing flags are not undoable");

    let outcome = harness
        .media
        .relink_asset(project_id, asset_id, &media_path("renamed.mp4"), false)
        .await
        .unwrap();
    assert_eq!(outcome, RelinkOutcome::Relinked);
    let asset = harness.assets(project_id).await.remove(0);
    assert!(!asset.missing);
    assert_eq!(
        asset.original_path,
        media_path("renamed.mp4").to_string_lossy()
    );
    assert_eq!(
        asset.relative_hint.as_deref(),
        Some("../rushes/renamed.mp4")
    );
}

#[tokio::test]
async fn relinking_to_other_content_needs_confirmation_and_the_same_kind() {
    let harness = harness();
    let project_id = harness.create().await;
    harness.files.add(media_path("clip.mp4"), 1, probed_video());
    harness
        .files
        .add(media_path("other.mp4"), 9, probed_video());
    harness
        .files
        .add(media_path("voice.wav"), 2, probed_audio());
    let asset_id = harness
        .media
        .import_files(project_id, &[media_path("clip.mp4")])
        .await
        .unwrap()
        .imported[0];
    let report = DecodabilityReport {
        project_id,
        asset_id,
        video: StreamDecodability::Decodable,
        audio: StreamDecodability::Decodable,
    };
    harness.media.report_decodability(&report).await.unwrap();

    let error = harness
        .media
        .relink_asset(project_id, asset_id, &media_path("voice.wav"), true)
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::MediaKindMismatch);
    let outcome = harness
        .media
        .relink_asset(project_id, asset_id, &media_path("other.mp4"), false)
        .await
        .unwrap();
    assert_eq!(outcome, RelinkOutcome::ContentDiffers);
    assert_eq!(
        harness.assets(project_id).await[0].fingerprint,
        Some(fingerprint(1))
    );

    let outcome = harness
        .media
        .relink_asset(project_id, asset_id, &media_path("other.mp4"), true)
        .await
        .unwrap();
    assert_eq!(outcome, RelinkOutcome::Relinked);
    let asset = harness.assets(project_id).await.remove(0);
    assert_eq!(asset.fingerprint, Some(fingerprint(9)));
    assert!(
        asset.decodability.is_pending(),
        "new content is checked again"
    );
}

#[tokio::test]
async fn media_moved_with_their_project_are_found_through_the_relative_hint() {
    let harness = harness();
    let project_id = harness.create().await;
    harness.files.add(media_path("clip.mp4"), 1, probed_video());
    harness
        .media
        .import_files(project_id, &[media_path("clip.mp4")])
        .await
        .unwrap();
    harness.projects.close_project(project_id).await.unwrap();
    // The user moves /films to /archive: project and media together.
    let stored = harness.repository.folder(project_path()).unwrap();
    let moved = Path::new("/archive").join("Film.seproj");
    harness
        .repository
        .insert(&moved, stored.snapshot, Vec::new());
    harness.files.remove(&media_path("clip.mp4"));
    let new_media = Path::new("/archive").join("rushes").join("clip.mp4");
    harness.files.add(&new_media, 1, probed_video());

    let session = harness.projects.open_project(&moved).await.unwrap();
    let asset = &session.snapshot.assets[0];
    assert!(!asset.missing);
    assert_eq!(Path::new(&asset.original_path), new_media);
}

#[tokio::test]
async fn decodability_reports_are_system_transactions_recorded_once() {
    let harness = harness();
    let project_id = harness.create().await;
    harness
        .files
        .add(media_path("voice.wav"), 2, probed_audio());
    let asset_id = harness
        .media
        .import_files(project_id, &[media_path("voice.wav")])
        .await
        .unwrap()
        .imported[0];
    let report = DecodabilityReport {
        project_id,
        asset_id,
        video: StreamDecodability::NoStream,
        audio: StreamDecodability::Decodable,
    };
    harness.events.take();
    harness.media.report_decodability(&report).await.unwrap();
    harness.media.report_decodability(&report).await.unwrap();
    let patches = harness
        .events
        .take()
        .into_iter()
        .filter(|event| matches!(event, CoreEvent::ProjectPatched(_)))
        .count();
    assert_eq!(patches, 1);
    let session = harness.projects.project_session(project_id).await.unwrap();
    assert_eq!(session.history.undo_label.as_deref(), Some("Import media"));
    assert_eq!(
        harness.assets(project_id).await[0].decodability.audio,
        StreamDecodability::Decodable
    );
}

#[tokio::test]
async fn removing_an_asset_is_undoable() {
    let harness = harness();
    let project_id = harness.create().await;
    harness
        .files
        .add(media_path("voice.wav"), 2, probed_audio());
    let asset_id = harness
        .media
        .import_files(project_id, &[media_path("voice.wav")])
        .await
        .unwrap()
        .imported[0];
    harness
        .media
        .remove_asset(project_id, asset_id)
        .await
        .unwrap();
    assert_eq!(harness.assets(project_id).await, []);
    harness.projects.undo(project_id).await.unwrap();
    assert_eq!(harness.assets(project_id).await[0].id, asset_id);
    let error = harness
        .media
        .remove_asset(project_id, se_domain::AssetId::from_test_index(77))
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::AssetNotFound);
}

#[tokio::test]
async fn closing_a_project_stops_its_jobs() {
    let harness = harness();
    let project_id = harness.create().await;
    harness.runner.hold();
    harness.files.add(media_path("clip.mp4"), 1, probed_video());
    harness
        .media
        .import_files(project_id, &[media_path("clip.mp4")])
        .await
        .unwrap();
    settle().await;
    harness.events.take();
    harness.projects.close_project(project_id).await.unwrap();
    let cancelled = harness
        .events
        .take()
        .into_iter()
        .filter(|event| matches!(event, CoreEvent::JobCancelled { .. }))
        .count();
    assert_eq!(cancelled, 2);
    let error = harness.media.list_jobs(project_id).unwrap_err();
    assert_eq!(code(error), ErrorCode::ProjectNotOpen);
}
