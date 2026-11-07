//! [`SqliteAppLibrary`]: the app's `library.db` (recent projects, settings).

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use rusqlite::{Connection, OptionalExtension, params};
use se_app::ports::{AppLibraryRepository, RecentProjectRecord, StorageError};
use se_domain::{Format, ProjectId};

use crate::{
    io_errors::{blocking, io_error, sqlite_error, utf8_path},
    sqlite::open_database,
};

/// Schema migrations of `library.db`. Append, never edit.
const MIGRATIONS: &[&str] = &["
CREATE TABLE recent_projects (
    path TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    format TEXT NOT NULL,
    last_opened_at INTEGER NOT NULL
);
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"];

/// The app library in a SQLite file of the app data folder.
pub struct SqliteAppLibrary {
    connection: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl SqliteAppLibrary {
    /// Opens (or creates, with its folder) the library at `path`.
    ///
    /// # Errors
    /// [`StorageError::SchemaTooNew`]; IO errors.
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder)
                .map_err(|error| io_error("create the folder", folder, &error))?;
        }
        Ok(Self {
            connection: Arc::new(Mutex::new(open_database(path, MIGRATIONS)?)),
            path: path.to_path_buf(),
        })
    }

    /// Runs `work` with the connection on the blocking pool.
    async fn with_connection<T: Send + 'static>(
        &self,
        action: &'static str,
        work: impl FnOnce(&Connection) -> rusqlite::Result<T> + Send + 'static,
    ) -> Result<T, StorageError> {
        let connection = Arc::clone(&self.connection);
        let path = self.path.clone();
        blocking(&self.path, move || {
            let connection = connection.lock().unwrap_or_else(PoisonError::into_inner);
            work(&connection).map_err(|error| sqlite_error(action, &path, &error))
        })
        .await
    }
}

#[async_trait]
impl AppLibraryRepository for SqliteAppLibrary {
    async fn upsert_recent(&self, record: &RecentProjectRecord) -> Result<(), StorageError> {
        let path = utf8_path("remember", &record.path)?.to_owned();
        let format = format_json(&record.format);
        let (id, name, opened) = (
            record.id.to_string(),
            record.name.clone(),
            record.last_opened_at,
        );
        self.with_connection("update the recent projects in", move |connection| {
            connection
                .execute(
                    "INSERT INTO recent_projects (path, project_id, name, format, last_opened_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT (path) DO UPDATE SET project_id = ?2, name = ?3, format = ?4,
                         last_opened_at = ?5",
                    params![path, id, name, format, opened],
                )
                .map(drop)
        })
        .await
    }

    async fn list_recent(&self) -> Result<Vec<RecentProjectRecord>, StorageError> {
        let rows = self
            .with_connection("read the recent projects in", |connection| {
                let mut statement = connection.prepare(
                    "SELECT path, project_id, name, format, last_opened_at FROM recent_projects
                     ORDER BY last_opened_at DESC",
                )?;
                statement
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, i64>(4)?,
                        ))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()
            })
            .await?;
        // Rows that no longer parse (written by a future version) are skipped, not fatal.
        Ok(rows
            .into_iter()
            .filter_map(|(path, id, name, format, last_opened_at)| {
                Some(RecentProjectRecord {
                    path: PathBuf::from(path),
                    id: ProjectId::parse(&id).ok()?,
                    name,
                    format: serde_json::from_str::<Format>(&format).ok()?,
                    last_opened_at,
                })
            })
            .collect())
    }

    async fn remove_recent(&self, path: &Path) -> Result<(), StorageError> {
        let path = utf8_path("forget", path)?.to_owned();
        self.with_connection("update the recent projects in", move |connection| {
            connection
                .execute("DELETE FROM recent_projects WHERE path = ?1", [path])
                .map(drop)
        })
        .await
    }

    async fn setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let key = key.to_owned();
        self.with_connection("read a setting in", move |connection| {
            connection
                .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                    row.get(0)
                })
                .optional()
        })
        .await
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let (key, value) = (key.to_owned(), value.to_owned());
        self.with_connection("write a setting in", move |connection| {
            connection
                .execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT (key) DO UPDATE SET value = ?2",
                    [key, value],
                )
                .map(drop)
        })
        .await
    }
}

/// A format as stored: the same JSON as in `project.json`.
fn format_json(format: &Format) -> String {
    // A `Format` is plain numbers: serialising it cannot fail.
    serde_json::to_string(format).unwrap_or_default()
}
