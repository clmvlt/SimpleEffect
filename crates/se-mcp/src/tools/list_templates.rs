//! `list_templates`: the motion templates of the project (prompt 24) or of the app's template
//! library (prompt 25), filtered by category; or one of them with its source.

use schemars::JsonSchema;
use se_domain::TemplateCategory;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    library_io::{LibrarySummary, library_summary, library_template},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    template_io::{TemplateSummary, find_template, template_summary},
};

/// The tool.
pub struct ListTemplates;

/// Where templates are listed from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The project's templates (default).
    #[default]
    Project,
    /// The user's template library, shared by every project.
    Library,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// `project` (default) or `library` (the user's saved templates: insert one with
    /// `insert_library_template`).
    pub source: Option<Source>,
    /// Only this category: `title`, `lowerThird`, `callToAction`, `transition` or `other`.
    pub category: Option<String>,
    /// One template (a project `template_id`, or a `library_template_id` with source
    /// `library`): answers its full source too.
    pub template_id: Option<String>,
}

/// Every template of the project.
#[derive(Debug, Serialize)]
struct Listing {
    templates: Vec<TemplateSummary>,
}

/// Every motion template of the library.
#[derive(Debug, Serialize)]
struct LibraryListing {
    templates: Vec<LibrarySummary>,
}

/// One template with its source.
#[derive(Debug, Serialize)]
struct Detail<T> {
    template: T,
    source: String,
}

impl McpTool for ListTemplates {
    type Input = Input;

    const NAME: &'static str = "list_templates";

    const DESCRIPTION: &'static str = "Motion templates of the project (default) or of the \
        user's template library (source \"library\"), optionally of one category: id, name, \
        category, version, default duration, parameters with types, defaults, ranges and brand \
        kit roles, number of instances (project) or the project's copy (library). With \
        template_id: that template and its full TypeScript source (project templates: to write a \
        new version with update_motion_template). Prefer a fitting library template \
        (insert_library_template) over writing a new one.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let category = input.category.as_deref().map(category_of).transpose()?;
        let project = context.project().await?;
        match (input.source.unwrap_or_default(), input.template_id) {
            (Source::Project, Some(reference)) => {
                let template = find_template(&project, &reference)?;
                ToolOutput::json(&Detail {
                    template: template_summary(&project, template),
                    source: template.source.clone(),
                })
            }
            (Source::Project, None) => ToolOutput::json(&Listing {
                templates: project
                    .templates
                    .iter()
                    .filter(|template| category.is_none_or(|wanted| template.category == wanted))
                    .map(|template| template_summary(&project, template))
                    .collect(),
            }),
            (Source::Library, Some(reference)) => {
                let (entry, template) = library_template(&context, &reference).await?;
                let summary = library_summary(&project, &entry).ok_or_else(|| {
                    ToolError::invalid_arguments("that library entry is not a motion template")
                })?;
                ToolOutput::json(&Detail {
                    template: summary,
                    source: template.source,
                })
            }
            (Source::Library, None) => {
                let entries = context.deps.library_templates.list().await?;
                let templates = entries
                    .iter()
                    .filter_map(|entry| library_summary(&project, entry))
                    .filter(|summary| {
                        category.is_none_or(|wanted| summary.category == wanted.name())
                    })
                    .collect();
                ToolOutput::json(&LibraryListing { templates })
            }
        }
    }
}

fn category_of(name: &str) -> Result<TemplateCategory, ToolError> {
    TemplateCategory::from_name(name).ok_or_else(|| {
        ToolError::invalid_arguments(format!(
            "category '{name}' does not exist; use one of: {}",
            TemplateCategory::names()
        ))
    })
}
