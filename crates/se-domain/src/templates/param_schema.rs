//! One parameter of a template's `params` export: a JSON Schema fragment read into a
//! [`TemplateParam`] (the table of supported forms is in `definition.rs`).

use serde_json::{Map, Value};

use super::DefinitionProblem;
use crate::{
    BrandRole, Color, DEFAULT_TEXT_MAX_LENGTH, TemplateChoice, TemplateParam, TemplateParamSpec,
};

/// Keys every parameter may carry.
const COMMON_KEYS: [&str; 6] = [
    "type",
    "title",
    "description",
    "default",
    "examples",
    "$comment",
];

pub(super) fn parse_param(
    name: &str,
    schema: &Value,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParam> {
    let path = format!("params.{name}");
    let Some(schema) = schema.as_object() else {
        problems.push(DefinitionProblem::new(
            path,
            "describe the parameter with an object such as { type: 'string', default: '…' }",
        ));
        return None;
    };
    let before = problems.len();
    let label =
        optional_string(schema, "title", &path, problems).unwrap_or_else(|| name.to_owned());
    let description = optional_string(schema, "description", &path, problems);
    let spec = parse_spec(schema, &path, problems)?;
    let brand = parse_brand(schema.get("brand"), &spec, &path, problems);
    (problems.len() == before).then(|| TemplateParam {
        name: name.to_owned(),
        label,
        description,
        spec,
        brand,
    })
}

/// The `brand` role of a colour, font or asset parameter (checked against its kind; other kinds
/// already refused the key as unknown).
fn parse_brand(
    value: Option<&Value>,
    spec: &TemplateParamSpec,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<BrandRole> {
    let value = value.filter(|value| !value.is_null())?;
    let kind = spec.kind_name();
    let roles = BrandRole::names_for_kind(kind);
    if roles.is_empty() {
        return None;
    }
    let role = value
        .as_str()
        .and_then(BrandRole::from_name)
        .filter(|role| role.param_kind() == kind);
    if role.is_none() {
        problems.push(DefinitionProblem::new(
            format!("{path}.brand"),
            format!("use one of: {roles} (a brand kit role for a {kind} parameter)"),
        ));
    }
    role
}

fn parse_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    let format = schema.get("format").and_then(Value::as_str);
    let problem = match schema.get("type").and_then(Value::as_str) {
        Some("string") => return string_spec(schema, format, path, problems),
        Some("boolean") if format.is_none() => return boolean_spec(schema, path, problems),
        Some(kind @ ("number" | "integer")) if format.is_none() => {
            return number_spec(schema, path, kind == "integer", problems);
        }
        Some("number" | "integer" | "boolean") => DefinitionProblem::new(
            format!("{path}.format"),
            "`format` applies to strings only (color, font, asset)",
        ),
        Some(other) => DefinitionProblem::new(
            format!("{path}.type"),
            format!("type '{other}' is not supported; use string, number, integer or boolean"),
        ),
        None => DefinitionProblem::new(
            format!("{path}.type"),
            "give the parameter a type: string, number, integer or boolean",
        ),
    };
    problems.push(problem);
    None
}

fn string_spec(
    schema: &Map<String, Value>,
    format: Option<&str>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    if schema.contains_key("enum") {
        return enum_spec(schema, path, problems);
    }
    match format {
        None => text_spec(schema, path, problems),
        Some("color") => color_spec(schema, path, problems),
        Some("font") => font_spec(schema, path, problems),
        Some("asset" | "image") => Some(asset_spec(schema, path, problems)),
        Some(other) => {
            problems.push(DefinitionProblem::new(
                format!("{path}.format"),
                format!("format '{other}' is not supported; use 'color', 'font' or 'asset'"),
            ));
            None
        }
    }
}

fn boolean_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    refuse_unknown_keys(schema, path, &COMMON_KEYS, problems);
    let default = schema.get("default").and_then(Value::as_bool);
    require(default, path, "a boolean default (true or false)", problems)
        .map(|default| TemplateParamSpec::Boolean { default })
}

fn font_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    refuse_unknown_keys(schema, path, &with_common(&["format", "brand"]), problems);
    let default = schema
        .get("default")
        .and_then(Value::as_str)
        .filter(|family| !family.trim().is_empty());
    require(
        default,
        path,
        "a font family default such as 'Inter'",
        problems,
    )
    .map(|default| TemplateParamSpec::Font {
        default: default.to_owned(),
    })
}

fn asset_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> TemplateParamSpec {
    refuse_unknown_keys(schema, path, &with_common(&["format", "brand"]), problems);
    if !matches!(schema.get("default"), None | Some(Value::Null)) {
        problems.push(DefinitionProblem::new(
            format!("{path}.default"),
            "an asset parameter has no default: leave `default` out (the template receives null \
             until the user picks an image)",
        ));
    }
    TemplateParamSpec::Asset
}

