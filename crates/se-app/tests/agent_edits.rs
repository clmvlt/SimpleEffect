//! Edits of AI agents through `AgentEditService` over the example project: the scope matrix,
//! atomic calls, optimistic concurrency, changesets and task reverts (prompt 19, ADR-0029).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_app::{
    AgentContext, AppError, EditFailure,
    agent_edits::{AgentEdit, ItemRef, NewAgentItem, PresetSpec},
    test_support::{MediaTestbed, ReplayScript, probed_video, settle},
};
use se_domain::{
    AnimationPhase, Conflict, ItemEdge, ItemId, ItemKind, ItemPatch, Origin, Project, ProjectId,
    SequenceId, TextItem, TextPatch, TimeRange, TrackId,
    test_support::{example_project, millis},
};
use se_protocol::{
    CoreEvent, ErrorCode, ItemPlacement, PlacementMode, ProtocolError, SubmitTaskRequest, TaskScope,
};

const PROJECT_PATH: &str = "/films/Example.seproj";

/// Items of the example project (`se_domain::test_support::example_project`).
const CLIP: u64 = 1; // V1, 0–10 s, linked to SOUND
const TITLE: u64 = 3; // V2, 1–4 s
const SHAPE: u64 = 6; // V3, 6–8 s
const CAPTION: u64 = 7; // V3, 8–10 s

struct Bed {
    services: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
    tracks: Vec<TrackId>,
    initial: Project,
}

fn id(index: u64) -> ItemId {
    ItemId::from_test_index(index)
}

fn secs(seconds: f64) -> se_domain::Flicks {
    se_domain::Flicks::from_seconds_f64(seconds).unwrap()
}

fn range(start: f64, end: f64) -> TimeRange {
    TimeRange::from_start_end(secs(start), secs(end)).unwrap()
}

fn code(failure: EditFailure) -> ErrorCode {
    ProtocolError::from(failure.error).code
}

async fn run_for(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}

async fn bed() -> Bed {
    let services = MediaTestbed::new(Path::new("/films"));
    services.tasks.start().await.unwrap();
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
    let initial = services
        .projects
        .project(session.snapshot.project.id)
        .await
        .unwrap();
    Bed {
        project_id: session.snapshot.project.id,
        sequence_id: project.sequences[0].id,
        tracks: project.sequences[0].tracks.iter().map(|t| t.id).collect(),
        initial: initial.as_ref().clone(),
        services,
    }
}

impl Bed {
    /// A finished task (its replayed run completed) and the context of its agent.
    async fn task(&self, range: Option<TimeRange>, tracks: Option<Vec<TrackId>>) -> AgentContext {
        let scope = TaskScope {
            sequence_id: self.sequence_id,
            range,
            tracks,
        };
        let task = self
            .services
            .tasks
            .submit(SubmitTaskRequest {
                project_id: self.project_id,
                prompt: "Edit".to_owned(),
                scope: scope.clone(),
                provider: None,
                model: None,
                effort: None,
            })
            .await
            .unwrap();
        run_for(100).await;
        AgentContext {
            project_id: self.project_id,
            task_id: task.id,
            run: 0,
            scope,
        }
    }

    async fn project(&self) -> Arc<Project> {
        self.services
            .projects
            .project(self.project_id)
            .await
            .unwrap()
    }

    async fn apply(&self, agent: &AgentContext, edits: Vec<AgentEdit>) -> Result<(), EditFailure> {
        self.services
            .edits
            .apply(agent, "test", &edits)
            .await
            .map(drop)
    }

    fn track(&self, index: usize) -> TrackId {
        self.tracks[index]
    }
}

fn text(start: f64, end: f64, track: Option<TrackId>) -> AgentEdit {
    AgentEdit::AddItem(Box::new(NewAgentItem {
        kind: ItemKind::Text(TextItem::new("Abonne-toi")),
        range: range(start, end),
        track,
        presets: vec![PresetSpec {
            preset: "pop".to_owned(),
            phase: AnimationPhase::In,
            duration: None,
        }],
    }))
}

