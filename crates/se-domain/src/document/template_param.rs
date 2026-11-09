//! [`TemplateParam`]: one parameter of a motion template's schema (ADR-0035), the controls the
//! inspector generates and the values template items may hold.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Animatable, TemplateParamValue};
use crate::color::Color;

/// Most parameters a template may declare.
pub const MAX_TEMPLATE_PARAMS: usize = 32;
/// Longest parameter name, in characters.
pub const MAX_PARAM_NAME_CHARS: usize = 48;
/// Longest text value when the schema sets no `maxLength`, in characters.
pub const DEFAULT_TEXT_MAX_LENGTH: u32 = 2000;
/// Most options of an `enum` parameter.
pub const MAX_ENUM_OPTIONS: usize = 32;

/// One parameter of a template: a name the code reads (`params.title`), a label for the
/// inspector, and what values it takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateParam {
    /// The key in `params` (an identifier: letters, digits, `_`, not starting with a digit).
    pub name: String,
    /// What the inspector shows (`title` in the schema; the name when absent).
    pub label: String,
    /// A longer explanation (tooltip), if any.
    pub description: Option<String>,
    /// The kind of values and their constraints.
    pub spec: TemplateParamSpec,
}

/// The values a parameter takes, with its default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TemplateParamSpec {
    /// A string (an input, or a text area when `multiline`).
    #[serde(rename = "string")]
    Text {
        /// Value when the item sets none.
        default: String,
        /// Whether line breaks are expected (a text area in the inspector).
        multiline: bool,
        /// Longest value, in characters.
        max_length: u32,
    },
    /// A number, animatable (a scrub input with keyframes).
    Number {
        /// Value when the item sets none.
        default: f64,
        /// Smallest value allowed, if any.
        minimum: Option<f64>,
        /// Largest value allowed, if any.
        maximum: Option<f64>,
        /// Step of the inspector's control, if any (> 0).
        step: Option<f64>,
    },
    /// A colour, animatable (a colour field with keyframes).
    Color {
        /// Value when the item sets none.
        default: Color,
    },
    /// A flag (a switch).
    Boolean {
        /// Value when the item sets none.
        default: bool,
    },
    /// One option of a list (a select).
    #[serde(rename = "enum")]
    Enum {
        /// Value when the item sets none (one of the options).
        default: String,
        /// The options, in order.
        options: Vec<TemplateChoice>,
    },
    /// A font family of the catalogue (a font picker).
    Font {
        /// Family when the item sets none.
        default: String,
    },
    /// An image asset of the project (a media picker); none by default.
    Asset,
}

/// One option of an `enum` parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateChoice {
    /// The value the code receives.
    pub value: String,
    /// What the inspector shows (the value when the schema gives no labels).
    pub label: String,
}

impl TemplateParam {
    /// The value used when an item sets none.
    #[must_use]
    pub fn default_value(&self) -> TemplateParamValue {
        match &self.spec {
            TemplateParamSpec::Text { default, .. } => TemplateParamValue::Text {
                value: default.clone(),
            },
            TemplateParamSpec::Number { default, .. } => TemplateParamValue::Number {
                value: Animatable::constant(*default),
            },
            TemplateParamSpec::Color { default } => TemplateParamValue::Color {
                value: Animatable::constant(*default),
            },
            TemplateParamSpec::Boolean { default } => {
                TemplateParamValue::Boolean { value: *default }
            }
            TemplateParamSpec::Enum { default, .. } => TemplateParamValue::Enum {
                value: default.clone(),
            },
            TemplateParamSpec::Font { default } => TemplateParamValue::Font {
                value: default.clone(),
            },
            TemplateParamSpec::Asset => TemplateParamValue::Asset { value: None },
        }
    }

    /// What is wrong with the declaration itself (name, label, default, constraints), as
    /// `(field, rule)` pairs; empty when it is usable.
    #[must_use]
    pub fn problems(&self) -> Vec<(&'static str, String)> {
        let mut problems = Vec::new();
        if !is_param_name(&self.name) {
            problems.push((
                "name",
                format!(
                    "'{}' is not a parameter name: use letters, digits and _ (not starting with a \
                     digit), at most {MAX_PARAM_NAME_CHARS} characters",
                    self.name
                ),
            ));
        }
        if self.label.trim().is_empty() {
            problems.push(("label", "must not be empty".to_owned()));
        }
        if let Some(problem) = self.spec_problem() {
            problems.push(("spec", problem));
        }
        if let Some(problem) = self.value_problem(&self.default_value()) {
            problems.push(("spec.default", problem));
        }
        problems
    }

