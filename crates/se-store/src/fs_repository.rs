//! [`FsProjectRepository`]: project folders on the local file system.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use se_app::ports::{ProjectRepository, ProjectStore, StorageError};
use se_domain::Project;

use crate::{
    folder_name::project_folder_name,
    fs_store::FsProjectStore,
    io_errors::{blocking, io_error},
    layout::ProjectLayout,
    platform,
    project_db::ProjectDatabase,
    project_lock::ProjectLock,
    snapshot_file::write_snapshot,
};

/// How many numbered names (`Name 2`, `Name 3`…) a copy may try.
const MAX_COPY_NUMBER: u32 = 999;

/// Project folders on disk (ADR-0006). Stateless: every open project owns its
/// [`FsProjectStore`].
#[derive(Debug, Clone, Copy, Default)]
pub struct FsProjectRepository;

#[async_trait]
impl ProjectRepository for FsProjectRepository {
    async fn locate(&self, path: &Path) -> Result<PathBuf, StorageError> {
        let path = path.to_path_buf();
        blocking(&path.clone(), move || locate(&path)).await
    }

    async fn create(&self, parent_dir: &Path, project: &Project) -> Result<PathBuf, StorageError> {
        let (parent_dir, project) = (parent_dir.to_path_buf(), project.clone());
        blocking(&parent_dir.clone(), move || {
            fs::create_dir_all(&parent_dir)
                .map_err(|error| io_error("create the folder", &parent_dir, &error))?;
            let parent = canonical(&parent_dir)?;
            let root = parent.join(project_folder_name(&project.name, 1));
            create_folder(&root)?;
            fill_new_folder(&root, &project, None)
        })
        .await
    }

    async fn duplicate(&self, source: &Path, project: &Project) -> Result<PathBuf, StorageError> {
        let (source, project) = (source.to_path_buf(), project.clone());
        blocking(&source.clone(), move || {
            let source = locate(&source)?;
            let parent = source.parent().unwrap_or(&source).to_path_buf();
            let root = create_numbered_folder(&parent, &project.name)?;
            fill_new_folder(&root, &project, Some(&ProjectLayout::new(&source)))
        })
        .await
    }

    async fn open(&self, path: &Path) -> Result<Box<dyn ProjectStore>, StorageError> {
        let path = path.to_path_buf();
        blocking(&path.clone(), move || {
            let layout = ProjectLayout::new(locate(&path)?);
            let lock = ProjectLock::acquire(&layout.lock(), layout.root())?;
            let database = ProjectDatabase::open(&layout.database())?;
            Ok(Box::new(FsProjectStore::new(layout, database, lock)) as Box<dyn ProjectStore>)
        })
        .await
    }

    async fn trash(&self, path: &Path) -> Result<(), StorageError> {
        let path = path.to_path_buf();
        blocking(&path.clone(), move || {
            let root = locate(&path)?;
            platform::move_to_trash(&root).map_err(|error| StorageError::Io {
                action: "move to the trash",
                path: root,
                reason: error.to_string(),
            })
        })
        .await
    }
}

/// The canonical path of the project folder at `path`.
fn locate(path: &Path) -> Result<PathBuf, StorageError> {
    let not_found = || StorageError::NotFound {
        path: path.to_path_buf(),
    };
    let root = dunce::canonicalize(path).map_err(|_| not_found())?;
    if ProjectLayout::new(&root).snapshot().is_file() {
        Ok(root)
    } else {
        Err(not_found())
    }
}

/// The canonical form of an existing folder (no `\\?\` prefix on Windows).
fn canonical(path: &Path) -> Result<PathBuf, StorageError> {
    dunce::canonicalize(path).map_err(|error| io_error("resolve", path, &error))
}

fn create_folder(root: &Path) -> Result<(), StorageError> {
    fs::create_dir(root).map_err(|error| match error.kind() {
        io::ErrorKind::AlreadyExists => StorageError::AlreadyExists {
            path: root.to_path_buf(),
        },
        _ => io_error("create the folder", root, &error),
    })
}

/// Creates `<parent>/<name>.seproj`, or `<name> 2.seproj`, `<name> 3.seproj`… when taken.
fn create_numbered_folder(parent: &Path, name: &str) -> Result<PathBuf, StorageError> {
    for number in 1..=MAX_COPY_NUMBER {
        let root = parent.join(project_folder_name(name, number));
        match create_folder(&root) {
            Ok(()) => return Ok(root),
            Err(StorageError::AlreadyExists { .. }) => {}
            Err(error) => return Err(error),
        }
    }
    Err(StorageError::AlreadyExists {
        path: parent.join(project_folder_name(name, MAX_COPY_NUMBER)),
    })
}

/// Fills the folder just created at `root`: database (copied from `source` without its journal,
/// or new), `cache/`, `exports/`, then `project.json` last, so a folder without it is never
/// taken for a project. On failure the half-made folder is removed.
fn fill_new_folder(
    root: &Path,
    project: &Project,
    source: Option<&ProjectLayout>,
) -> Result<PathBuf, StorageError> {
    let layout = ProjectLayout::new(root);
    let filled = (|| {
        if let Some(source) = source.filter(|source| source.database().is_file()) {
            ProjectDatabase::open(&source.database())?.copy_to(&layout.database())?;
        }
        let database = ProjectDatabase::open(&layout.database())?;
        database.truncate_through(u64::MAX)?;
        drop(database);
        for folder in [layout.cache(), layout.exports()] {
            fs::create_dir(&folder)
                .map_err(|error| io_error("create the folder", &folder, &error))?;
        }
        write_snapshot(&layout, project)?;
        canonical(root)
    })();
    if filled.is_err() {
        // Only this call's own, just-created folder: nothing of the user's is removed.
        if let Err(error) = fs::remove_dir_all(root) {
            tracing::warn!(%error, path = %root.display(), "removing a half-created project failed");
        }
    }
    filled
}
