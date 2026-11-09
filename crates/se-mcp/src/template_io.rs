//! What the motion template tools share (prompt 24, ADR-0035): finding a template, describing it
//! compactly, reading parameter values from JSON against its schema, and the pictures of a check.

use std::collections::BTreeMap;

use se_app::CheckedTemplate;
use se_domain::{
    Animatable, AssetId, Color, Easing, Flicks, Keyframe, MotionTemplate, Project, TemplateId,
    TemplateParam, TemplateParamSpec, TemplateParamValue,
};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    output::{ToolError, ToolOutput, codes},
    time_io::{TimeInput, TimeOutput},
    tools::set_keyframes::KeyframeInput,
    write_inputs::{color, span},
    writes::WriteScene,
};

/// The template called `reference` (a `template_id`).
///
/// # Errors
/// [`codes::TEMPLATE_NOT_FOUND`] listing the project's templates.
pub fn find_template<'a>(
    project: &'a Project,
    reference: &str,
) -> Result<&'a MotionTemplate, ToolError> {
    TemplateId::parse(reference)
        .ok()
        .and_then(|id| project.templates.iter().find(|template| template.id == id))
        .ok_or_else(|| {
            let known: Vec<String> = project
                .templates
                .iter()
                .map(|template| format!("{} « {} »", template.id, template.name))
                .collect();
            ToolError::new(
                codes::TEMPLATE_NOT_FOUND,
                if known.is_empty() {
                    format!(
                        "no motion template '{reference}': the project has none yet; write one \
                         with create_motion_template"
                    )
                } else {
                    format!(
                        "no motion template '{reference}'; the project's templates are: {}",
                        known.join(", ")
                    )
                },
            )
        })
}

/// A template as agents read it.
#[derive(Debug, Serialize)]
pub struct TemplateSummary {
    /// Its id (for `instantiate_template`, `update_motion_template`).
    pub template_id: String,
    /// `meta.name`.
    pub name: String,
    /// `meta.description`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Version of its source.
    pub version: u32,
    /// Its browser category (`title`, `lowerThird`, `callToAction`, `transition`, `other`).
    pub category: &'static str,
    /// Length of new instances.
    pub default_duration: TimeOutput,
    /// Its parameters, in declaration order.
    pub params: Vec<ParamSummary>,
    /// Items using it.
    pub instances: usize,
    /// The library entry it was copied from (`list_templates` with `source: "library"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_template_id: Option<String>,
}

/// One parameter as agents read it.
#[derive(Debug, Serialize)]
pub struct ParamSummary {
    /// The key in `params`.
    pub name: String,
    /// The inspector's label.
    pub label: String,
    /// `string`, `number`, `color`, `boolean`, `enum`, `font` or `asset`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Value when an instance sets none (`null` for assets).
    pub default: Value,
    /// Smallest number allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Largest number allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Options of an `enum`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
    /// Whether a string expects several lines.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub multiline: bool,
    /// Whether keyframes may animate it (numbers and colours).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub animatable: bool,
    /// The brand kit role new instances take their value from (`accent`, `heading`, `logo`…).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<&'static str>,
}

/// The summary of `template` in `project`.
#[must_use]
pub fn template_summary(project: &Project, template: &MotionTemplate) -> TemplateSummary {
    TemplateSummary {
        template_id: template.id.to_string(),
        name: template.name.clone(),
        description: template.description.clone(),
        version: template.version,
        category: template.category.name(),
        default_duration: TimeOutput::from(template.default_duration),
        params: template.params.iter().map(param_summary).collect(),
        instances: project
            .items()
            .filter(|item| {
                matches!(&item.kind, se_domain::ItemKind::MotionTemplate(instance) if instance.template_id == template.id)
            })
            .count(),
        library_template_id: template
            .library
            .map(|link| link.template_id.to_string()),
    }
}

/// The summary of one parameter.
#[must_use]
pub fn param_summary(param: &TemplateParam) -> ParamSummary {
    let mut summary = ParamSummary {
        name: param.name.clone(),
        label: param.label.clone(),
        kind: param.spec.kind_name(),
        default: Value::Null,
        minimum: None,
        maximum: None,
        options: Vec::new(),
        multiline: false,
        animatable: false,
        brand: param.brand.map(se_domain::BrandRole::name),
    };
    match &param.spec {
        TemplateParamSpec::Text {
            default, multiline, ..
        } => {
            summary.default = Value::from(default.clone());
            summary.multiline = *multiline;
        }
        TemplateParamSpec::Number {
            default,
            minimum,
            maximum,
            ..
        } => {
            summary.default = Value::from(*default);
            summary.minimum = *minimum;
            summary.maximum = *maximum;
            summary.animatable = true;
        }
        TemplateParamSpec::Color { default } => {
            summary.default = Value::from(default.to_string());
            summary.animatable = true;
        }
        TemplateParamSpec::Boolean { default } => summary.default = Value::from(*default),
        TemplateParamSpec::Enum { default, options } => {
            summary.default = Value::from(default.clone());
            summary.options = options.iter().map(|option| option.value.clone()).collect();
        }
        TemplateParamSpec::Font { default } => summary.default = Value::from(default.clone()),
        TemplateParamSpec::Asset => {}
    }
    summary
}

