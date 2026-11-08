//! What write tools share (prompt 19, ADR-0029): a tool turns its input into se-app
//! [`AgentEdit`]s ([`WriteTool::plan`], against a [`WriteScene`]: the current document, the task's
//! sequence and the batch's names), then [`run`] hands them to `AgentEditService` — which plans,
//! checks the scope and commits them as one AI transaction — and answers with what changed.
//! `apply_batch` plans several tools' inputs into one call.

use std::{collections::HashMap, sync::Arc};

use se_app::{
    AgentContext, AppError, EditFailure,
    agent_edits::{AgentEdit, AgentEditReport, ItemRef},
};
use se_domain::{
    DomainError, DomainErrorCode, Flicks, ItemId, Project, Sequence, TrackId, TrackKind,
};
use serde::Serialize;

use crate::{
    context::{ToolContext, find_sequence, find_tracks},
    describe::{ItemSummary, item_summary},
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::TimeInput,
};

/// The document a write tool plans against.
pub struct WriteScene<'a> {
    /// The current document.
    pub project: &'a Project,
    /// The task's sequence (write tools only change it).
    pub sequence: &'a Sequence,
    /// The calling run.
    pub agent: &'a AgentContext,
    /// `apply_batch` names given so far (`"as"`) and the items they stand for.
    pub names: &'a HashMap<String, NamedItem>,
}

/// An item created earlier in a batch, named by its action's `"as"`.
#[derive(Debug, Clone, Copy)]
pub struct NamedItem {
    /// Index of the edit (in the whole batch) that creates it.
    pub edit: usize,
    /// Its kind (`text`, `shape`…, as `ItemKind::label`), when known.
    pub kind: Option<&'static str>,
}

/// A tool's input as se-app edits.
#[derive(Debug, Default)]
pub struct Planned {
    /// The edits, in order.
    pub edits: Vec<AgentEdit>,
    /// Which of them creates the item the tool is about (`add_text`…), to name it in a batch.
    pub creates: Option<usize>,
}

impl Planned {
    /// One edit that creates nothing.
    #[must_use]
    pub fn one(edit: AgentEdit) -> Self {
        Self {
            edits: vec![edit],
            creates: None,
        }
    }
}

/// A tool that changes the project.
pub trait WriteTool: McpTool {
    /// The tool's input as edits.
    ///
    /// # Errors
    /// [`ToolError`] for arguments that cannot be read (times, names, values).
    fn plan(scene: &WriteScene<'_>, input: Self::Input) -> Result<Planned, ToolError>;
}

/// Runs write tool `T`: plans it on the current document, applies it, reports.
///
/// # Errors
/// The planning error, or the refusal of the edit (scope, revision, document rules).
pub async fn run<T: WriteTool>(
    context: ToolContext,
    input: T::Input,
) -> Result<ToolOutput, ToolError> {
    let project = context.project().await?;
    let names = HashMap::new();
    let sequence = find_sequence(&project, &context.agent, None)?;
    let scene = WriteScene {
        project: &project,
        sequence,
        agent: &context.agent,
        names: &names,
    };
    let planned = T::plan(&scene, input)?;
    let report = apply(&context, T::NAME, &planned.edits, |_| None).await?;
    ToolOutput::json(&EditOutput::new(&report, &flatten(&report.created)))
}

/// Applies `edits` as one AI transaction labelled `label`; `action_of` names the batch action
/// an edit index came from (for error messages).
///
/// # Errors
/// The refusal, as a [`ToolError`] the agent can act on.
pub async fn apply(
    context: &ToolContext,
    label: &str,
    edits: &[AgentEdit],
    action_of: impl Fn(usize) -> Option<String>,
) -> Result<AgentEditReport, ToolError> {
    context
        .deps
        .edits
        .apply(&context.agent, label, edits)
        .await
        .map_err(|failure| edit_error(failure, action_of))
}

/// The tool error of a refused call.
fn edit_error(failure: EditFailure, action_of: impl Fn(usize) -> Option<String>) -> ToolError {
    let mut error = app_error(failure.error);
    if let Some(action) = failure.edit.and_then(action_of) {
        error.message = format!("{action}: {}", error.message);
    }
    error
}

/// A use-case error as the agent sees it: domain refusals get the write tools' codes
/// (`REVISION_CONFLICT`, `LOCKED_TRACK`, `*_NOT_FOUND`, `VALIDATION_ERROR`).
#[must_use]
pub fn app_error(error: AppError) -> ToolError {
    match error {
        AppError::Domain(domain) => domain_error(&domain),
        AppError::InvalidEdit { reason } => ToolError::new(codes::VALIDATION_ERROR, reason),
        other => ToolError::from(other),
    }
}

fn domain_error(error: &DomainError) -> ToolError {
    let code = error.code();
    let (tool_code, hint) = match code {
        DomainErrorCode::RevisionConflict => (
            codes::REVISION_CONFLICT,
            "; someone changed it after you read it: read it again (get_item) and redo your \
             change on what is there now",
        ),
        DomainErrorCode::TrackLocked => (
            codes::LOCKED_TRACK,
            "; the user locked that track: leave it alone or use another track",
        ),
        DomainErrorCode::ItemNotFound => (codes::ITEM_NOT_FOUND, ""),
        DomainErrorCode::TrackNotFound => (codes::TRACK_NOT_FOUND, ""),
        DomainErrorCode::SequenceNotFound => (codes::SEQUENCE_NOT_FOUND, ""),
        DomainErrorCode::AssetNotFound => (codes::ASSET_NOT_FOUND, ""),
        _ => (codes::VALIDATION_ERROR, ""),
    };
    let message = if tool_code == codes::VALIDATION_ERROR {
        format!("{error} (rule {code})")
    } else {
        format!("{error}{hint}")
    };
    ToolError::new(tool_code, message)
}

/// What a write tool answers.
#[derive(Debug, Serialize)]
pub struct EditOutput {
    /// The project's revision after the edit.
    pub revision: u64,
    /// Items created, in order.
    pub created: Vec<String>,
    /// Items added or changed, as they are now (with their new revisions).
    pub items: Vec<ItemSummary>,
    /// Items removed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<String>,
}

impl EditOutput {
    /// The output of `report`, listing `created`.
    #[must_use]
    pub fn new(report: &AgentEditReport, created: &[ItemId]) -> Self {
        Self {
            revision: report.revision,
            created: created.iter().map(ToString::to_string).collect(),
            items: summaries(&report.project, &report.changed),
            removed: report.removed.iter().map(ToString::to_string).collect(),
        }
    }
}

/// Summaries of the items `ids` that exist in `project`, in `ids` order.
#[must_use]
pub fn summaries(project: &Arc<Project>, ids: &[ItemId]) -> Vec<ItemSummary> {
    ids.iter()
        .filter_map(|id| {
            project
                .sequences
                .iter()
                .flat_map(|sequence| &sequence.tracks)
                .find_map(|track| {
                    let item = track.items.iter().find(|item| item.id == *id)?;
                    Some(item_summary(project, track, item))
                })
        })
        .collect()
}

/// Every created item, in order.
#[must_use]
pub fn flatten(created: &[Vec<ItemId>]) -> Vec<ItemId> {
    created.iter().flatten().copied().collect()
}

impl WriteScene<'_> {
    /// A time of the task's sequence.
    ///
    /// # Errors
    /// [`codes::INVALID_TIME`].
    pub fn time(&self, time: &TimeInput) -> Result<Flicks, ToolError> {
        time.to_flicks(self.sequence.format.frame_rate)
    }

