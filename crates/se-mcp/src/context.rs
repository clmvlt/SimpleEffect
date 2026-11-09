//! What a tool call works with: the services it reads through ([`McpDeps`]) and who is calling
//! ([`ToolContext::agent`]), plus lookups shared by the tools.

use std::sync::Arc;

use se_app::{
    AgentContext, AgentEditService, CaptionService, FrameRenderService, MediaService,
    ProjectService, SmartCutService, TaskTokenRegistry, TemplateCheckService, TranscriptionService,
};
use se_domain::{Asset, Flicks, Item, Project, Sequence, TimeRange, Track};

use crate::output::{ToolError, codes};

/// The se-app services tools call.
#[derive(Clone)]
pub struct McpDeps {
    /// Open projects (the current document).
    pub projects: Arc<ProjectService>,
    /// Media: stored analyses, extracted source frames.
    pub media: Arc<MediaService>,
    /// Composited frames drawn by the renderer.
    pub renders: Arc<FrameRenderService>,
    /// Motion template modules checked by the renderer (prompt 24).
    pub template_checks: Arc<TemplateCheckService>,
    /// Tokens of running agents: who is calling.
    pub tokens: Arc<TaskTokenRegistry>,
    /// Edits of agents: the only way write tools change a project.
    pub edits: Arc<AgentEditService>,
    /// Transcripts of assets and of sequences.
    pub transcription: TranscriptionService,
    /// Smart cuts: their settings and the media facts they read (prompt 22).
    pub smart_cuts: SmartCutService,
    /// Captions: the words they are made of (prompt 23).
    pub captions: CaptionService,
}

/// One tool call: the services and the calling run.
#[derive(Clone)]
pub struct ToolContext {
    /// The services.
    pub deps: McpDeps,
    /// The calling run: its project, task and scope.
    pub agent: AgentContext,
}

impl ToolContext {
    /// The current document of the task's project.
    ///
    /// # Errors
    /// `PROJECT_NOT_OPEN` once the user closed the project.
    pub async fn project(&self) -> Result<Arc<Project>, ToolError> {
        Ok(self.deps.projects.project(self.agent.project_id).await?)
    }
}

/// The sequence called `reference` (an id or an exact name), or the task's sequence when `None`.
///
/// # Errors
/// [`codes::SEQUENCE_NOT_FOUND`] listing the sequences.
pub fn find_sequence<'a>(
    project: &'a Project,
    agent: &AgentContext,
    reference: Option<&str>,
) -> Result<&'a Sequence, ToolError> {
    let found = match reference {
        None => project
            .sequences
            .iter()
            .find(|sequence| sequence.id == agent.sequence_id()),
        Some(reference) => project
            .sequences
            .iter()
            .find(|sequence| sequence.id.to_string() == reference || sequence.name == reference),
    };
    found.ok_or_else(|| {
        let known: Vec<String> = project
            .sequences
            .iter()
            .map(|sequence| format!("« {} » ({})", sequence.name, sequence.id))
            .collect();
        ToolError::new(
            codes::SEQUENCE_NOT_FOUND,
            format!(
                "no sequence {}; the project has {}",
                reference.unwrap_or("of the task"),
                known.join(", ")
            ),
        )
    })
}

/// The range read when a tool gets none: the task's range on the task's sequence, else the
/// whole of `sequence`.
///
/// # Errors
/// [`codes::INTERNAL`] for an unrepresentable sequence length (never in a valid project).
pub fn default_range(agent: &AgentContext, sequence: &Sequence) -> Result<TimeRange, ToolError> {
    match agent.scope.range {
        Some(range) if agent.sequence_id() == sequence.id => Ok(range),
        _ => whole_sequence(sequence),
    }
}

/// From 0 to the end of `sequence`'s content (one frame when it is empty, so the range is never
/// empty).
///
/// # Errors
/// [`codes::INTERNAL`] for an unrepresentable sequence length (never in a valid project).
pub fn whole_sequence(sequence: &Sequence) -> Result<TimeRange, ToolError> {
    let end = sequence
        .content_end()
        .max(sequence.format.frame_rate.frame_duration());
    TimeRange::from_start_end(Flicks::ZERO, end)
        .map_err(|error| ToolError::new(codes::INTERNAL, error.to_string()))
}