    /// What is wrong with the constraints of the spec, if anything.
    fn spec_problem(&self) -> Option<String> {
        match &self.spec {
            TemplateParamSpec::Text { max_length, .. } if *max_length == 0 => {
                Some("maxLength must be at least 1".to_owned())
            }
            TemplateParamSpec::Number {
                default,
                minimum,
                maximum,
                step,
            } => number_spec_problem(*default, *minimum, *maximum, *step),
            TemplateParamSpec::Enum { options, .. } => enum_spec_problem(options),
            _ => None,
        }
    }

    /// Why `value` does not fit this parameter (wrong kind, out of range, unknown option…), or
    /// `None`. Asset references are checked against the project by validation.
    #[must_use]
    pub fn value_problem(&self, value: &TemplateParamValue) -> Option<String> {
        match (&self.spec, value) {
            (TemplateParamSpec::Text { max_length, .. }, TemplateParamValue::Text { value }) => {
                let length = value.chars().count();
                (length > *max_length as usize).then(|| {
                    format!("the text has {length} characters; at most {max_length} are allowed")
                })
            }
            (
                TemplateParamSpec::Number {
                    minimum, maximum, ..
                },
                TemplateParamValue::Number { value },
            ) => value
                .values()
                .into_iter()
                .find(|number| !in_bounds(**number, *minimum, *maximum))
                .map(|number| format!("{number} is not {}", bounds_text(*minimum, *maximum))),
            (TemplateParamSpec::Enum { options, .. }, TemplateParamValue::Enum { value }) => {
                (!options.iter().any(|option| option.value == *value)).then(|| {
                    format!(
                        "'{value}' is not an option; use one of: {}",
                        option_list(options)
                    )
                })
            }
            (TemplateParamSpec::Font { .. }, TemplateParamValue::Font { value }) => value
                .trim()
                .is_empty()
                .then(|| "the font family must not be empty".to_owned()),
            // Colours, flags and assets fit when the kinds agree; any other pair is a mismatch.
            _ => (self.spec.kind_name() != value.kind_name()).then(|| {
                format!(
                    "a {} value was given to a {} parameter",
                    value.kind_name(),
                    self.spec.kind_name()
                )
            }),
        }
    }
}

impl TemplateParamSpec {
    /// Name of the kind, as in the schema (`string`, `number`, `color`, `boolean`, `enum`,
    /// `font`, `asset`).
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Text { .. } => "string",
            Self::Number { .. } => "number",
            Self::Color { .. } => "color",
            Self::Boolean { .. } => "boolean",
            Self::Enum { .. } => "enum",
            Self::Font { .. } => "font",
            Self::Asset => "asset",
        }
    }
}

