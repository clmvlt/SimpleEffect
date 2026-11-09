//! [`parse_definition`]: the `meta` and `params` exports of a template module, as JSON, into a
//! checked [`TemplateDefinition`].
//!
//! `params` maps each parameter name to a JSON Schema fragment (or is a whole
//! `{"type": "object", "properties": {…}}` schema). The renderer sends it as a list of
//! `[name, schema]` pairs in declaration order (JSON objects lose their order here); an object is
//! read in key order.
//!
//! | Schema | Parameter |
//! |---|---|
//! | `{"type": "string", "default": "…", "multiline"?, "maxLength"?}` | text |
//! | `{"type": "number" \| "integer", "default": n, "minimum"?, "maximum"?, "multipleOf"?}` | number (animatable; `integer` steps by 1) |
//! | `{"type": "string", "format": "color", "default": "#rrggbb[aa]"}` | colour (animatable) |
//! | `{"type": "boolean", "default": b}` | flag |
//! | `{"type": "string", "enum": […], "enumLabels"?: […], "default": "…"}` | one option |
//! | `{"type": "string", "format": "font", "default": "Inter"}` | font family |
//! | `{"type": "string", "format": "asset"}` | image asset (no default) |
//!
//! Every parameter takes `title` (inspector label) and `description`; `examples` and `$comment`
//! are ignored; any other key is refused so typos come back to the agent. Colour, font and asset
//! parameters may also take `brand` (prompt 25): a brand kit role (`primary`, `secondary`,
//! `accent`, `text`, `background` for colours, `heading`, `body` for fonts, `logo` for assets)
//! whose value new instances start with in a project using a kit.
//!
//! `meta` holds `name`, `description`, `defaultDuration` (seconds) and, since prompt 25, an
//! optional `category` (`title`, `lowerThird`, `callToAction`, `transition`, `other`).

use serde_json::Value;

use super::param_schema::{parse_param, refuse_unknown_keys};
use crate::{
    MAX_TEMPLATE_DURATION, MotionTemplate, Origin, TemplateCategory, TemplateId, TemplateParam,
    time::Flicks,
};

/// What a template module declares, checked.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateDefinition {
    /// `meta.name`.
    pub name: String,
    /// `meta.description` (empty when absent).
    pub description: String,
    /// `meta.defaultDuration`, in flicks.
    pub default_duration: Flicks,
    /// `meta.category` (`other` when absent).
    pub category: TemplateCategory,
    /// The parameters of `params`, in declaration order.
    pub params: Vec<TemplateParam>,
}

/// The checked fields of `meta`.
struct Meta {
    name: String,
    description: String,
    default_duration: Flicks,
    category: TemplateCategory,
}

/// One problem of a module's declarations: where (`meta.name`, `params.title.default`) and what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionProblem {
    /// Path in the module's exports.
    pub path: String,
    /// What is wrong and how to fix it.
    pub message: String,
}