fn move_to(item: u64, start: f64) -> AgentEdit {
    AgentEdit::MoveItem {
        item: ItemRef::Id(id(item)),
        track: None,
        start: secs(start),
        expected_revision: None,
    }
}

fn find(project: &Project, item: ItemId) -> Option<se_domain::Item> {
    project
        .items()
        .find(|candidate| candidate.id == item)
        .cloned()
}

fn without_revisions(mut project: Project) -> Project {
    project.revision = 0;
    for track in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
    {
        track.items.iter_mut().for_each(|item| item.revision = 0);
    }
    project
}

#[tokio::test(start_paused = true)]
async fn the_scope_matrix_refuses_what_lies_outside_before_or_after() {
    let bed = bed().await;
    let ranged = bed.task(Some(range(1.0, 11.0)), None).await;
    // Inside: the shape (6–8 s) moves within 1–11 s.
    bed.apply(&ranged, vec![move_to(SHAPE, 5.5)]).await.unwrap();
    // Outside after the edit: 10–12 s ends past 11 s.
    let failure = bed
        .apply(&ranged, vec![move_to(SHAPE, 10.0)])
        .await
        .unwrap_err();
    assert_eq!(failure.edit, None, "the scope is checked on the whole call");
    let message = failure.error.to_string();
    assert_eq!(code(failure), ErrorCode::OutOfScope);
    assert!(message.contains("00:00:01.000–00:00:11.000"), "{message}");
    assert!(message.contains("00:00:10.000–00:00:12.000"), "{message}");
    // Straddling the edge before the edit: the clip (0–10 s) cannot change, even inside.
    let trim = AgentEdit::TrimItem {
        item: ItemRef::Id(id(CLIP)),
        edge: ItemEdge::End,
        to: secs(8.0),
        expected_revision: None,
    };
    let failure = bed.apply(&ranged, vec![trim]).await.unwrap_err();
    assert!(
        failure
            .error
            .to_string()
            .contains("only items lying entirely inside")
    );
    assert_eq!(code(failure), ErrorCode::OutOfScope);
    // Creation: inside is accepted, past the range refused.
    bed.apply(&ranged, vec![text(2.0, 3.0, None)])
        .await
        .unwrap();
    let failure = bed
        .apply(&ranged, vec![text(10.0, 12.0, None)])
        .await
        .unwrap_err();
    assert_eq!(code(failure), ErrorCode::OutOfScope);
    // Ripple edits move what follows: refused in a time-limited scope, with the alternative.
    let ripple = AgentEdit::RippleDelete {
        range: range(2.0, 2.5),
        tracks: None,
    };
    let failure = bed.apply(&ranged, vec![ripple]).await.unwrap_err();
    assert!(failure.error.to_string().contains("delete_items"));
    assert_eq!(code(failure), ErrorCode::OutOfScope);
}

#[tokio::test(start_paused = true)]
async fn tracks_outside_the_scope_are_refused_and_new_graphics_use_the_tasks_tracks() {
    let bed = bed().await;
    let v2 = bed.track(1);
    let only_v2 = bed.task(None, Some(vec![v2])).await;
    let failure = bed
        .apply(&only_v2, vec![move_to(SHAPE, 11.0)])
        .await
        .unwrap_err();
    assert!(failure.error.to_string().contains("V3"));
    assert_eq!(code(failure), ErrorCode::OutOfScope);
    // V2 holds the title (1–4 s) and the image (5–7 s): 4.2–4.8 s is free there.
    let report = bed
        .services
        .edits
        .apply(&only_v2, "add_text", &[text(4.2, 4.8, None)])
        .await
        .unwrap();
    let created = report.created[0][0];
    let (track, item) = report.project.sequences[0]
        .tracks
        .iter()
        .find_map(|track| Some((track.id, track.items.iter().find(|i| i.id == created)?)))
        .unwrap();
    assert_eq!(track, v2);
    assert_eq!(
        item.origin,
        Origin::AiTask {
            task_id: only_v2.task_id
        }
    );
    assert!(
        item.kind
            .property(se_domain::AnimatableProperty::Scale)
            .unwrap()
            .is_keyframed(),
        "the pop preset animated it"
    );
    // No free track of the task over the title: refused instead of creating a track.
    let failure = bed
        .apply(&only_v2, vec![text(2.0, 3.0, None)])
        .await
        .unwrap_err();
    assert_eq!(code(failure), ErrorCode::OutOfScope);
}