fn number_spec(
    schema: &Map<String, Value>,
    path: &str,
    integer: bool,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    refuse_unknown_keys(
        schema,
        path,
        &with_common(&["minimum", "maximum", "multipleOf"]),
        problems,
    );
    let mut bound = |key: &str| match schema.get(key) {
        None | Some(Value::Null) => None,
        Some(value) => {
            let number = value.as_f64();
            if number.is_none() {
                problems.push(DefinitionProblem::new(
                    format!("{path}.{key}"),
                    "must be a number",
                ));
            }
            number
        }
    };
    let minimum = bound("minimum");
    let maximum = bound("maximum");
    let step = bound("multipleOf").or(integer.then_some(1.0));
    let default = schema.get("default").and_then(Value::as_f64);
    require(default, path, "a number default", problems).map(|default| TemplateParamSpec::Number {
        default,
        minimum,
        maximum,
        step,
    })
}

fn color_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    refuse_unknown_keys(schema, path, &with_common(&["format", "brand"]), problems);
    let default = schema
        .get("default")
        .and_then(Value::as_str)
        .and_then(|text| Color::parse(text).ok());
    require(
        default,
        path,
        "a colour default written #rrggbb or #rrggbbaa",
        problems,
    )
    .map(|default| TemplateParamSpec::Color { default })
}

fn text_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    refuse_unknown_keys(
        schema,
        path,
        &with_common(&["multiline", "maxLength"]),
        problems,
    );
    let multiline = match schema.get("multiline") {
        None => false,
        Some(value) => value.as_bool().unwrap_or_else(|| {
            problems.push(DefinitionProblem::new(
                format!("{path}.multiline"),
                "must be true or false",
            ));
            false
        }),
    };
    let max_length = match schema.get("maxLength") {
        None => DEFAULT_TEXT_MAX_LENGTH,
        Some(value) => value
            .as_u64()
            .and_then(|length| u32::try_from(length).ok())
            .filter(|length| *length >= 1)
            .unwrap_or_else(|| {
                problems.push(DefinitionProblem::new(
                    format!("{path}.maxLength"),
                    "must be a whole number ≥ 1",
                ));
                DEFAULT_TEXT_MAX_LENGTH
            }),
    };
    let default = schema.get("default").and_then(Value::as_str);
    require(default, path, "a string default (may be empty)", problems).map(|default| {
        TemplateParamSpec::Text {
            default: default.to_owned(),
            multiline,
            max_length,
        }
    })
}

fn enum_spec(
    schema: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<TemplateParamSpec> {
    refuse_unknown_keys(
        schema,
        path,
        &with_common(&["enum", "enumLabels"]),
        problems,
    );
    let values: Option<Vec<String>> =
        schema
            .get("enum")
            .and_then(Value::as_array)
            .and_then(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().map(str::to_owned))
                    .collect()
            });
    let Some(values) = values else {
        problems.push(DefinitionProblem::new(
            format!("{path}.enum"),
            "list the options as strings: enum: ['left', 'center', 'right']",
        ));
        return None;
    };
    let labels: Option<Vec<String>> = match schema.get("enumLabels") {
        None => None,
        Some(labels) => {
            let labels: Option<Vec<String>> = labels.as_array().and_then(|labels| {
                labels
                    .iter()
                    .map(|label| label.as_str().map(str::to_owned))
                    .collect()
            });
            if labels
                .as_ref()
                .is_none_or(|labels| labels.len() != values.len())
            {
                problems.push(DefinitionProblem::new(
                    format!("{path}.enumLabels"),
                    "give one string label per option, in the order of `enum`",
                ));
            }
            labels
        }
    };
    let options = values
        .iter()
        .enumerate()
        .map(|(index, value)| TemplateChoice {
            value: value.clone(),
            label: labels
                .as_ref()
                .and_then(|labels| labels.get(index))
                .cloned()
                .unwrap_or_else(|| value.clone()),
        })
        .collect();
    let default = schema.get("default").and_then(Value::as_str);
    require(
        default,
        path,
        "a default that is one of the options",
        problems,
    )
    .map(|default| TemplateParamSpec::Enum {
        default: default.to_owned(),
        options,
    })
}

fn with_common(extra: &[&'static str]) -> Vec<&'static str> {
    COMMON_KEYS.iter().chain(extra).copied().collect()
}

fn require<T>(
    value: Option<T>,
    path: &str,
    expected: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<T> {
    if value.is_none() {
        problems.push(DefinitionProblem::new(
            format!("{path}.default"),
            format!("give {expected}"),
        ));
    }
    value
}

fn optional_string(
    schema: &Map<String, Value>,
    key: &str,
    path: &str,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<String> {
    match schema.get(key) {
        Some(Value::String(text)) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        None | Some(Value::Null | Value::String(_)) => None,
        Some(_) => {
            problems.push(DefinitionProblem::new(
                format!("{path}.{key}"),
                "must be a string",
            ));
            None
        }
    }
}

pub(super) fn refuse_unknown_keys(
    object: &Map<String, Value>,
    path: &str,
    known: &[&str],
    problems: &mut Vec<DefinitionProblem>,
) {
    for key in object.keys().filter(|key| !known.contains(&key.as_str())) {
        problems.push(DefinitionProblem::new(
            format!("{path}.{key}"),
            format!("'{key}' is not supported here; use: {}", known.join(", ")),
        ));
    }
}
