//! `insert_library_template`: copies a motion template of the user's library into the project
//! (reusing a copy of the same version) and places an instance on the task's sequence
//! (prompt 25). Not batchable: the library is read first.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, LibraryInstance};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{
    context::{ToolContext, find_sequence},
    library_io::library_template,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    template_io::{param_values, template_range},
    time_io::TimeInput,
    write_inputs::{AnimationInput, presets},
    writes::{EditOutput, WriteScene, apply},
};

/// The tool.
pub struct InsertLibraryTemplate;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The library template (a `library_template_id` from `list_templates` with source
    /// "library").
    pub template_id: String,
    /// When it starts, in sequence time.
    pub start: TimeInput,
    /// When it ends (excluded). Give `end` or `duration`, or neither for the template's default
    /// duration.
    pub end: Option<TimeInput>,
    /// How long it lasts.
    pub duration: Option<TimeInput>,
    /// Parameter values by name (the others take the brand kit's value for parameters bound to
    /// a role, else their default). Numbers and colours also take
    /// `{"keyframes": [{"time": 0, "value": …}]}`; assets take an image `asset_id`.
    pub params: Option<Map<String, Value>>,
    /// A video track (id or name); default: the highest free track of the task.
    pub track: Option<String>,
    /// Entrance animation of the whole frame (preset `in` phase); usually the template animates
    /// itself.
    pub animation_in: Option<AnimationInput>,
    /// Exit animation of the whole frame (preset `out` phase).
    pub animation_out: Option<AnimationInput>,
}

impl McpTool for InsertLibraryTemplate {
    type Input = Input;

    const NAME: &'static str = "insert_library_template";

    const DESCRIPTION: &'static str = "Places a motion template of the user's template library \
        on the task's sequence over a time range (default length: the template's): the template \
        is copied into the project (a copy of the same version is reused) and an instance gets \
        the values given, the brand kit's values for parameters bound to a role, else the \
        defaults. Answers the new item_id; check it with get_frames. Not batchable.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let (entry, template) = library_template(&context, &input.template_id).await?;
        let project = context.project().await?;
        let names = std::collections::HashMap::new();
        let sequence = find_sequence(&project, &context.agent, None)?;
        let scene = WriteScene {
            project: &project,
            sequence,
            agent: &context.agent,
            names: &names,
        };
        let range = template_range(
            &scene,
            &input.start,
            input.end.as_ref(),
            input.duration.as_ref(),
            template.default_duration,
        )?;
        let params: BTreeMap<_, _> =
            param_values(&scene, &template, &input.params.unwrap_or_default())?
                .into_iter()
                .filter_map(|(name, value)| value.map(|value| (name, value)))
                .collect();
        let instance = LibraryInstance {
            entry,
            range,
            track: input
                .track
                .as_deref()
                .map(|track| scene.video_track(track))
                .transpose()?,
            params,
            presets: presets(
                &scene,
                input.animation_in.as_ref(),
                input.animation_out.as_ref(),
            )?,
        };
        let edits = [AgentEdit::InsertLibraryTemplate(Box::new(instance))];
        let report = apply(&context, Self::NAME, &edits, |_| None).await?;
        let created: Vec<_> = report.created.iter().flatten().copied().collect();
        ToolOutput::json(&EditOutput::new(&report, &created))
    }
}
