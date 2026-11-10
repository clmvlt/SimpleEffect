//! Load scenario of prompt 29 (`docs/performance.md`): five AI tasks editing at once while the
//! user edits the same project. Every edit must land whole, in one order, with nothing lost,
//! saved, and each task must stay revertible on its own.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use se_app::{
    AgentContext,
    agent_edits::{AgentEdit, NewAgentItem},
    test_support::{MediaTestbed, ReplayScript, probed_video, settle},
};
use se_domain::{
    Flicks, ItemId, ItemKind, Origin, Project, ProjectId, SequenceId, TaskId, TextItem, TimeRange,
    TrackId,
    test_support::{example_project, millis},
};
use se_protocol::{ItemPlacement, PlacementMode, SubmitTaskRequest, TaskScope};

const PROJECT_PATH: &str = "/films/Load.seproj";
const TASKS: usize = 5;
const EDITS_PER_TASK: usize = 20;
const USER_MOVES: usize = 20;
/// The example project's title (V2, 1–4 s): the user nudges it back and forth.
const TITLE: u64 = 3;

fn secs(seconds: f64) -> Flicks {
    Flicks::from_seconds_f64(seconds).unwrap()
}

struct Bed {
    services: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
    tracks: Vec<TrackId>,
}

async fn bed() -> Bed {
    let services = MediaTestbed::new(Path::new("/films"));
    services.tasks.start().await.unwrap();
    let mut settings = services.tasks.ai_settings();
    settings.max_concurrent_tasks = u8::try_from(TASKS).unwrap();
    services.tasks.update_ai_settings(settings).await.unwrap();
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
    Bed {
        project_id: session.snapshot.project.id,
        sequence_id: project.sequences[0].id,
        tracks: project.sequences[0].tracks.iter().map(|t| t.id).collect(),
        services,
    }
}

impl Bed {
    /// Starts `TASKS` held tasks over the whole sequence and returns their agents' contexts.
    async fn running_tasks(&self) -> Vec<AgentContext> {
        let scope = TaskScope {
            sequence_id: self.sequence_id,
            range: None,
            tracks: None,
        };
        let mut agents = Vec::new();
        for index in 0..TASKS {
            self.services.agent.push_script(ReplayScript::held());
            let task = self
                .services
                .tasks
                .submit(SubmitTaskRequest {
                    project_id: self.project_id,
                    prompt: format!("Task {index}"),
                    scope: scope.clone(),
                    provider: None,
                    model: None,
                    effort: None,
                })
                .await
                .unwrap();
            agents.push(AgentContext {
                project_id: self.project_id,
                task_id: task.id,
                run: 0,
                scope: scope.clone(),
            });
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        agents
    }

    /// Agent `index` adds `EDITS_PER_TASK` texts on V3, in its own stretch after the content.
    async fn agent_edits(&self, agent: &AgentContext, index: usize) {
        for edit in 0..EDITS_PER_TASK {
            let start = 20.0 + f64::from(u32::try_from(index * EDITS_PER_TASK + edit).unwrap());
            let text = AgentEdit::AddItem(Box::new(NewAgentItem {
                kind: ItemKind::Text(TextItem::new("Abonne-toi")),
                range: TimeRange::from_start_end(secs(start), secs(start + 0.5)).unwrap(),
                track: Some(self.tracks[2]),
                presets: Vec::new(),
            }));
            self.services
                .edits
                .apply(agent, "add_items", &[text])
                .await
                .unwrap();
            tokio::task::yield_now().await;
        }
    }

    /// The user moves the title between 1 s and 1.5 s, `USER_MOVES` times.
    async fn user_edits(&self) {
        for step in 0..USER_MOVES {
            let start = if step % 2 == 0 {
                millis(1500)
            } else {
                millis(1000)
            };
            self.services
                .projects
                .move_items(
                    self.project_id,
                    &[ItemPlacement {
                        item_id: ItemId::from_test_index(TITLE),
                        track_id: self.tracks[1],
                        start,
                    }],
                    PlacementMode::Overwrite,
                )
                .await
                .unwrap();
            tokio::task::yield_now().await;
        }
    }

    async fn project(&self) -> std::sync::Arc<Project> {
        self.services
            .projects
            .project(self.project_id)
            .await
            .unwrap()
    }
}

fn items_of(project: &Project, task: TaskId) -> usize {
    project
        .items()
        .filter(|item| item.origin == Origin::AiTask { task_id: task })
        .count()
}

#[tokio::test(start_paused = true)]
async fn five_tasks_and_the_user_edit_at_once_without_losing_anything() {
    let bed = bed().await;
    let initial_revision = bed.project().await.revision;
    let agents = bed.running_tasks().await;

    // The five agents and the user race each other on the same actor.
    tokio::join!(
        bed.agent_edits(&agents[0], 0),
        bed.agent_edits(&agents[1], 1),
        bed.agent_edits(&agents[2], 2),
        bed.agent_edits(&agents[3], 3),
        bed.agent_edits(&agents[4], 4),
        bed.user_edits(),
    );

    let project = bed.project().await;
    let commits = TASKS * EDITS_PER_TASK + USER_MOVES;
    assert_eq!(project.revision, initial_revision + commits as u64);
    for agent in &agents {
        assert_eq!(items_of(&project, agent.task_id), EDITS_PER_TASK);
    }

    // Everything reaches the disk: journal entries since the last snapshot, then a snapshot.
    tokio::time::sleep(Duration::from_secs(5)).await;
    let folder = bed.services.repository.folder(PROJECT_PATH).unwrap();
    assert_eq!(folder.snapshot.revision, project.revision);

    // One task reverted alone: its texts go, the others' and the user's moves stay.
    bed.services.agent.release(TASKS);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let report = bed
        .services
        .tasks
        .revert(bed.project_id, agents[0].task_id)
        .await
        .unwrap();
    assert!(report.reverted);
    let after = bed.project().await;
    assert_eq!(items_of(&after, agents[0].task_id), 0);
    assert_eq!(items_of(&after, agents[1].task_id), EDITS_PER_TASK);
    let title = after
        .items()
        .find(|item| item.id == ItemId::from_test_index(TITLE))
        .unwrap();
    assert_eq!(title.timeline_range.start(), millis(1000));
}