impl DefinitionProblem {
    pub(super) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl TemplateDefinition {
    /// The template made of this definition, checked as a whole (name length, unique names…).
    ///
    /// # Errors
    /// Every problem of the template, with paths in the module's terms.
    pub fn into_template(
        self,
        id: TemplateId,
        source: String,
        version: u32,
        origin: Origin,
    ) -> Result<MotionTemplate, Vec<DefinitionProblem>> {
        let template = MotionTemplate {
            id,
            name: self.name.trim().to_owned(),
            description: self.description.trim().to_owned(),
            version,
            source,
            params: self.params,
            default_duration: self.default_duration,
            origin,
            category: self.category,
            library: None,
        };
        let problems: Vec<DefinitionProblem> = template
            .problems()
            .into_iter()
            .map(|(field, rule)| DefinitionProblem::new(module_path(&template, &field), rule))
            .collect();
        if problems.is_empty() {
            Ok(template)
        } else {
            Err(problems)
        }
    }
}

/// `params[2].spec.default` → `params.<name>.default`; other fields under `meta`.
fn module_path(template: &MotionTemplate, field: &str) -> String {
    let Some(rest) = field.strip_prefix("params[") else {
        return match field {
            "source" => "source".to_owned(),
            other => format!("meta.{other}"),
        };
    };
    let (index, tail) = rest.split_once(']').unwrap_or((rest, ""));
    let name = index
        .parse::<usize>()
        .ok()
        .and_then(|index| template.params.get(index))
        .map_or(index, |param| param.name.as_str());
    let tail = tail.trim_start_matches('.').trim_start_matches("spec.");
    if tail.is_empty() || tail == "spec" {
        format!("params.{name}")
    } else {
        format!("params.{name}.{tail}")
    }
}

/// [`parse_definition`] of the two exports written as JSON text (what the renderer sends).
///
/// # Errors
/// An export that is not JSON, or every problem [`parse_definition`] finds.
pub fn parse_definition_json(
    meta_json: &str,
    params_json: &str,
) -> Result<TemplateDefinition, Vec<DefinitionProblem>> {
    let read = |json: &str, export: &str| {
        serde_json::from_str::<Value>(json).map_err(|error| {
            vec![DefinitionProblem::new(
                export,
                format!(
                    "the `{export}` export is not plain data ({error}): use only strings, numbers, booleans, arrays and objects"
                ),
            )]
        })
    };
    parse_definition(&read(meta_json, "meta")?, &read(params_json, "params")?)
}

/// Reads the `meta` and `params` exports of a template module.
///
/// # Errors
/// Every problem found (not just the first), with its path and an actionable message.
pub fn parse_definition(
    meta: &Value,
    params: &Value,
) -> Result<TemplateDefinition, Vec<DefinitionProblem>> {
    let mut problems = Vec::new();
    let meta = parse_meta(meta, &mut problems);
    let params = parse_params(params, &mut problems);
    match meta {
        Some(meta) if problems.is_empty() => Ok(TemplateDefinition {
            name: meta.name,
            description: meta.description,
            default_duration: meta.default_duration,
            category: meta.category,
            params,
        }),
        _ => Err(problems),
    }
}

fn parse_meta(meta: &Value, problems: &mut Vec<DefinitionProblem>) -> Option<Meta> {
    let Some(meta) = meta.as_object() else {
        problems.push(DefinitionProblem::new(
            "meta",
            "export `meta` as an object: { name, description, defaultDuration }",
        ));
        return None;
    };
    refuse_unknown_keys(
        meta,
        "meta",
        &["name", "description", "defaultDuration", "category"],
        problems,
    );
    let name = match meta.get("name").and_then(Value::as_str) {
        Some(name) if !name.trim().is_empty() => Some(name.to_owned()),
        _ => {
            problems.push(DefinitionProblem::new(
                "meta.name",
                "give the template a name (a non-empty string)",
            ));
            None
        }
    };
    let description = match meta.get("description") {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(_) => {
            problems.push(DefinitionProblem::new(
                "meta.description",
                "must be a string",
            ));
            String::new()
        }
    };
    let duration = meta
        .get("defaultDuration")
        .and_then(Value::as_f64)
        .filter(|seconds| *seconds > 0.0)
        .and_then(|seconds| Flicks::from_seconds_f64(seconds).ok())
        .filter(|duration| *duration > Flicks::ZERO && *duration <= MAX_TEMPLATE_DURATION);
    if duration.is_none() {
        problems.push(DefinitionProblem::new(
            "meta.defaultDuration",
            "give the default length of an instance in seconds: a number > 0 and at most 3600",
        ));
    }
    let category = parse_category(meta.get("category"), problems);
    Some(Meta {
        name: name?,
        description,
        default_duration: duration?,
        category,
    })
}

fn parse_category(
    value: Option<&Value>,
    problems: &mut Vec<DefinitionProblem>,
) -> TemplateCategory {
    match value {
        None | Some(Value::Null) => TemplateCategory::Other,
        Some(value) => value
            .as_str()
            .and_then(TemplateCategory::from_name)
            .unwrap_or_else(|| {
                problems.push(DefinitionProblem::new(
                    "meta.category",
                    format!(
                        "use one of: {} (or leave it out)",
                        TemplateCategory::names()
                    ),
                ));
                TemplateCategory::Other
            }),
    }
}

fn parse_params(params: &Value, problems: &mut Vec<DefinitionProblem>) -> Vec<TemplateParam> {
    let Some(entries) = param_entries(params, problems) else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter_map(|(name, schema)| parse_param(name, schema, problems))
        .collect()
}

/// The declared parameters in order: `[[name, schema], …]` pairs, or an object (unwrapping a
/// whole object schema).
fn param_entries<'a>(
    params: &'a Value,
    problems: &mut Vec<DefinitionProblem>,
) -> Option<Vec<(&'a str, &'a Value)>> {
    let not_declared = || {
        DefinitionProblem::new(
            "params",
            "export `params` as an object mapping each parameter name to its schema (use {} for \
             none)",
        )
    };
    if let Some(pairs) = params.as_array() {
        let entries: Option<Vec<(&str, &Value)>> = pairs
            .iter()
            .map(|pair| match pair.as_array().map(Vec::as_slice) {
                Some([Value::String(name), schema]) => Some((name.as_str(), schema)),
                _ => None,
            })
            .collect();
        if entries.is_none() {
            problems.push(not_declared());
        }
        return entries;
    }
    let Some(object) = params.as_object() else {
        problems.push(not_declared());
        return None;
    };
    let properties = if object.get("type").and_then(Value::as_str) == Some("object") {
        let Some(properties) = object.get("properties").and_then(Value::as_object) else {
            problems.push(DefinitionProblem::new(
                "params.properties",
                "an object schema needs `properties`",
            ));
            return None;
        };
        properties
    } else {
        object
    };
    Some(
        properties
            .iter()
            .map(|(name, schema)| (name.as_str(), schema))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::TemplateParamSpec;

    fn meta() -> Value {
        json!({ "name": "Lower third", "description": "Name and role", "defaultDuration": 5 })
    }

    fn paths(problems: &[DefinitionProblem]) -> Vec<&str> {
        problems
            .iter()
            .map(|problem| problem.path.as_str())
            .collect()
    }

    #[test]
    fn reads_every_kind_of_parameter_in_order() {
        let params = json!([
            ["name", { "type": "string", "title": "Nom", "default": "Jane Doe" }],
            ["role", { "type": "string", "default": "", "multiline": true, "maxLength": 80 }],
            ["slide", { "type": "number", "default": 120, "minimum": 0, "maximum": 400 }],
            ["count", { "type": "integer", "default": 3 }],
            ["accent", { "type": "string", "format": "color", "default": "#ffcc00" }],
            ["line", { "type": "boolean", "default": true }],
            ["align", { "type": "string", "enum": ["left", "right"], "enumLabels": ["Gauche", "Droite"], "default": "left" }],
            ["font", { "type": "string", "format": "font", "default": "Inter" }],
            ["logo", { "type": "string", "format": "asset", "title": "Logo" }]
        ]);
        let definition = parse_definition(&meta(), &params).unwrap();
        assert_eq!(definition.name, "Lower third");
        assert_eq!(definition.default_duration, Flicks::from_millis(5000));
        let names: Vec<&str> = definition.params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "name", "role", "slide", "count", "accent", "line", "align", "font", "logo"
            ]
        );
        assert_eq!(definition.params[0].label, "Nom");
        assert_eq!(definition.params[1].label, "role");
        assert!(matches!(
            definition.params[1].spec,
            TemplateParamSpec::Text {
                multiline: true,
                max_length: 80,
                ..
            }
        ));
        assert!(matches!(
            definition.params[3].spec,
            TemplateParamSpec::Number { step: Some(step), .. } if (step - 1.0).abs() < f64::EPSILON
        ));
        let TemplateParamSpec::Enum { options, .. } = &definition.params[6].spec else {
            panic!("align is an enum");
        };
        assert_eq!(options[1].label, "Droite");
        assert_eq!(definition.params[8].spec, TemplateParamSpec::Asset);
    }

    #[test]
    fn keeps_the_order_of_pairs() {
        let params = json!([
            ["zoom", { "type": "number", "default": 1 }],
            ["alpha", { "type": "boolean", "default": false }]
        ]);
        let definition = parse_definition(&meta(), &params).unwrap();
        let names: Vec<&str> = definition.params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["zoom", "alpha"]);
        let broken = parse_definition(&meta(), &json!([["zoom"]])).unwrap_err();
        assert_eq!(paths(&broken), ["params"]);
    }

    #[test]
    fn accepts_a_whole_object_schema() {
        let params = json!({ "type": "object", "properties": {
            "title": { "type": "string", "default": "Hello" } } });
        let definition = parse_definition(&meta(), &params).unwrap();
        assert_eq!(definition.params.len(), 1);
    }

    #[test]
    fn reports_every_problem_with_its_path() {
        let meta = json!({ "name": "", "defaultDuration": -1, "author": "me" });
        let params = json!({
            "a": { "type": "string" },
            "b": { "type": "number", "default": 1, "minimun": 0 },
            "c": { "type": "date", "default": "x" },
            "d": { "type": "string", "format": "color", "default": "yellow" },
            "e": { "type": "string", "format": "asset", "default": "logo.png" },
            "f": { "type": "string", "enum": ["x", 2], "default": "x" }
        });
        let problems = parse_definition(&meta, &params).unwrap_err();
        assert_eq!(
            paths(&problems),
            [
                "meta.author",
                "meta.name",
                "meta.defaultDuration",
                "params.a.default",
                "params.b.minimun",
                "params.c.type",
                "params.d.default",
                "params.e.default",
                "params.f.enum",
            ]
        );
        assert!(problems[4].message.contains("minimum"), "{problems:?}");
    }

    #[test]
    fn whole_template_checks_name_the_parameter() {
        let params = json!({
            "2fast": { "type": "string", "default": "" },
            "speed": { "type": "number", "default": 9, "maximum": 2 }
        });
        let problems = parse_definition(&meta(), &params)
            .unwrap()
            .into_template(
                TemplateId::from_test_index(1),
                "export function render() {}".to_owned(),
                1,
                Origin::User,
            )
            .unwrap_err();
        assert_eq!(
            paths(&problems),
            ["params.2fast.name", "params.speed.default"],
            "{problems:?}"
        );
    }

    #[test]
    fn reads_the_category_and_brand_roles() {
        let meta = json!({ "name": "CTA", "defaultDuration": 4, "category": "callToAction" });
        let params = json!([
            ["accent", { "type": "string", "format": "color", "default": "#ffcc00", "brand": "accent" }],
            ["font", { "type": "string", "format": "font", "default": "Inter", "brand": "heading" }],
            ["logo", { "type": "string", "format": "asset", "brand": "logo" }]
        ]);
        let definition = parse_definition(&meta, &params).unwrap();
        assert_eq!(definition.category, TemplateCategory::CallToAction);
        let roles: Vec<_> = definition.params.iter().map(|param| param.brand).collect();
        assert_eq!(
            roles,
            [
                Some(crate::BrandRole::Accent),
                Some(crate::BrandRole::Heading),
                Some(crate::BrandRole::Logo)
            ]
        );
        assert_eq!(
            parse_definition(&super::tests::meta(), &json!({}))
                .unwrap()
                .category,
            TemplateCategory::Other
        );
    }

    #[test]
    fn refuses_unknown_categories_and_mismatched_roles() {
        let meta = json!({ "name": "CTA", "defaultDuration": 4, "category": "intro" });
        let params = json!({
            "accent": { "type": "string", "format": "color", "default": "#ffcc00", "brand": "heading" },
            "speed": { "type": "number", "default": 1, "brand": "primary" }
        });
        let problems = parse_definition(&meta, &params).unwrap_err();
        assert_eq!(
            paths(&problems),
            ["meta.category", "params.accent.brand", "params.speed.brand"]
        );
        assert!(problems[0].message.contains("lowerThird"), "{problems:?}");
        assert!(
            problems[1].message.contains("primary, secondary"),
            "{problems:?}"
        );
    }

    #[test]
    fn a_missing_export_is_explained() {
        let problems = parse_definition(&Value::Null, &Value::Null).unwrap_err();
        assert_eq!(paths(&problems), ["meta", "params"]);
    }
}
