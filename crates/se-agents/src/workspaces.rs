//! [`FsAgentWorkspaces`]: the empty working folder of each agent run,
//! `<app data>/agent-workdirs/<task id>/`, created when a run starts and removed when it ends.
//!
//! The same task always gets the same folder: CLIs that key their sessions by working folder
//! (Claude Code does) find them again for follow-ups.

use std::{io, path::PathBuf};

use async_trait::async_trait;
use se_app::ports::{AgentError, AgentWorkspaces};
use se_domain::TaskId;

/// Working folders under one root.
#[derive(Debug, Clone)]
pub struct FsAgentWorkspaces {
    root: PathBuf,
}

impl FsAgentWorkspaces {
    /// Folders under `root` (created when needed).
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// The folder of `task_id`.
    #[must_use]
    pub fn folder(&self, task_id: TaskId) -> PathBuf {
        self.root.join(task_id.to_string())
    }

    /// Removes the folders a previous core left behind (it stopped while agents ran). Call once
    /// at startup, before any run.
    pub async fn remove_leftovers(&self) {
        let mut entries = match tokio::fs::read_dir(&self.root).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return,
            Err(error) => {
                tracing::warn!(%error, root = %self.root.display(), "listing agent folders failed");
                return;
            }
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Err(error) = tokio::fs::remove_dir_all(&path).await {
                tracing::warn!(%error, path = %path.display(), "removing a leftover agent folder failed");
            }
        }
    }
}

#[async_trait]
impl AgentWorkspaces for FsAgentWorkspaces {
    async fn prepare(&self, task_id: TaskId) -> Result<PathBuf, AgentError> {
        let folder = self.folder(task_id);
        match tokio::fs::remove_dir_all(&folder).await {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(AgentError::Failed {
                    message: format!("emptying {} failed: {error}", folder.display()),
                });
            }
        }
        tokio::fs::create_dir_all(&folder)
            .await
            .map_err(|error| AgentError::Failed {
                message: format!(
                    "creating the agent's working folder {} failed: {error}; check the free space \
                     and permissions of the app data folder",
                    folder.display()
                ),
            })?;
        Ok(folder)
    }

    async fn release(&self, task_id: TaskId) {
        let folder = self.folder(task_id);
        if let Err(error) = tokio::fs::remove_dir_all(&folder).await
            && error.kind() != io::ErrorKind::NotFound
        {
            tracing::warn!(%error, path = %folder.display(), "removing an agent folder failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn folders_are_created_empty_and_removed() {
        let root = tempfile::tempdir().unwrap();
        let workspaces = FsAgentWorkspaces::new(root.path().join("agent-workdirs"));
        let task = TaskId::from_test_index(1);
        let folder = workspaces.prepare(task).await.unwrap();
        assert_eq!(folder, workspaces.folder(task));
        std::fs::write(folder.join("left.txt"), "x").unwrap();
        let again = workspaces.prepare(task).await.unwrap();
        assert_eq!(std::fs::read_dir(&again).unwrap().count(), 0);
        workspaces.release(task).await;
        assert!(!folder.exists());
        workspaces.release(task).await;
    }

    #[tokio::test]
    async fn leftovers_of_a_previous_run_are_removed() {
        let root = tempfile::tempdir().unwrap();
        let workspaces = FsAgentWorkspaces::new(root.path().to_path_buf());
        let folder = workspaces
            .prepare(TaskId::from_test_index(2))
            .await
            .unwrap();
        workspaces.remove_leftovers().await;
        assert!(!folder.exists());
        FsAgentWorkspaces::new(root.path().join("missing"))
            .remove_leftovers()
            .await;
    }
}
