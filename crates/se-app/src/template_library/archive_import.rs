//! Importing one `.setemplate` file (prompt 25): read, checked like an agent's module (the
//! renderer runs it, the domain reads its declarations), then stored unless the library already
//! holds that version.

use std::path::Path;

use se_domain::{Format, MAX_TEMPLATE_NAME_CHARS, ProjectId};
use se_protocol::{LibraryContent, LibraryImportOutcome, LibraryTemplate};

use crate::{AppError, TemplateCheckService, ports::TemplateLibrary};

/// Imports the file at `path` (see the module documentation); failures become
/// [`LibraryImportOutcome::Failed`] with their message.
pub(super) async fn import_one(
    library: &dyn TemplateLibrary,
    checks: &TemplateCheckService,
    project_id: ProjectId,
    format: &Format,
    path: &Path,
) -> LibraryImportOutcome {
    match checked_entry(library, checks, project_id, format, path).await {
        Ok(entry) => store(library, entry)
            .await
            .unwrap_or_else(|error| failed(&error)),
        Err(error) => failed(&error),
    }
}

fn failed(error: &AppError) -> LibraryImportOutcome {
    LibraryImportOutcome::Failed {
        message: error.to_string(),
    }
}

/// The archive's entry once its content passed the checks; a module's declarations replace what
/// the archive says about its parameters (the module is the truth, ADR-0035).
async fn checked_entry(
    library: &dyn TemplateLibrary,
    checks: &TemplateCheckService,
    project_id: ProjectId,
    format: &Format,
    path: &Path,
) -> Result<LibraryTemplate, AppError> {
    let mut entry = library.read_archive(path).await?;
    let name_chars = entry.name.trim().chars().count();
    if entry.version == 0 || !(1..=MAX_TEMPLATE_NAME_CHARS).contains(&name_chars) {
        return Err(invalid(path, "its name or version is unusable"));
    }
    entry.content = match entry.content {
        LibraryContent::MotionTemplate { source, .. } => {
            let checked = checks.check(project_id, format, &source).await?;
            LibraryContent::MotionTemplate {
                category: checked.definition.category,
                params: checked.definition.params,
                default_duration: checked.definition.default_duration,
                source,
            }
        }
        LibraryContent::CaptionStyle { style } => {
            let problems = style.problems();
            if !problems.is_empty() {
                let fields: Vec<String> = problems
                    .into_iter()
                    .map(|(field, rule)| format!("{field} {rule}"))
                    .collect();
                return Err(invalid(
                    path,
                    &format!("its caption style is unusable ({})", fields.join("; ")),
                ));
            }
            LibraryContent::CaptionStyle { style }
        }
    };
    Ok(entry)
}

/// Stores `entry` when it is new or newer than the library's.
async fn store(
    library: &dyn TemplateLibrary,
    entry: LibraryTemplate,
) -> Result<LibraryImportOutcome, AppError> {
    match library.get(entry.id).await? {
        Some(existing) if existing.version >= entry.version => {
            Ok(LibraryImportOutcome::Unchanged { template: existing })
        }
        Some(_) => {
            library.save(&entry).await?;
            Ok(LibraryImportOutcome::Updated { template: entry })
        }
        None => {
            library.save(&entry).await?;
            Ok(LibraryImportOutcome::Imported { template: entry })
        }
    }
}

fn invalid(path: &Path, reason: &str) -> AppError {
    AppError::InvalidLibraryFile {
        path: path.display().to_string(),
        reason: format!("{reason}; export it again from SimpleEffect"),
    }
}