/// Whether `name` is a usable parameter name.
#[must_use]
pub fn is_param_name(name: &str) -> bool {
    let mut characters = name.chars();
    let starts_well = characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_');
    starts_well
        && name.chars().count() <= MAX_PARAM_NAME_CHARS
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn number_spec_problem(
    default: f64,
    minimum: Option<f64>,
    maximum: Option<f64>,
    step: Option<f64>,
) -> Option<String> {
    let finite = [Some(default), minimum, maximum, step]
        .into_iter()
        .flatten()
        .all(f64::is_finite);
    if !finite {
        return Some("default, minimum, maximum and step must be finite numbers".to_owned());
    }
    if let (Some(minimum), Some(maximum)) = (minimum, maximum)
        && minimum > maximum
    {
        return Some(format!(
            "minimum {minimum} is greater than maximum {maximum}"
        ));
    }
    step.filter(|step| *step <= 0.0)
        .map(|step| format!("step (multipleOf) must be > 0, got {step}"))
}

fn enum_spec_problem(options: &[TemplateChoice]) -> Option<String> {
    if options.is_empty() || options.len() > MAX_ENUM_OPTIONS {
        return Some(format!(
            "an enum needs between 1 and {MAX_ENUM_OPTIONS} options, got {}",
            options.len()
        ));
    }
    if options.iter().any(|option| option.value.is_empty()) {
        return Some("enum options must not be empty strings".to_owned());
    }
    let duplicate = options.iter().enumerate().find(|(index, option)| {
        options[..*index]
            .iter()
            .any(|other| other.value == option.value)
    });
    duplicate.map(|(_, option)| format!("option '{}' is listed twice", option.value))
}

fn in_bounds(value: f64, minimum: Option<f64>, maximum: Option<f64>) -> bool {
    value.is_finite()
        && minimum.is_none_or(|minimum| value >= minimum)
        && maximum.is_none_or(|maximum| value <= maximum)
}

fn bounds_text(minimum: Option<f64>, maximum: Option<f64>) -> String {
    match (minimum, maximum) {
        (Some(minimum), Some(maximum)) => format!("between {minimum} and {maximum}"),
        (Some(minimum), None) => format!("at least {minimum}"),
        (None, Some(maximum)) => format!("at most {maximum}"),
        (None, None) => "a finite number".to_owned(),
    }
}

fn option_list(options: &[TemplateChoice]) -> String {
    options
        .iter()
        .map(|option| format!("'{}'", option.value))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Easing, Keyframe, time::Flicks};

    fn number(minimum: Option<f64>, maximum: Option<f64>) -> TemplateParam {
        TemplateParam {
            name: "speed".to_owned(),
            label: "Speed".to_owned(),
            description: None,
            spec: TemplateParamSpec::Number {
                default: 1.0,
                minimum,
                maximum,
                step: Some(0.1),
            },
        }
    }

    fn choice(values: &[&str]) -> TemplateParam {
        TemplateParam {
            name: "align".to_owned(),
            label: "Align".to_owned(),
            description: None,
            spec: TemplateParamSpec::Enum {
                default: values.first().copied().unwrap_or("").to_owned(),
                options: values
                    .iter()
                    .map(|value| TemplateChoice {
                        value: (*value).to_owned(),
                        label: (*value).to_owned(),
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn names_are_identifiers() {
        assert!(is_param_name("title"));
        assert!(is_param_name("_accent2"));
        assert!(!is_param_name("2fast"));
        assert!(!is_param_name("with-dash"));
        assert!(!is_param_name(""));
        assert!(!is_param_name(&"a".repeat(MAX_PARAM_NAME_CHARS + 1)));
    }

    #[test]
    fn a_sound_declaration_has_no_problem() {
        assert_eq!(number(Some(0.0), Some(2.0)).problems(), []);
        assert_eq!(choice(&["left", "right"]).problems(), []);
    }

    #[test]
    fn declarations_report_their_problems() {
        let inverted = number(Some(3.0), Some(2.0)).problems();
        assert!(
            inverted
                .iter()
                .any(|(field, rule)| *field == "spec" && rule.contains("greater than maximum"))
        );
        let outside = number(Some(2.0), Some(3.0)).problems();
        assert!(outside.iter().any(|(field, _)| *field == "spec.default"));
        let twice = choice(&["left", "left"]).problems();
        assert!(twice.iter().any(|(_, rule)| rule.contains("listed twice")));
        assert_eq!(
            choice(&[]).problems().len(),
            2,
            "no option and an unknown default"
        );
    }

    #[test]
    fn values_are_checked_against_the_parameter() {
        let speed = number(Some(0.0), Some(2.0));
        let keyframed = TemplateParamValue::Number {
            value: Animatable::Keyframed {
                keyframes: vec![
                    Keyframe {
                        time: Flicks::ZERO,
                        value: 1.0,
                        easing: Easing::Linear,
                    },
                    Keyframe {
                        time: Flicks::one_second(),
                        value: 4.0,
                        easing: Easing::Linear,
                    },
                ],
            },
        };
        assert_eq!(
            speed.value_problem(&keyframed).as_deref(),
            Some("4 is not between 0 and 2")
        );
        let text = TemplateParamValue::Text {
            value: "fast".to_owned(),
        };
        assert_eq!(
            speed.value_problem(&text).as_deref(),
            Some("a string value was given to a number parameter")
        );
        let align = choice(&["left", "right"]);
        let centre = TemplateParamValue::Enum {
            value: "centre".to_owned(),
        };
        assert_eq!(
            align.value_problem(&centre).as_deref(),
            Some("'centre' is not an option; use one of: 'left', 'right'")
        );
        assert_eq!(align.value_problem(&align.default_value()), None);
    }
}
