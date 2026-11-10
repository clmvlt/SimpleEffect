//! Tasks follow their project: loaded when it opens (those a stopped core left queued or running
//! fail as interrupted), cancelled when it closes — or interrupted when the core stops
//! ([`TaskService::begin_shutdown`], prompt 29).

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use se_domain::ProjectId;
use se_protocol::{CoreEvent, TaskStatus};

use super::{OpenProject, TaskService, task_state};
use crate::{project_actor::ProjectActorHandle, project_service::ProjectObserver};

/// How long closing a project waits for its running agents to stop.
const CLOSE_GRACE: Duration = Duration::from_secs(8);

impl TaskService {
    /// Tells the service the core is stopping: tasks stopped by the projects' closing from now
    /// on are marked interrupted (retry them later), not cancelled by the user.
    pub fn begin_shutdown(&self) {
        self.shared
            .shutting_down
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[async_trait]
impl ProjectObserver for TaskService {
    async fn project_opened(&self, handle: &ProjectActorHandle) {
        let project_id = handle.project_id();
        let store = Arc::clone(handle.tasks());
        let mut tasks = match store.load_tasks().await {
            Ok(tasks) => tasks,
            Err(error) => {
                tracing::warn!(%project_id, %error, "reading the project's AI tasks failed");
                Vec::new()
            }
        };
        let now = self.shared.now();
        for task in &mut tasks {
            // A copy made with « Duplicate » carries its source's tasks: they belong here now.
            task.project_id = project_id;
            if task_state::interrupt(task, now) {
                tracing::info!(task = %task.id, "AI task interrupted by a previous stop");
                if let Err(error) = store.save_task(task).await {
                    tracing::warn!(task = %task.id, %error, "saving an interrupted task failed");
                }
            }
        }
        let mut state = self.shared.state.lock().await;
        state.projects.insert(
            project_id,
            OpenProject {
                handle: handle.clone(),
                tasks,
            },
        );
    }

    async fn project_closing(&self, project_id: ProjectId) {
        let now = self.shared.now();
        let ending = {
            let mut state = self.shared.state.lock().await;
            let (removed, ending) = state.queue.cancel_project(project_id);
            if let Some(project) = state.projects.get_mut(&project_id) {
                let store = Arc::clone(project.handle.tasks());
                for task in project
                    .tasks
                    .iter_mut()
                    .filter(|task| removed.contains(&task.id))
                {
                    if self.shared.shutting_down() {
                        task_state::interrupt(task, now);
                    } else {
                        task_state::finish_run(task, TaskStatus::Cancelled, now);
                    }
                    if let Err(error) = store.save_task(task).await {
                        tracing::warn!(task = %task.id, %error, "saving a stopped task failed");
                    }
                    self.shared
                        .deps
                        .events
                        .publish(CoreEvent::TaskUpdated { task: task.clone() });
                }
            }
            ending
        };
        let all_ended = async {
            for gone in ending {
                // An error only means the run's task is gone, which is what we wait for.
                let _ = gone.await;
            }
        };
        if tokio::time::timeout(CLOSE_GRACE, all_ended).await.is_err() {
            tracing::warn!(%project_id, "AI tasks did not stop within {CLOSE_GRACE:?}");
        }
        self.shared.state.lock().await.projects.remove(&project_id);
    }
}
