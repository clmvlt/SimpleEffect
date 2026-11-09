//! What the library tools share (prompt 25): library motion templates as agents read them, and
//! the lookup of an entry.

use se_app::template_library::project_copy;
use se_domain::{LibraryTemplateId, MotionTemplate, Origin, Project, TemplateId};
use se_protocol::{LibraryContent, LibraryTemplate};
use serde::Serialize;

use crate::{
    context::ToolContext,
    output::{ToolError, codes},
    template_io::{ParamSummary, param_summary},
    time_io::TimeOutput,
};

/// A library motion template as agents read it.
#[derive(Debug, Serialize)]
pub struct LibrarySummary {
    /// Its library id (for `insert_library_template`).
    pub library_template_id: String,
    /// Its name.
    pub name: String,
    /// What it shows.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Its browser category.
    pub category: &'static str,
    /// Free words the user filed it under.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Version of its content in the library.
    pub version: u32,
    /// Length of new instances.
    pub default_duration: TimeOutput,
    /// Its parameters, in declaration order (`brand` names the kit role filling them).
    pub params: Vec<ParamSummary>,
    /// The project's copy of this version, when the project has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_template_id: Option<String>,
}

/// The library entry `reference` (a `library_template_id`), as a motion template.
///
/// # Errors
/// [`codes::INVALID_ARGUMENTS`] for a malformed id, the library's not-found error, or a caption
/// style entry.
pub async fn library_template(
    context: &ToolContext,
    reference: &str,
) -> Result<(LibraryTemplate, MotionTemplate), ToolError> {
    let id = LibraryTemplateId::parse(reference).map_err(|_| {
        ToolError::invalid_arguments(format!(
            "'{reference}' is not a library template id: list them with list_templates and \
             source \"library\""
        ))
    })?;
    let entry = context.deps.library_templates.entry(id).await?;
    let template = as_template(&entry)?;
    Ok((entry, template))
}

/// The motion template `entry` describes (a placeholder project id), to read values against it.
///
/// # Errors
/// [`codes::INVALID_ARGUMENTS`] for a caption style entry.
pub fn as_template(entry: &LibraryTemplate) -> Result<MotionTemplate, ToolError> {
    let placeholder = TemplateId::from_uuid(entry.id.as_uuid())
        .map_err(|error| ToolError::new(codes::INTERNAL, error.to_string()))?;
    project_copy(entry, placeholder, 1, Origin::User).map_err(ToolError::from)
}

/// The summary of `entry` (a motion template) for `project`.
#[must_use]
pub fn library_summary(project: &Project, entry: &LibraryTemplate) -> Option<LibrarySummary> {
    let LibraryContent::MotionTemplate {
        category,
        params,
        default_duration,
        ..
    } = &entry.content
    else {
        return None;
    };
    Some(LibrarySummary {
        library_template_id: entry.id.to_string(),
        name: entry.name.clone(),
        description: entry.description.clone(),
        category: category.name(),
        tags: entry.tags.clone(),
        version: entry.version,
        default_duration: TimeOutput::from(*default_duration),
        params: params.iter().map(param_summary).collect(),
        project_template_id: se_app::template_library::existing_copy(project, entry)
            .map(|id| id.to_string()),
    })
}
