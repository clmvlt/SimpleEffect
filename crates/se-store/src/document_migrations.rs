//! Upgrading stored documents to the current format (`schemaVersion`, ADR-0006).
//!
//! A migration is a pure function from the JSON of version `N` to the JSON of version `N + 1`.
//! Version 1 is the base format, so there is no migration yet. When the document format changes
//! (the snapshot in `se-domain/src/document/snapshots/` changes):
//! 1. bump `se_domain::CURRENT_SCHEMA_VERSION`;
//! 2. add `fn migrate_v1_to_v2(document: Value) -> Result<Value, String>` here and append it to
//!    [`MIGRATIONS`];
//! 3. keep `tests/fixtures/project-v1.json` and add a `project-v2.json` fixture.

use serde_json::Value;

/// One step: the document of version `N` (index `N - 1`) to version `N + 1`.
pub type DocumentMigration = fn(Value) -> Result<Value, String>;

/// Every migration, in order: `MIGRATIONS[0]` turns version 1 into version 2, and so on.
/// Its length is always `CURRENT_SCHEMA_VERSION - 1` (checked by a test).
pub const MIGRATIONS: &[DocumentMigration] = &[];

/// Field holding the format version.
const VERSION_FIELD: &str = "schemaVersion";

/// Why a document could not be migrated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// `schemaVersion` is missing or not a positive integer.
    MissingVersion,
    /// The document comes from a newer version of the app.
    TooNew {
        /// Version found in the document.
        found: u32,
        /// Newest version known.
        supported: u32,
    },
    /// A migration refused the document.
    Failed {
        /// Version the failing migration starts from.
        from: u32,
        /// Its explanation.
        reason: String,
    },
}

/// Upgrades `document` to `current` with `migrations` (see [`MIGRATIONS`]).
///
/// # Errors
/// [`MigrationError`] when the version is missing or too new, or a migration fails.
pub fn migrate_document(
    mut document: Value,
    migrations: &[DocumentMigration],
    current: u32,
) -> Result<Value, MigrationError> {
    let version = document
        .get(VERSION_FIELD)
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .filter(|version| *version >= 1)
        .ok_or(MigrationError::MissingVersion)?;
    if version > current {
        return Err(MigrationError::TooNew {
            found: version,
            supported: current,
        });
    }
    for from in version..current {
        let migration = usize::try_from(from - 1)
            .ok()
            .and_then(|index| migrations.get(index))
            .ok_or_else(|| MigrationError::Failed {
                from,
                reason: format!("no migration from version {from} is registered"),
            })?;
        document = migration(document).map_err(|reason| MigrationError::Failed { from, reason })?;
        if let Some(fields) = document.as_object_mut() {
            fields.insert(VERSION_FIELD.to_owned(), Value::from(from + 1));
        }
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[allow(
        clippy::unnecessary_wraps,
        reason = "migrations share the fallible `DocumentMigration` signature"
    )]
    fn add_title(mut document: Value) -> Result<Value, String> {
        document["title"] = json!("added by v2");
        Ok(document)
    }

    fn refuse(_: Value) -> Result<Value, String> {
        Err("cannot convert".to_owned())
    }

    #[test]
    fn there_is_one_migration_per_version_after_the_first() {
        let expected = se_domain::CURRENT_SCHEMA_VERSION - 1;
        assert_eq!(MIGRATIONS.len(), usize::try_from(expected).unwrap());
    }

    #[test]
    fn current_documents_are_left_unchanged() {
        let document = json!({ "schemaVersion": 1, "name": "Film" });
        assert_eq!(
            migrate_document(document.clone(), MIGRATIONS, 1),
            Ok(document)
        );
    }

    #[test]
    fn older_documents_go_through_every_step() {
        let migrated = migrate_document(json!({ "schemaVersion": 1 }), &[add_title], 2).unwrap();
        assert_eq!(
            migrated,
            json!({ "schemaVersion": 2, "title": "added by v2" })
        );
    }

    #[test]
    fn newer_or_unversioned_documents_are_refused() {
        assert_eq!(
            migrate_document(json!({ "schemaVersion": 9 }), MIGRATIONS, 1),
            Err(MigrationError::TooNew {
                found: 9,
                supported: 1
            })
        );
        assert_eq!(
            migrate_document(json!({ "name": "?" }), MIGRATIONS, 1),
            Err(MigrationError::MissingVersion)
        );
    }

    #[test]
    fn a_failing_migration_names_its_version() {
        let error = migrate_document(json!({ "schemaVersion": 1 }), &[refuse], 2).unwrap_err();
        assert_eq!(
            error,
            MigrationError::Failed {
                from: 1,
                reason: "cannot convert".to_owned()
            }
        );
    }
}