#[tokio::test(start_paused = true)]
async fn linked_partners_follow_moves_and_deletes() {
    let bed = bed().await;
    let whole = bed.task(None, None).await;
    // The clip (V1) and its sound (A1) are linked: the sound moves with it. The dissolve with
    // the next clip goes, so the next clip makes room first.
    bed.apply(
        &whole,
        vec![
            AgentEdit::DeleteItems {
                items: vec![(ItemRef::Id(id(2)), None)],
            },
            move_to(CLIP, 1.0),
        ],
    )
    .await
    .unwrap();
    let project = bed.project().await;
    let sound = project.sequences[0].tracks[3].items[0].clone();
    assert_eq!(sound.timeline_range, range(1.0, 11.0));
    bed.apply(
        &whole,
        vec![AgentEdit::DeleteItems {
            items: vec![(ItemRef::Id(id(CLIP)), None)],
        }],
    )
    .await
    .unwrap();
    assert!(
        find(&*bed.project().await, sound.id).is_none(),
        "the sound went too"
    );
}

#[tokio::test(start_paused = true)]
async fn a_call_is_atomic_and_names_its_failing_edit() {
    let bed = bed().await;
    let agent = bed.task(None, None).await;
    let revision = bed.project().await.revision;
    // The title is added, then the shape is moved onto the caption (8–10 s): refused.
    let failure = bed
        .apply(&agent, vec![text(11.0, 12.0, None), move_to(SHAPE, 7.5)])
        .await
        .unwrap_err();
    assert_eq!(failure.edit, Some(1));
    assert_eq!(code(failure), ErrorCode::EditRejected);
    assert_eq!(bed.project().await.revision, revision, "nothing changed");
}

#[tokio::test(start_paused = true)]
async fn stale_revisions_are_refused() {
    let bed = bed().await;
    let agent = bed.task(None, None).await;
    let revision = find(&*bed.project().await, id(TITLE)).unwrap().revision;
    let rename = |expected_revision| AgentEdit::UpdateItem {
        item: ItemRef::Id(id(TITLE)),
        patch: ItemPatch::Text(TextPatch {
            content: Some("Hello".to_owned()),
            ..TextPatch::default()
        }),
        expected_revision,
    };
    bed.apply(&agent, vec![rename(Some(revision))])
        .await
        .unwrap();
    let failure = bed
        .apply(&agent, vec![rename(Some(revision))])
        .await
        .unwrap_err();
    let AppError::Domain(error) = &failure.error else {
        panic!("a domain refusal is expected, got {:?}", failure.error);
    };
    assert_eq!(error.code(), se_domain::DomainErrorCode::RevisionConflict);
}

