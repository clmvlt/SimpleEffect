//! `add_transition`: a cross-fade (picture) or cross-fade (sound) on the cut between two adjacent
//! items of a track, given by the two items or by the track and the cut's time.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, ItemRef};
use se_domain::{Flicks, TransitionAlignment, format_clock};
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::TimeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct AddTransition;

/// Default length of a transition, in milliseconds.
const DEFAULT_DURATION_MS: u32 = 500;

/// Where the transition sits on the cut.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentInput {
    /// Centred on the cut (default).
    Center,
    /// Starts at the cut (inside the right item).
    Start,
    /// Ends at the cut (inside the left item).
    End,
}

/// Arguments: the two items, or the track and the cut's time.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item ending at the cut.
    pub left_item_id: Option<String>,
    /// The item starting at the cut.
    pub right_item_id: Option<String>,
    /// The track of the cut (id or name), with `at`.
    pub track: Option<String>,
    /// The time of the cut (within half a frame), with `track`.
    pub at: Option<TimeInput>,
    /// The wanted length (shortened to the media both items have beyond the cut); default 0.5 s.
    pub duration: Option<TimeInput>,
    /// Where it sits on the cut; default center.
    pub alignment: Option<AlignmentInput>,
}

impl McpTool for AddTransition {
    type Input = Input;

    const NAME: &'static str = "add_transition";

    const DESCRIPTION: &'static str = "Adds a cross-fade on the cut between two adjacent items of \
        one track (picture dissolve on video tracks, sound cross-fade on audio tracks), given by \
        the two items or by the track and the cut's time. The length is shortened to the media \
        both items have beyond the cut.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for AddTransition {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let (left, right) = match (
            &input.left_item_id,
            &input.right_item_id,
            &input.track,
            &input.at,
        ) {
            (Some(left), Some(right), None, None) => (scene.item(left)?, scene.item(right)?),
            (None, None, Some(track), Some(at)) => cut_at(scene, track, at)?,
            _ => {
                return Err(ToolError::invalid_arguments(
                    "give left_item_id and right_item_id, or track and at",
                ));
            }
        };
        let duration = match &input.duration {
            Some(duration) => scene.duration(duration, "the transition")?,
            None => Flicks::from_millis(DEFAULT_DURATION_MS),
        };
        Ok(Planned::one(AgentEdit::AddTransition {
            left,
            right,
            duration,
            alignment: match input.alignment.unwrap_or(AlignmentInput::Center) {
                AlignmentInput::Center => TransitionAlignment::Center,
                AlignmentInput::Start => TransitionAlignment::Start,
                AlignmentInput::End => TransitionAlignment::End,
            },
        }))
    }
}

/// The two items meeting at the cut nearest to `at` (within half a frame) on `track`.
fn cut_at(
    scene: &WriteScene<'_>,
    track: &str,
    at: &TimeInput,
) -> Result<(ItemRef, ItemRef), ToolError> {
    let track_id = scene
        .tracks(std::slice::from_ref(&track.to_owned()))?
        .first()
        .copied()
        .ok_or_else(|| ToolError::new(codes::TRACK_NOT_FOUND, format!("no track {track}")))?;
    let at = scene.time(at)?;
    let half_frame = Flicks::new(scene.sequence.format.frame_rate.frame_duration().get() / 2)
        .unwrap_or(Flicks::ZERO);
    let items = scene
        .sequence
        .tracks
        .iter()
        .find(|candidate| candidate.id == track_id)
        .map(|found| found.items.as_slice())
        .unwrap_or_default();
    items
        .windows(2)
        .find_map(|pair| {
            let [left, right] = pair else { return None };
            let cut = left.timeline_range.end();
            let close = cut
                .checked_sub(at)
                .is_some_and(|gap| gap.abs() <= half_frame);
            (close && right.timeline_range.start() == cut)
                .then_some((ItemRef::Id(left.id), ItemRef::Id(right.id)))
        })
        .ok_or_else(|| {
            ToolError::invalid_arguments(format!(
                "track {track} has no cut at {} (two items touching there): list its items with \
                 get_timeline",
                format_clock(at)
            ))
        })
}
