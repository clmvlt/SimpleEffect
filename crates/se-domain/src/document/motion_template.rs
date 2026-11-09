//! Motion templates (ADR-0011, ADR-0035): the [`MotionTemplate`] stored in a project and the
//! [`TemplateItem`] instantiating it on a track.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Animatable, LibraryLink, MAX_TEMPLATE_PARAMS, Origin, TemplateCategory, TemplateParam,
    Transform2D,
};
use crate::{AssetId, color::Color, ids::TemplateId, time::Flicks};

/// Longest template name, in characters.
pub const MAX_TEMPLATE_NAME_CHARS: usize = 80;
/// Longest template description, in characters.
pub const MAX_TEMPLATE_DESCRIPTION_CHARS: usize = 500;
/// Largest template source, in bytes (UTF-8).
pub const MAX_TEMPLATE_SOURCE_BYTES: usize = 200_000;
/// Longest default duration of a template: one hour.
pub const MAX_TEMPLATE_DURATION: Flicks = Flicks::from_millis(3_600_000);

/// An AI-authored, parameterised render module stored in the project.
///
/// The TypeScript `source` is the single truth (ADR-0035): it exports `meta` (name, description,
/// default duration), `params` (the parameter schema) and `render(ctx, time, params)`. The
/// renderer compiles and runs it in a sandboxed Worker; the core keeps what `meta` and `params`
/// declared, checked, in the other fields. A template is added only once it compiled, rendered
/// within its budget and proved deterministic; a new source makes a new `version`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MotionTemplate {
    /// Stable id, referenced by template items.
    pub id: TemplateId,
    /// Name shown in the library and the inspector (`meta.name`).
    pub name: String,
    /// What it shows (`meta.description`; may be empty).
    #[serde(default)]
    pub description: String,
    /// Version of the source, 1 for a new template, bumped by each new source.
    #[serde(default = "first_version")]
    pub version: u32,
    /// The TypeScript module (`meta`, `params`, `render`).
    #[serde(default)]
    pub source: String,
    /// The parameters `params` declares, in declaration order.
    #[serde(default)]
    pub params: Vec<TemplateParam>,
    /// Length of new instances (`meta.defaultDuration`); > 0.
    #[serde(default = "default_template_duration")]
    pub default_duration: Flicks,
    /// Who wrote it.
    #[serde(default = "user_origin")]
    pub origin: Origin,
    /// Where the template browser files it (`meta.category`; schema version 10).
    #[serde(default)]
    pub category: TemplateCategory,
    /// The app's library entry it was copied from, if any (schema version 10).
    #[serde(default)]
    pub library: Option<LibraryLink>,
}

impl MotionTemplate {
    /// The parameter called `name`.
    #[must_use]
    pub fn param(&self, name: &str) -> Option<&TemplateParam> {
        self.params.iter().find(|param| param.name == name)
    }

    /// What is wrong with the template, as `(field path, rule)` pairs (`params[2].spec`…);
    /// empty when it is usable.
    #[must_use]
    pub fn problems(&self) -> Vec<(String, String)> {
        let mut problems = Vec::new();
        let mut check = |ok: bool, field: &str, rule: String| {
            if !ok {
                problems.push((field.to_owned(), rule));
            }
        };
        let name_chars = self.name.trim().chars().count();
        check(
            (1..=MAX_TEMPLATE_NAME_CHARS).contains(&name_chars),
            "name",
            format!("must hold 1 to {MAX_TEMPLATE_NAME_CHARS} characters"),
        );
        check(
            self.description.chars().count() <= MAX_TEMPLATE_DESCRIPTION_CHARS,
            "description",
            format!("must hold at most {MAX_TEMPLATE_DESCRIPTION_CHARS} characters"),
        );
        check(
            self.version >= 1,
            "version",
            "must be at least 1".to_owned(),
        );
        check(
            !self.source.trim().is_empty() && self.source.len() <= MAX_TEMPLATE_SOURCE_BYTES,
            "source",
            format!("must be a module of 1 to {MAX_TEMPLATE_SOURCE_BYTES} bytes"),
        );
        check(
            Flicks::ZERO < self.default_duration && self.default_duration <= MAX_TEMPLATE_DURATION,
            "defaultDuration",
            "must be longer than 0 s and at most one hour".to_owned(),
        );
        check(
            self.params.len() <= MAX_TEMPLATE_PARAMS,
            "params",
            format!("must hold at most {MAX_TEMPLATE_PARAMS} parameters"),
        );
        for (index, param) in self.params.iter().enumerate() {
            if self.params[..index]
                .iter()
                .any(|other| other.name == param.name)
            {
                problems.push((
                    format!("params[{index}].name"),
                    format!("'{}' is declared twice", param.name),
                ));
            }
            for (field, rule) in param.problems() {
                problems.push((format!("params[{index}].{field}"), rule));
            }
        }
        problems
    }
}

const fn first_version() -> u32 {
    1
}

/// Five seconds: the length of templates read from documents written before prompt 24.
const fn default_template_duration() -> Flicks {
    Flicks::from_millis(5000)
}

const fn user_origin() -> Origin {
    Origin::User
}

/// An item rendering a motion template with parameter values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateItem {
    /// The template rendered (must exist in the project).
    pub template_id: TemplateId,
    /// Parameter values by parameter name; a name missing here takes the schema default.
    pub params: BTreeMap<String, TemplateParamValue>,
    /// Placement of the template's frame (it draws over the whole frame, then is placed like any
    /// graphic: position, scale, rotation, opacity, blur). Centred and natural by default.
    #[serde(default)]
    pub transform: Transform2D,
}

/// The value of one template parameter; its kind matches the parameter's (validation).
///
/// Numbers and colours are animatable (keyframe times relative to the item's start, like every
/// keyframe). Reading accepts a bare number or colour as a static value (documents written before
/// prompt 24).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TemplateParamValue {
    /// A string parameter (one line or several).
    Text {
        /// The text.
        value: String,
    },
    /// A number parameter.
    Number {
        /// The number, possibly keyframed.
        value: Animatable<f64>,
    },
    /// A boolean parameter.
    Boolean {
        /// The flag.
        value: bool,
    },
    /// A colour parameter.
    Color {
        /// The colour, possibly keyframed.
        value: Animatable<Color>,
    },
    /// One of the parameter's options.
    #[serde(rename = "enum")]
    Enum {
        /// The option chosen.
        value: String,
    },
    /// A font family of the catalogue.
    Font {
        /// The family name.
        value: String,
    },
    /// An image asset of the project, or none.
    Asset {
        /// The asset, `null` for none.
        value: Option<AssetId>,
    },
}

impl TemplateParamValue {
    /// Name of the value's kind, as in the schema (`number`, `enum`…).
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Text { .. } => "string",
            Self::Number { .. } => "number",
            Self::Boolean { .. } => "boolean",
            Self::Color { .. } => "color",
            Self::Enum { .. } => "enum",
            Self::Font { .. } => "font",
            Self::Asset { .. } => "asset",
        }
    }

    /// Whether the value has keyframes.
    #[must_use]
    pub const fn is_keyframed(&self) -> bool {
        match self {
            Self::Number { value } => value.is_keyframed(),
            Self::Color { value } => value.is_keyframed(),
            _ => false,
        }
    }
}