#[tokio::test(start_paused = true)]
async fn a_reverted_task_leaves_the_project_as_it_was() {
    let bed = bed().await;
    let agent = bed.task(None, None).await;
    let created = bed
        .services
        .edits
        .apply(&agent, "add_text", &[text(11.0, 12.0, None)])
        .await
        .unwrap()
        .created[0][0];
    bed.apply(&agent, vec![move_to(SHAPE, 5.5)]).await.unwrap();
    let task = bed.services.tasks.list(bed.project_id).await.unwrap()[0].clone();
    assert!(
        task.changeset_id.is_some(),
        "the first edit opened a changeset"
    );
    let changes = bed.services.edits.changes(&agent).await.unwrap();
    assert_eq!((changes.edits, changes.created.clone()), (2, vec![created]));
    assert_eq!(changes.changed, [id(SHAPE)]);
    let touched = bed
        .services
        .events
        .take()
        .into_iter()
        .rev()
        .find_map(|event| match event {
            CoreEvent::TaskChangesetUpdated { touched_items, .. } => Some(touched_items),
            _ => None,
        });
    let mut expected = vec![created, id(SHAPE)];
    expected.sort();
    assert_eq!(touched, Some(expected));

    let report = bed
        .services
        .tasks
        .revert(bed.project_id, agent.task_id)
        .await
        .unwrap();
    assert!(report.reverted);
    assert_eq!(report.conflicts, []);
    assert_eq!(
        without_revisions(bed.project().await.as_ref().clone()),
        without_revisions(bed.initial.clone())
    );
    let task = bed.services.tasks.list(bed.project_id).await.unwrap()[0].clone();
    assert_eq!(task.changeset_id, None);
    let again = bed
        .services
        .tasks
        .revert(bed.project_id, agent.task_id)
        .await;
    assert!(matches!(again, Err(AppError::NothingToRevert { .. })));
    // The revert is a user transaction: undo brings the task's edits back.
    bed.services.projects.undo(bed.project_id).await.unwrap();
    assert!(find(&*bed.project().await, created).is_some());
}

#[tokio::test(start_paused = true)]
async fn items_the_user_changed_since_are_reported_and_the_rest_is_reverted() {
    let bed = bed().await;
    let agent = bed.task(None, None).await;
    let created = bed
        .services
        .edits
        .apply(&agent, "add_text", &[text(11.0, 12.0, None)])
        .await
        .unwrap()
        .created[0][0];
    bed.apply(&agent, vec![move_to(SHAPE, 5.5)]).await.unwrap();
    // The user moves the shape again.
    let v3 = bed.track(2);
    bed.services
        .projects
        .move_items(
            bed.project_id,
            &[ItemPlacement {
                item_id: id(SHAPE),
                track_id: v3,
                start: millis(5200),
            }],
            PlacementMode::Overwrite,
        )
        .await
        .unwrap();
    let report = bed
        .services
        .tasks
        .revert(bed.project_id, agent.task_id)
        .await
        .unwrap();
    assert!(report.reverted);
    assert!(matches!(
        report.conflicts.as_slice(),
        [Conflict::ItemChanged { item, .. }] if *item == id(SHAPE)
    ));
    let project = bed.project().await;
    assert!(find(&project, created).is_none(), "the task's text is gone");
    assert_eq!(
        find(&project, id(SHAPE)).unwrap().timeline_range.start(),
        millis(5200),
        "the user's move stays"
    );
}

#[tokio::test(start_paused = true)]
async fn a_running_task_cannot_be_reverted_and_changesets_survive_reopening() {
    let bed = bed().await;
    bed.services.agent.push_script(ReplayScript::held());
    let running = bed.task(None, None).await;
    bed.apply(&running, vec![move_to(CAPTION, 8.5)])
        .await
        .unwrap();
    let refused = bed
        .services
        .tasks
        .revert(bed.project_id, running.task_id)
        .await;
    assert!(matches!(refused, Err(AppError::TaskStillActive { .. })));
    bed.services
        .tasks
        .cancel(bed.project_id, running.task_id)
        .await
        .unwrap();
    run_for(100).await;
    bed.services
        .projects
        .close_project(bed.project_id)
        .await
        .unwrap();
    bed.services
        .projects
        .open_project(Path::new(PROJECT_PATH))
        .await
        .unwrap();
    settle().await;
    let report = bed
        .services
        .tasks
        .revert(bed.project_id, running.task_id)
        .await
        .unwrap();
    assert!(report.reverted, "the changeset came back with the project");
    assert_eq!(
        find(&*bed.project().await, id(CAPTION))
            .unwrap()
            .timeline_range,
        range(8.0, 10.0)
    );
}