/// The tracks of `sequence` called by each of `references` (ids or names), in the sequence's
/// order; every track when `None`.
///
/// # Errors
/// [`codes::TRACK_NOT_FOUND`] listing the sequence's tracks.
pub fn find_tracks<'a>(
    sequence: &'a Sequence,
    references: Option<&[String]>,
) -> Result<Vec<&'a Track>, ToolError> {
    let Some(references) = references else {
        return Ok(sequence.tracks.iter().collect());
    };
    for reference in references {
        let known = sequence
            .tracks
            .iter()
            .any(|track| track.id.to_string() == *reference || track.name == *reference);
        if !known {
            let names: Vec<String> = sequence
                .tracks
                .iter()
                .map(|track| format!("{} ({})", track.name, track.id))
                .collect();
            return Err(ToolError::new(
                codes::TRACK_NOT_FOUND,
                format!(
                    "no track {reference} in « {} »; its tracks are {}",
                    sequence.name,
                    names.join(", ")
                ),
            ));
        }
    }
    Ok(sequence
        .tracks
        .iter()
        .filter(|track| {
            references
                .iter()
                .any(|reference| track.id.to_string() == *reference || track.name == *reference)
        })
        .collect())
}

/// An item of any sequence, with its sequence and track.
///
/// # Errors
/// [`codes::ITEM_NOT_FOUND`].
pub fn find_item<'a>(
    project: &'a Project,
    item_id: &str,
) -> Result<(&'a Sequence, &'a Track, &'a Item), ToolError> {
    for sequence in &project.sequences {
        for track in &sequence.tracks {
            if let Some(item) = track
                .items
                .iter()
                .find(|item| item.id.to_string() == item_id)
            {
                return Ok((sequence, track, item));
            }
        }
    }
    Err(ToolError::new(
        codes::ITEM_NOT_FOUND,
        format!("no item {item_id}: list items with get_timeline and copy an item_id from there"),
    ))
}

/// An asset of the project.
///
/// # Errors
/// [`codes::ASSET_NOT_FOUND`].
pub fn find_asset<'a>(project: &'a Project, asset_id: &str) -> Result<&'a Asset, ToolError> {
    project
        .assets
        .iter()
        .find(|asset| asset.id.to_string() == asset_id)
        .ok_or_else(|| {
            ToolError::new(
                codes::ASSET_NOT_FOUND,
                format!("no asset {asset_id}: list assets with list_assets and copy an asset_id"),
            )
        })
}

#[cfg(test)]
mod tests {
    use se_domain::{ProjectId, TaskId, test_support::example_project};
    use se_protocol::TaskScope;

    use super::*;

    fn agent(project: &Project) -> AgentContext {
        AgentContext {
            project_id: ProjectId::from_test_index(1),
            task_id: TaskId::from_test_index(1),
            run: 0,
            scope: TaskScope {
                sequence_id: project.sequences[0].id,
                range: None,
                tracks: None,
            },
        }
    }

    #[test]
    fn sequences_are_found_by_id_name_or_scope() {
        let project = example_project();
        let agent = agent(&project);
        let main = &project.sequences[0];
        assert_eq!(find_sequence(&project, &agent, None).unwrap().id, main.id);
        assert_eq!(
            find_sequence(&project, &agent, Some("Main edit"))
                .unwrap()
                .id,
            main.id
        );
        let error = find_sequence(&project, &agent, Some("Other")).unwrap_err();
        assert_eq!(error.code, codes::SEQUENCE_NOT_FOUND);
        assert!(error.message.contains("Main edit"));
    }

    #[test]
    fn tracks_keep_the_sequence_order() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let picked = find_tracks(sequence, Some(&["V2".to_owned(), "V1".to_owned()])).unwrap();
        let names: Vec<&str> = picked.iter().map(|track| track.name.as_str()).collect();
        assert_eq!(names, ["V1", "V2"]);
        assert_eq!(
            find_tracks(sequence, Some(&["V9".to_owned()]))
                .unwrap_err()
                .code,
            codes::TRACK_NOT_FOUND
        );
    }

    #[test]
    fn items_and_assets_are_found_anywhere() {
        let project = example_project();
        let item = &project.sequences[0].tracks[0].items[0];
        let (_, track, found) = find_item(&project, &item.id.to_string()).unwrap();
        assert_eq!(found.id, item.id);
        assert_eq!(track.name, "V1");
        assert_eq!(
            find_item(&project, "x").unwrap_err().code,
            codes::ITEM_NOT_FOUND
        );
        let asset = &project.assets[1];
        assert_eq!(
            find_asset(&project, &asset.id.to_string()).unwrap().id,
            asset.id
        );
    }
}
