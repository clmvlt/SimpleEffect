//! `set_keyframes`: the keyframes (or a static value) of one animatable property of an item.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::{AnimatableProperty, PropertyAnimation};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    write_inputs::{EasingInput, ValueInput},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct SetKeyframes;

/// One keyframe.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KeyframeInput {
    /// Time **relative to the item's start** (0 = its first instant), within the item.
    pub time: TimeInput,
    /// The value: a number, `[x, y]` (position, scale, anchor, reframeFocus, shadowOffset) or
    /// `#rrggbb[aa]` (colours).
    pub value: ValueInput,
    /// How the value travels to the next keyframe; default linear.
    pub easing: Option<EasingInput>,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// The property: position, scale, rotation, anchor, opacity, blur, volumeDb, reframeFocus,
    /// fontSize, fillColor, letterSpacing, reveal, strokeColor, strokeWidth, shadowColor,
    /// shadowBlur, shadowOffset, backgroundColor.
    pub property: String,
    /// The keyframes, in time order (at least one). Give `keyframes` or `value`.
    pub keyframes: Option<Vec<KeyframeInput>>,
    /// A static value replacing any keyframes. Give `keyframes` or `value`.
    pub value: Option<ValueInput>,
    /// The revision you read; refused (`REVISION_CONFLICT`) if the item changed since.
    pub expected_revision: Option<u64>,
}

impl McpTool for SetKeyframes {
    type Input = Input;

    const NAME: &'static str = "set_keyframes";

    const DESCRIPTION: &'static str = "Replaces the keyframes (or sets a static value) of one \
        animatable property of an item: position, scale, rotation, anchor, opacity, blur, \
        volumeDb, reframeFocus, text properties, colours. Keyframe times are relative to the \
        item's start. Prefer apply_animation_preset for usual entrances and exits.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for SetKeyframes {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let property = property(&input.property)?;
        let value = match (&input.keyframes, &input.value) {
            (Some(keyframes), None) if !keyframes.is_empty() => {
                let keyframes = keyframes
                    .iter()
                    .map(|keyframe| {
                        let easing = keyframe
                            .easing
                            .map_or(se_domain::Easing::Linear, EasingInput::to_easing);
                        Ok(json!({
                            "time": scene.time(&keyframe.time)?.get(),
                            "value": keyframe.value.to_json(),
                            "easing": easing_json(easing)?,
                        }))
                    })
                    .collect::<Result<Vec<Value>, ToolError>>()?;
                json!({ "kind": "keyframed", "keyframes": keyframes })
            }
            (None, Some(value)) => json!({ "kind": "static", "value": value.to_json() }),
            _ => {
                return Err(ToolError::invalid_arguments(
                    "give either `keyframes` (at least one) or `value`",
                ));
            }
        };
        Ok(Planned::one(AgentEdit::SetProperty {
            item: scene.item(&input.item_id)?,
            animation: animation(property, &value)?,
            expected_revision: input.expected_revision,
        }))
    }
}

/// The property called `name`.
///
/// # Errors
/// [`crate::output::codes::INVALID_ARGUMENTS`] listing the properties.
pub fn property(name: &str) -> Result<AnimatableProperty, ToolError> {
    AnimatableProperty::ALL
        .into_iter()
        .find(|property| property.name() == name)
        .ok_or_else(|| {
            let names: Vec<&str> = AnimatableProperty::ALL
                .iter()
                .map(|property| property.name())
                .collect();
            ToolError::invalid_arguments(format!(
                "no property « {name} »; the properties are {}",
                names.join(", ")
            ))
        })
}

/// `value` (the document's `Animatable` JSON) typed for `property`.
///
/// # Errors
/// [`crate::output::codes::INVALID_ARGUMENTS`] saying what values the property takes.
pub fn animation(
    property: AnimatableProperty,
    value: &Value,
) -> Result<PropertyAnimation, ToolError> {
    serde_json::from_value(json!({ "property": property.name(), "value": value })).map_err(
        |error| {
            ToolError::invalid_arguments(format!(
                "values of {} are {}: {error}",
                property.name(),
                value_kind(property)
            ))
        },
    )
}

fn value_kind(property: AnimatableProperty) -> &'static str {
    use AnimatableProperty as P;
    match property {
        P::Position | P::Scale | P::Anchor | P::ReframeFocus | P::ShadowOffset => "pairs [x, y]",
        P::FillColor | P::StrokeColor | P::ShadowColor | P::BackgroundColor => {
            "colours \"#rrggbb\" or \"#rrggbbaa\""
        }
        _ => "numbers",
    }
}

fn easing_json(easing: se_domain::Easing) -> Result<Value, ToolError> {
    serde_json::to_value(easing).map_err(|error| ToolError::invalid_arguments(error.to_string()))
}
