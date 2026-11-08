//! `get_item`: everything about one item — its summary, then its whole document form (style,
//! transform, keyframes, fades, reframing), with times made readable.

use schemars::JsonSchema;
use se_domain::{Flicks, Item};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    context::{ToolContext, find_item},
    describe::{ItemSummary, item_summary},
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::TimeOutput,
};

/// Media clip fields holding a duration or source time in flicks.
const CLIP_TIME_FIELDS: [&str; 3] = ["sourceStart", "fadeIn", "fadeOut"];

/// The tool.
pub struct GetItem;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item's id (from `get_timeline`).
    pub item_id: String,
}

#[derive(Debug, Serialize)]
struct ItemDetail {
    sequence_id: String,
    track_name: String,
    item: ItemSummary,
    /// The item's document form (camelCase fields, as stored). Keyframe `time`s are relative to
    /// the item's start; `sequence_time` gives the same instant in the sequence.
    properties: Value,
}

impl McpTool for GetItem {
    type Input = Input;

    const NAME: &'static str = "get_item";

    const DESCRIPTION: &'static str = "Everything about one item: its summary (as in \
        get_timeline) and its full properties — text and style, shape, transform (position, \
        scale, rotation, anchor, opacity, blur), keyframes with their easing, clip source, speed, \
        volume, fades, reframing, origin.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let (sequence, track, item) = find_item(&project, &input.item_id)?;
        ToolOutput::json(&ItemDetail {
            sequence_id: sequence.id.to_string(),
            track_name: track.name.clone(),
            item: item_summary(&project, track, item),
            properties: properties(item)?,
        })
    }
}

/// The item's kind as JSON, with every flicks time replaced by a [`TimeOutput`].
fn properties(item: &Item) -> Result<Value, ToolError> {
    let mut value = serde_json::to_value(&item.kind)
        .map_err(|error| ToolError::new(codes::INTERNAL, error.to_string()))?;
    if let Value::Object(fields) = &mut value {
        for field in CLIP_TIME_FIELDS {
            if let Some(slot) = fields.get_mut(field) {
                readable_time(slot, None);
            }
        }
    }
    readable_keyframes(&mut value, item.timeline_range.start());
    Ok(value)
}

/// Replaces keyframe times, wherever they are, by readable times.
fn readable_keyframes(value: &mut Value, item_start: Flicks) {
    match value {
        Value::Object(fields) => {
            if let Some(Value::Array(keyframes)) = fields.get_mut("keyframes") {
                for keyframe in keyframes {
                    if let Some(time) = keyframe.get_mut("time") {
                        readable_time(time, Some(item_start));
                    }
                }
            }
            for (key, child) in fields.iter_mut() {
                if key != "keyframes" {
                    readable_keyframes(child, item_start);
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                readable_keyframes(child, item_start);
            }
        }
        _ => {}
    }
}

/// Replaces a flicks number by a [`TimeOutput`], plus its sequence time for keyframes.
fn readable_time(slot: &mut Value, item_start: Option<Flicks>) {
    let Some(time) = slot.as_i64().and_then(|value| Flicks::new(value).ok()) else {
        return;
    };
    let mut readable = serde_json::to_value(TimeOutput::from(time)).unwrap_or(Value::Null);
    if let (Some(start), Value::Object(fields)) = (item_start, &mut readable)
        && let Some(absolute) = start.checked_add(time)
    {
        fields.insert(
            "sequence_time".to_owned(),
            serde_json::to_value(TimeOutput::from(absolute)).unwrap_or(Value::Null),
        );
    }
    *slot = readable;
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn clip_times_and_keyframes_become_readable() {
        let project = example_project();
        let item = &project.sequences[0].tracks[0].items[1];
        let value = properties(item).unwrap();
        assert_eq!(value["kind"], "mediaClip");
        assert_eq!(value["sourceStart"]["timecode"], "00:00:20.000");
        assert_eq!(value["fadeIn"]["seconds"], 0.5);
        let keyframes = &value["reframe"]["focus"]["keyframes"];
        assert_eq!(keyframes[1]["time"]["timecode"], "00:00:00.400");
        assert_eq!(
            keyframes[1]["time"]["sequence_time"]["timecode"],
            "00:00:10.400"
        );
        assert_eq!(keyframes[1]["value"]["x"], 0.7);
    }
}
