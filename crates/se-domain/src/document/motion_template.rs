//! Motion templates (ADR-0011): the [`MotionTemplate`] stored in a project and the
//! [`TemplateItem`] instantiating it on a track.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{color::Color, ids::TemplateId};

/// An AI-authored, parameterised render module stored in the project.
///
/// TODO(prompt-24): source code, parameter schema, version and validation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MotionTemplate {
    /// Stable id, referenced by template items.
    pub id: TemplateId,
    /// Name shown in the library and the inspector.
    pub name: String,
}

/// An item rendering a motion template with parameter values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateItem {
    /// The template rendered (must exist in the project).
    pub template_id: TemplateId,
    /// Parameter values by parameter name; a name missing here takes the schema default.
    pub params: BTreeMap<String, TemplateParamValue>,
}

/// The value of one template parameter.
///
/// TODO(prompt-24): enum, font and asset parameters (ADR-0011), validated against the schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TemplateParamValue {
    /// A string parameter.
    Text {
        /// The text.
        value: String,
    },
    /// A number parameter.
    Number {
        /// The number.
        value: f64,
    },
    /// A boolean parameter.
    Boolean {
        /// The flag.
        value: bool,
    },
    /// A colour parameter.
    Color {
        /// The colour.
        value: Color,
    },
}
