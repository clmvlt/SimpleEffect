//! Shared helpers: a project with tracks V1, V2, A1 opened over in-memory ports, and readers of
//! its tracks and undo state.

use std::{path::Path, path::PathBuf, sync::Arc, time::Duration};

use se_app::{
    AppError, ProjectService, ProjectServiceConfig, ProjectServiceDeps,
    test_support::{
        FixedClock, InMemoryAppLibrary, InMemoryProjectRepository, RecordingEventSink,
        SequentialIds,
    },
};
use se_domain::{
    Flicks, Item, ItemId, ItemKind, LinkGroupId, MediaClip, Project, ProjectId, SequenceId,
    TimeRange, TrackId, TrackKind,
    test_support::{example_ids, example_project, item, track},
};
use se_protocol::{ErrorCode, ItemPlacement, ProjectSession, ProtocolError};

/// Test index of track V1.
pub const V1: u64 = 101;
/// Test index of track V2.
pub const V2: u64 = 102;
/// Test index of track A1.
pub const A1: u64 = 103;

/// `seconds` as flicks.
pub fn secs(seconds: i64) -> Flicks {
    Flicks::new(seconds * Flicks::PER_SECOND).unwrap()
}

/// The track id of a test index.
pub fn track_id(index: u64) -> TrackId {
    TrackId::from_test_index(index)
}

/// The item id of a test index.
pub fn item_id(index: u64) -> ItemId {
    ItemId::from_test_index(index)
}

/// A clip of the example's 60 s interview over `[start, end)` seconds.
pub fn clip(index: u64, start: i64, end: i64, group: Option<u64>) -> Item {
    let range = TimeRange::from_start_end(secs(start), secs(end)).unwrap();
    let mut clip = item(
        index,
        range,
        ItemKind::MediaClip(MediaClip::new(example_ids::VIDEO_ASSET)),
    );
    clip.link_group = group.map(LinkGroupId::from_test_index);
    clip
}

/// The example project reduced to one sequence with tracks V1, V2, A1 holding these items.
pub fn project_with(video1: Vec<Item>, video2: Vec<Item>, audio1: Vec<Item>) -> Project {
    let mut project = example_project();
    project.sequences.truncate(1);
    project.sequences[0].tracks = vec![
        track(V1, TrackKind::Video, "V1", video1),
        track(V2, TrackKind::Video, "V2", video2),
        track(A1, TrackKind::Audio, "A1", audio1),
    ];
    project
}

/// An open project and its first sequence.
pub struct Opened {
    pub service: ProjectService,
    pub project: ProjectId,
    pub sequence: SequenceId,
}

/// A service with `project` open from an in-memory folder.
pub async fn open(project: Project) -> Opened {
    let repository = Arc::new(InMemoryProjectRepository::default());
    repository.insert("/projects/Film.seproj", project.clone(), Vec::new());
    let deps = ProjectServiceDeps {
        repository,
        library: Arc::new(InMemoryAppLibrary::default()),
        events: Arc::new(RecordingEventSink::default()),
        clock: Arc::new(FixedClock::default()),
        ids: Arc::new(SequentialIds::default()),
    };
    let config = ProjectServiceConfig {
        default_parent_dir: PathBuf::from("/projects"),
        save_delay: Duration::from_secs(2),
    };
    let service = ProjectService::new(deps, config);
    service
        .open_project(Path::new("/projects/Film.seproj"))
        .await
        .unwrap();
    Opened {
        service,
        project: project.id,
        sequence: project.sequences[0].id,
    }
}

impl Opened {
    /// The current session.
    pub async fn session(&self) -> ProjectSession {
        self.service.project_session(self.project).await.unwrap()
    }

    /// `(start, end)` in whole seconds of the items of a track, in order.
    pub async fn spans(&self, track: TrackId) -> Vec<(i64, i64)> {
        let session = self.session().await;
        let entity = session
            .snapshot
            .tracks
            .iter()
            .find(|entity| entity.id == track)
            .unwrap();
        entity
            .item_ids
            .iter()
            .map(|id| {
                let item = session
                    .snapshot
                    .items
                    .iter()
                    .find(|i| i.item.id == *id)
                    .unwrap();
                let range = item.item.timeline_range;
                (
                    range.start().get() / Flicks::PER_SECOND,
                    range.end().get() / Flicks::PER_SECOND,
                )
            })
            .collect()
    }

    /// The current state of item `id`.
    pub async fn item(&self, id: ItemId) -> Item {
        let session = self.session().await;
        session
            .snapshot
            .items
            .into_iter()
            .find(|entity| entity.item.id == id)
            .unwrap()
            .item
    }

    /// The label `undo` would revert.
    pub async fn undo_label(&self) -> Option<String> {
        self.session().await.history.undo_label
    }

    /// Undoes once and checks that the tracks are back as `before`.
    pub async fn undo_restores(&self, before: &[(u64, Vec<(i64, i64)>)]) {
        self.service.undo(self.project).await.unwrap();
        for (track, spans) in before {
            assert_eq!(&self.spans(track_id(*track)).await, spans, "track {track}");
        }
    }
}

/// Item `item` to `track` at `start` seconds.
pub fn placement(item: u64, track: u64, start: i64) -> ItemPlacement {
    ItemPlacement {
        item_id: item_id(item),
        track_id: track_id(track),
        start: secs(start),
    }
}

/// The protocol code of a use-case error.
pub fn code(error: AppError) -> ErrorCode {
    ProtocolError::from(error).code
}
