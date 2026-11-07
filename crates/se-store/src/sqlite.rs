//! Opening SQLite databases: pragmas and versioned schema migrations (`PRAGMA user_version`).

use std::{path::Path, time::Duration};

use rusqlite::Connection;
use se_app::ports::StorageError;

use crate::io_errors::sqlite_error;

/// How long a statement waits for another connection's lock before failing.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Opens (creating if needed) the database at `path` in WAL mode with full durability, and
/// applies the `migrations` it has not seen yet, in one transaction.
///
/// `migrations[i]` is the SQL bringing the schema from version `i` to `i + 1`; the schema version
/// is kept in `PRAGMA user_version`. Never edit a migration once released: append a new one.
///
/// # Errors
/// [`StorageError::SchemaTooNew`] when the database was created by a newer version; IO errors.
pub fn open_database(path: &Path, migrations: &[&str]) -> Result<Connection, StorageError> {
    let mut connection =
        Connection::open(path).map_err(|error| sqlite_error("open", path, &error))?;
    configure(&connection).map_err(|error| sqlite_error("configure", path, &error))?;
    migrate(&mut connection, path, migrations)?;
    Ok(connection)
}

fn configure(connection: &Connection) -> rusqlite::Result<()> {
    connection.busy_timeout(BUSY_TIMEOUT)?;
    // WAL: readers never block the writer; FULL: a commit is on disk before it returns, so a
    // journaled transaction survives a crash or a power cut.
    connection
        .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    connection.pragma_update(None, "foreign_keys", "ON")
}

fn migrate(
    connection: &mut Connection,
    path: &Path,
    migrations: &[&str],
) -> Result<(), StorageError> {
    let error = |error: rusqlite::Error| sqlite_error("upgrade the schema of", path, &error);
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(error)?;
    let supported = u32::try_from(migrations.len()).unwrap_or(u32::MAX);
    if version > supported {
        return Err(StorageError::SchemaTooNew {
            path: path.to_path_buf(),
            found: version,
            supported,
        });
    }
    if version == supported {
        return Ok(());
    }
    let transaction = connection.transaction().map_err(error)?;
    for sql in migrations
        .iter()
        .skip(usize::try_from(version).unwrap_or(usize::MAX))
    {
        transaction.execute_batch(sql).map_err(error)?;
    }
    transaction
        .pragma_update(None, "user_version", supported)
        .map_err(error)?;
    transaction.commit().map_err(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIGRATIONS: &[&str] = &[
        "CREATE TABLE a (id INTEGER PRIMARY KEY);",
        "CREATE TABLE b (id INTEGER PRIMARY KEY);",
    ];

    fn tables(connection: &Connection) -> Vec<String> {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn applies_only_new_migrations() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("test.db");
        drop(open_database(&path, &MIGRATIONS[..1]).unwrap());
        let connection = open_database(&path, MIGRATIONS).unwrap();
        assert_eq!(tables(&connection), ["a", "b"]);
        let mode: String = connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
    }

    #[test]
    fn refuses_a_database_from_a_newer_version() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("test.db");
        drop(open_database(&path, MIGRATIONS).unwrap());
        assert!(matches!(
            open_database(&path, &MIGRATIONS[..1]),
            Err(StorageError::SchemaTooNew {
                found: 2,
                supported: 1,
                ..
            })
        ));
    }
}
