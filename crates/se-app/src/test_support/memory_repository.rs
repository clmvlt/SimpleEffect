//! Fakes of [`ProjectRepository`] and [`ProjectStore`]: folders in a shared map.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use se_domain::{AnalysisKind, AssetId, Changeset, Project, TaskId};
use se_protocol::{Task, TaskEventRecord};

use super::{
    lock,
    memory_changesets::{StoredChangeset, apply_write, open_changesets},
    memory_tasks::InMemoryTaskStore,
    memory_transcripts::InMemoryTranscriptStore,
};
use crate::ports::{
    AnalysisStore, ChangesetWrite, JournalEntry, ProjectRepository, ProjectStore, StorageError,
    StoredAnalysis, StoredTranscript, TaskStore, TranscriptStore,
};

/// One fake project folder.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredFolder {
    /// The last saved snapshot.
    pub snapshot: Project,
    /// Entries appended since that snapshot.
    pub journal: Vec<JournalEntry>,
    /// Whether a store holds the lock.
    pub locked: bool,
    /// How many snapshots were saved through stores.
    pub snapshot_saves: usize,
    /// Analyses saved in its database.
    pub analyses: BTreeMap<(AssetId, AnalysisKind), StoredAnalysis>,
    /// AI tasks saved in its database, oldest first.
    pub tasks: Vec<Task>,
    /// Their event logs.
    pub task_events: BTreeMap<TaskId, Vec<TaskEventRecord>>,
    /// AI changesets saved in its database, open or reverted, in creation order.
    pub changesets: Vec<StoredChangeset>,
    /// Transcripts saved in its database.
    pub transcripts: BTreeMap<AssetId, StoredTranscript>,
}

#[derive(Debug, Default)]
struct State {
    folders: BTreeMap<PathBuf, StoredFolder>,
    trashed: Vec<PathBuf>,
    fail_journal: bool,
    fail_snapshots: bool,
}

/// Project folders in memory. Paths are used as given (no canonicalisation); clones share the
/// same folders, so a test keeps one to inspect what the service wrote.
#[derive(Debug, Clone, Default)]
pub struct InMemoryProjectRepository {
    state: Arc<Mutex<State>>,
}

impl InMemoryProjectRepository {
    /// Adds a folder holding `snapshot` and `journal`, unlocked.
    pub fn insert(&self, path: impl Into<PathBuf>, snapshot: Project, journal: Vec<JournalEntry>) {
        lock(&self.state).folders.insert(
            path.into(),
            StoredFolder {
                snapshot,
                journal,
                locked: false,
                snapshot_saves: 0,
                analyses: BTreeMap::new(),
                tasks: Vec::new(),
                task_events: BTreeMap::new(),
                changesets: Vec::new(),
                transcripts: BTreeMap::new(),
            },
        );
    }

    /// The folder at `path`, if any.
    #[must_use]
    pub fn folder(&self, path: impl AsRef<Path>) -> Option<StoredFolder> {
        lock(&self.state).folders.get(path.as_ref()).cloned()
    }

    /// Folders moved to the trash, in order.
    #[must_use]
    pub fn trashed(&self) -> Vec<PathBuf> {
        lock(&self.state).trashed.clone()
    }

    /// Makes journal appends fail (as a full disk would) until called with `false`.
    pub fn fail_journal_appends(&self, fail: bool) {
        lock(&self.state).fail_journal = fail;
    }

    /// Makes snapshot saves fail until called with `false`.
    pub fn fail_snapshot_saves(&self, fail: bool) {
        lock(&self.state).fail_snapshots = fail;
    }

    /// Stores an analysis in the folder at `path` (replacing the asset's previous one of that
    /// kind); nothing happens when no folder is there.
    pub fn set_analysis(&self, path: impl AsRef<Path>, analysis: StoredAnalysis) {
        if let Some(folder) = lock(&self.state).folders.get_mut(path.as_ref()) {
            folder
                .analyses
                .insert((analysis.asset_id, analysis.result.kind()), analysis);
        }
    }

    /// Stores a transcript in the folder at `path` (replacing the asset's previous one); nothing
    /// happens when no folder is there.
    pub fn set_transcript(&self, path: impl AsRef<Path>, transcript: StoredTranscript) {
        if let Some(folder) = lock(&self.state).folders.get_mut(path.as_ref()) {
            folder.transcripts.insert(transcript.asset_id, transcript);
        }
    }

    /// Replaces the stored tasks of the folder at `path` (as a previous core run left them);
    /// nothing happens when no folder is there.
    pub fn set_tasks(&self, path: impl AsRef<Path>, tasks: Vec<Task>) {
        if let Some(folder) = lock(&self.state).folders.get_mut(path.as_ref()) {
            folder.tasks = tasks;
        }
    }

    pub(super) fn with_folder<T>(
        &self,
        path: &Path,
        action: impl FnOnce(&mut StoredFolder) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let mut state = lock(&self.state);
        let folder = state
            .folders
            .get_mut(path)
            .ok_or_else(|| StorageError::NotFound {
                path: path.to_path_buf(),
            })?;
        action(folder)
    }
}