/// The span of a new instance: `start` with `end` or `duration`, or `default_duration`.
///
/// # Errors
/// [`codes::INVALID_TIME`] for unreadable or out-of-range times.
pub fn template_range(
    scene: &WriteScene<'_>,
    start: &TimeInput,
    end: Option<&TimeInput>,
    duration: Option<&TimeInput>,
    default_duration: Flicks,
) -> Result<se_domain::TimeRange, ToolError> {
    if end.is_some() || duration.is_some() {
        return span(scene, start, end, duration);
    }
    let start = scene.time(start)?;
    let end = start
        .checked_add(default_duration)
        .ok_or_else(|| ToolError::new(codes::INVALID_TIME, "the end is too far"))?;
    se_domain::TimeRange::from_start_end(start, end)
        .map_err(|_| ToolError::new(codes::INVALID_TIME, "the start is invalid"))
}

/// Values for `template` read from `values` (parameter name → value; `null` resets to the
/// default). Numbers and colours also take `{"keyframes": [{time, value, easing?}]}` with times
/// relative to the item's start.
///
/// # Errors
/// [`codes::INVALID_ARGUMENTS`] naming the parameter, its type and what is expected.
pub fn param_values(
    scene: &WriteScene<'_>,
    template: &MotionTemplate,
    values: &Map<String, Value>,
) -> Result<BTreeMap<String, Option<TemplateParamValue>>, ToolError> {
    values
        .iter()
        .map(|(name, json)| {
            let param = template.param(name).ok_or_else(|| {
                let names: Vec<&str> = template.params.iter().map(|p| p.name.as_str()).collect();
                ToolError::invalid_arguments(format!(
                    "« {} » has no parameter '{name}'; its parameters are: {}",
                    template.name,
                    if names.is_empty() {
                        "none".to_owned()
                    } else {
                        names.join(", ")
                    }
                ))
            })?;
            if json.is_null() {
                return Ok((name.clone(), None));
            }
            let value = param_value(scene, param, json).map_err(|expected| {
                ToolError::invalid_arguments(format!(
                    "params.{name} ({}): {expected}",
                    param.spec.kind_name()
                ))
            })?;
            Ok((name.clone(), Some(value)))
        })
        .collect()
}

/// One value, or what was expected.
fn param_value(
    scene: &WriteScene<'_>,
    param: &TemplateParam,
    json: &Value,
) -> Result<TemplateParamValue, String> {
    let text = || {
        json.as_str()
            .map(str::to_owned)
            .ok_or_else(|| "give a string".to_owned())
    };
    Ok(match &param.spec {
        TemplateParamSpec::Text { .. } => TemplateParamValue::Text { value: text()? },
        TemplateParamSpec::Enum { .. } => TemplateParamValue::Enum { value: text()? },
        TemplateParamSpec::Font { .. } => TemplateParamValue::Font { value: text()? },
        TemplateParamSpec::Boolean { .. } => TemplateParamValue::Boolean {
            value: json.as_bool().ok_or("give true or false")?,
        },
        TemplateParamSpec::Asset => TemplateParamValue::Asset {
            value: Some(
                json.as_str()
                    .and_then(|id| AssetId::parse(id).ok())
                    .ok_or("give the asset_id of an image (list_assets), or null for none")?,
            ),
        },
        TemplateParamSpec::Number { .. } => TemplateParamValue::Number {
            value: animatable(scene, json, |value| {
                value.as_f64().ok_or_else(|| "a number".to_owned())
            })?,
        },
        TemplateParamSpec::Color { .. } => TemplateParamValue::Color {
            value: animatable(scene, json, |value| {
                let text = value.as_str().ok_or("a colour \"#rrggbb[aa]\"")?;
                color("value", text).map_err(|error| error.message)
            })?,
        },
    })
}

/// A static value, or `{"keyframes": […]}`.
fn animatable<T>(
    scene: &WriteScene<'_>,
    json: &Value,
    read: impl Fn(&Value) -> Result<T, String>,
) -> Result<Animatable<T>, String> {
    let Some(keyframes) = json.get("keyframes") else {
        return read(json)
            .map(Animatable::constant)
            .map_err(|expected| format!("give {expected}, or {{\"keyframes\": [...]}}"));
    };
    let inputs: Vec<KeyframeInput> =
        serde_json::from_value(keyframes.clone()).map_err(|error| {
            format!("keyframes are [{{\"time\": seconds, \"value\": …, \"easing\"?}}] ({error})")
        })?;
    if inputs.is_empty() {
        return Err("give at least one keyframe".to_owned());
    }
    inputs
        .iter()
        .map(|input| {
            let time: Flicks = scene.time(&input.time).map_err(|error| error.message)?;
            Ok(Keyframe {
                time,
                value: read(&input.value.to_json())
                    .map_err(|expected| format!("give {expected}"))?,
                easing: input
                    .easing
                    .map_or(Easing::Linear, crate::write_inputs::EasingInput::to_easing),
            })
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|keyframes| Animatable::Keyframed { keyframes })
}

/// What `create_motion_template` and `update_motion_template` answer about the pictures of the
/// check: one caption and image per frame.
pub fn push_frames(output: &mut ToolOutput, checked: &CheckedTemplate) {
    let names = ["entrance", "middle", "exit"];
    for (index, frame) in checked.frames.iter().enumerate() {
        let name = names.get(index).copied().unwrap_or("frame");
        output.push_text(format!(
            "Frame {} ({name}) at {} of an instance, default values",
            index + 1,
            TimeOutput::from(frame.time).timecode
        ));
        output.push_jpeg_base64(&frame.jpeg_base64);
    }
}

/// The instants of a check's pictures.
#[must_use]
pub fn frame_times(checked: &CheckedTemplate) -> Vec<TimeOutput> {
    checked
        .frames
        .iter()
        .map(|frame| TimeOutput::from(frame.time))
        .collect()
}

/// A colour as agents write it.
#[must_use]
pub fn color_text(color: Color) -> String {
    color.to_string()
}
