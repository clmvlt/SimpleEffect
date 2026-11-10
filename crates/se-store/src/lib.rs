//! # se-store
//!
//! Persistence adapter of SimpleEffect (ADR-0006, `docs/architecture.md` §11):
//! - [`FsProjectRepository`] implements `se_app::ports::ProjectRepository` with `.seproj`
//!   folders: `project.json` (atomic writes, 3 backups, migrations), `project.db` (SQLite WAL:
//!   operation journal, tasks, transcripts with FTS5, analyses), `cache/`, `exports/`, and a
//!   `.lock` file held with an OS lock while the project is open.
//! - [`SqliteAppLibrary`] implements `se_app::ports::AppLibraryRepository` with `library.db`
//!   in the app data folder (recent projects, settings).
//! - [`FsTemplateLibrary`] and [`FsBrandKitLibrary`] (prompt 25) implement the template library
//!   (`<app data>/templates/`, `.setemplate` zip files) and the brand kits
//!   (`<app data>/brand-kits/`).
//!
//! It decides nothing about *when* to save: the project actor of se-app does. Blocking file and
//! SQLite calls run on tokio's blocking pool.
//!
//! Allowed workspace dependencies: `se-app`, `se-domain` (see `docs/architecture.md` §3).

mod document_migrations;
mod folder_name;
mod fs_analysis_store;
mod fs_brand_kits;
mod fs_diagnostics;
mod fs_repository;
mod fs_store;
mod fs_task_store;
mod fs_template_library;
mod fs_transcript_store;
mod io_errors;
mod layout;
mod library;
mod library_files;
mod platform;
mod project_db;
mod project_lock;
mod snapshot_file;
mod sqlite;

pub use document_migrations::{
    MIGRATED_BRAND_CAPTION_STYLE_ID, MIGRATED_BRAND_KIT_NAME, MIGRATED_CAPTION_STYLE_ID,
    MIGRATED_TEMPLATE_SOURCE,
};
pub use fs_brand_kits::FsBrandKitLibrary;
pub use fs_diagnostics::FsDiagnosticsFiles;
pub use fs_repository::FsProjectRepository;
pub use fs_template_library::FsTemplateLibrary;
pub use layout::{BACKUP_COUNT, PROJECT_EXTENSION, ProjectLayout};
pub use library::SqliteAppLibrary;
pub use project_db::ProjectDatabase;