fn io_error(path: &Path) -> StorageError {
    StorageError::Io {
        action: "write",
        path: path.to_path_buf(),
        reason: "simulated failure".to_owned(),
    }
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn locate(&self, path: &Path) -> Result<PathBuf, StorageError> {
        self.with_folder(path, |_| Ok(path.to_path_buf()))
    }

    async fn create(&self, parent_dir: &Path, project: &Project) -> Result<PathBuf, StorageError> {
        let path = parent_dir.join(format!("{}.seproj", project.name));
        if lock(&self.state).folders.contains_key(&path) {
            return Err(StorageError::AlreadyExists { path });
        }
        self.insert(path.clone(), project.clone(), Vec::new());
        Ok(path)
    }

    async fn duplicate(&self, source: &Path, project: &Project) -> Result<PathBuf, StorageError> {
        let analyses = self.with_folder(source, |folder| Ok(folder.analyses.clone()))?;
        let parent = source.parent().unwrap_or_else(|| Path::new(""));
        let state = lock(&self.state);
        let path = (1..=u32::MAX)
            .map(|number| match number {
                1 => parent.join(format!("{}.seproj", project.name)),
                _ => parent.join(format!("{} {number}.seproj", project.name)),
            })
            .find(|candidate| !state.folders.contains_key(candidate))
            .unwrap_or_default();
        drop(state);
        self.insert(path.clone(), project.clone(), Vec::new());
        self.with_folder(&path, |folder| {
            folder.analyses = analyses;
            Ok(())
        })?;
        Ok(path)
    }

    async fn open(&self, path: &Path) -> Result<Box<dyn ProjectStore>, StorageError> {
        self.with_folder(path, |folder| {
            if folder.locked {
                return Err(StorageError::Locked {
                    path: path.to_path_buf(),
                });
            }
            folder.locked = true;
            Ok(())
        })?;
        Ok(Box::new(InMemoryProjectStore {
            repository: self.clone(),
            path: path.to_path_buf(),
        }))
    }

    async fn trash(&self, path: &Path) -> Result<(), StorageError> {
        let mut state = lock(&self.state);
        if state.folders.remove(path).is_none() {
            return Err(StorageError::NotFound {
                path: path.to_path_buf(),
            });
        }
        state.trashed.push(path.to_path_buf());
        Ok(())
    }
}

/// An open fake folder.
struct InMemoryProjectStore {
    repository: InMemoryProjectRepository,
    path: PathBuf,
}

#[async_trait]
impl ProjectStore for InMemoryProjectStore {
    fn path(&self) -> &Path {
        &self.path
    }

    async fn load_snapshot(&self) -> Result<Project, StorageError> {
        self.repository
            .with_folder(&self.path, |folder| Ok(folder.snapshot.clone()))
    }

    async fn read_journal_since(&self, revision: u64) -> Result<Vec<JournalEntry>, StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            Ok(folder
                .journal
                .iter()
                .filter(|entry| entry.revision > revision)
                .cloned()
                .collect())
        })
    }

    async fn append_journal(&self, entry: &JournalEntry) -> Result<(), StorageError> {
        let fail = lock(&self.repository.state).fail_journal;
        self.repository.with_folder(&self.path, |folder| {
            if fail {
                return Err(io_error(&self.path));
            }
            folder.journal.push(entry.clone());
            Ok(())
        })
    }

    async fn append_journal_with_changeset(
        &self,
        entry: &JournalEntry,
        change: &ChangesetWrite,
    ) -> Result<(), StorageError> {
        let fail = lock(&self.repository.state).fail_journal;
        self.repository.with_folder(&self.path, |folder| {
            if fail {
                return Err(io_error(&self.path));
            }
            apply_write(&mut folder.changesets, change)?;
            folder.journal.push(entry.clone());
            Ok(())
        })
    }

    async fn load_changesets(&self) -> Result<Vec<Changeset>, StorageError> {
        self.repository
            .with_folder(&self.path, |folder| Ok(open_changesets(&folder.changesets)))
    }

    async fn save_snapshot(&self, project: &Project) -> Result<(), StorageError> {
        let fail = lock(&self.repository.state).fail_snapshots;
        self.repository.with_folder(&self.path, |folder| {
            if fail {
                return Err(io_error(&self.path));
            }
            folder.snapshot = project.clone();
            folder
                .journal
                .retain(|entry| entry.revision > project.revision);
            folder.snapshot_saves += 1;
            Ok(())
        })
    }

    fn analyses(&self) -> Arc<dyn AnalysisStore> {
        Arc::new(InMemoryAnalysisStore {
            repository: self.repository.clone(),
            path: self.path.clone(),
        })
    }

    fn tasks(&self) -> Arc<dyn TaskStore> {
        Arc::new(InMemoryTaskStore::new(
            self.repository.clone(),
            self.path.clone(),
        ))
    }

    fn transcripts(&self) -> Arc<dyn TranscriptStore> {
        Arc::new(InMemoryTranscriptStore::new(
            self.repository.clone(),
            self.path.clone(),
        ))
    }

    async fn close(&self) -> Result<(), StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            folder.locked = false;
            Ok(())
        })
    }
}

/// The analyses of a fake folder.
struct InMemoryAnalysisStore {
    repository: InMemoryProjectRepository,
    path: PathBuf,
}

#[async_trait]
impl AnalysisStore for InMemoryAnalysisStore {
    async fn save_analysis(&self, analysis: &StoredAnalysis) -> Result<(), StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            let key = (analysis.asset_id, analysis.result.kind());
            folder.analyses.insert(key, analysis.clone());
            Ok(())
        })
    }

    async fn load_analysis(
        &self,
        asset_id: AssetId,
        kind: AnalysisKind,
    ) -> Result<Option<StoredAnalysis>, StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            Ok(folder.analyses.get(&(asset_id, kind)).cloned())
        })
    }
}