    /// A duration (a time measured from 0), refused when not positive.
    ///
    /// # Errors
    /// [`codes::INVALID_TIME`].
    pub fn duration(&self, duration: &TimeInput, what: &str) -> Result<Flicks, ToolError> {
        let flicks = self.time(duration)?;
        if flicks <= Flicks::ZERO {
            return Err(ToolError::new(
                codes::INVALID_TIME,
                format!("{what} must be longer than 0"),
            ));
        }
        Ok(flicks)
    }

    /// The item called `reference`: an item id, or `$name` for an item created earlier in the
    /// batch (`"as": "name"`).
    ///
    /// # Errors
    /// [`codes::ITEM_NOT_FOUND`] for an unknown name or a malformed id.
    pub fn item(&self, reference: &str) -> Result<ItemRef, ToolError> {
        if let Some(name) = reference.strip_prefix('$') {
            return self
                .names
                .get(name)
                .map(|named| ItemRef::Created { edit: named.edit })
                .ok_or_else(|| {
                    ToolError::new(
                        codes::ITEM_NOT_FOUND,
                        format!(
                            "no earlier action of this batch is named « {name} »: give \"as\": \
                             \"{name}\" to the action that adds the item, before this one"
                        ),
                    )
                });
        }
        ItemId::parse(reference).map(ItemRef::Id).map_err(|_| {
            ToolError::new(
                codes::ITEM_NOT_FOUND,
                format!(
                    "{reference} is not an item id: copy an item_id from get_timeline (or write \
                     $name for an item added earlier in the batch)"
                ),
            )
        })
    }

    /// The kind (`ItemKind::label`) of the item called `reference`.
    ///
    /// # Errors
    /// [`codes::ITEM_NOT_FOUND`].
    pub fn item_kind(&self, reference: &str) -> Result<&'static str, ToolError> {
        let not_found = || {
            ToolError::new(
                codes::ITEM_NOT_FOUND,
                format!("no item {reference}: list items with get_timeline and copy an item_id"),
            )
        };
        if let Some(name) = reference.strip_prefix('$') {
            return self
                .names
                .get(name)
                .and_then(|named| named.kind)
                .ok_or_else(not_found);
        }
        self.project
            .items()
            .find(|item| item.id.to_string() == reference)
            .map(|item| item.kind.label())
            .ok_or_else(not_found)
    }

    /// The tracks of the task's sequence called `references` (ids or names), in sequence order.
    ///
    /// # Errors
    /// [`codes::TRACK_NOT_FOUND`].
    pub fn tracks(&self, references: &[String]) -> Result<Vec<TrackId>, ToolError> {
        Ok(find_tracks(self.sequence, Some(references))?
            .iter()
            .map(|track| track.id)
            .collect())
    }

    /// The video track of the task's sequence called `reference`.
    ///
    /// # Errors
    /// [`codes::TRACK_NOT_FOUND`]; [`codes::INVALID_ARGUMENTS`] for a sound track.
    pub fn video_track(&self, reference: &str) -> Result<TrackId, ToolError> {
        let tracks = find_tracks(self.sequence, Some(&[reference.to_owned()]))?;
        let track = tracks.first().ok_or_else(|| {
            ToolError::new(codes::TRACK_NOT_FOUND, format!("no track {reference}"))
        })?;
        if track.kind != TrackKind::Video {
            return Err(ToolError::invalid_arguments(format!(
                "track {} holds sound: texts, shapes and images go on video tracks",
                track.name
            )));
        }
        Ok(track.id)
    }
}
