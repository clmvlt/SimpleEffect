//! Motion templates (prompt 24, ADR-0035): checking a template module in the renderer.
//!
//! Only the renderer can run template code (its sandboxed Workers, golden rule 7). When an agent
//! creates or updates a template, the core publishes [`crate::CoreEvent::TemplateCheckRequested`];
//! the renderer compiles the module, runs it in a sandbox, renders a few instants twice (time
//! budget, determinism) and answers with [`crate::Command::ReportTemplateCheck`]: the module's
//! `meta` and `params` exports (the core reads and checks them) and pictures for the agent, or
//! the problems found.

use se_domain::{Flicks, ProjectId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A template module the renderer must check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCheckRequest {
    /// Identifies the request in the report (a UUID).
    pub request_id: String,
    /// The project the template is for (its fonts and images).
    pub project_id: ProjectId,
    /// The TypeScript module.
    pub source: String,
    /// Frame the template draws in, in design pixels (the task sequence's aspect, shorter side
    /// 1080).
    pub width: u32,
    /// Height of that frame.
    pub height: u32,
    /// Width of the pictures returned, in pixels.
    pub picture_width: u32,
    /// Height of the pictures returned, in pixels.
    pub picture_height: u32,
}

/// What the renderer found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TemplateCheckOutcome {
    /// The module compiled, rendered within its budget and twice the same.
    Passed {
        /// `JSON.stringify` of the `meta` export.
        meta_json: String,
        /// The `params` export as JSON `[name, schema]` pairs in declaration order (a whole
        /// object schema is unwrapped first).
        params_json: String,
        /// Pictures of the entrance, middle and exit, rendered with the default values.
        frames: Vec<TemplateCheckFrame>,
        /// Longest render of one frame during the check, in milliseconds.
        slowest_frame_ms: f64,
    },
    /// The module cannot be used as it is.
    Failed {
        /// Every problem found, in the order met.
        problems: Vec<TemplateProblem>,
    },
}

/// One picture of a checked template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateCheckFrame {
    /// Instant drawn, relative to the instance's start.
    pub time: Flicks,
    /// The picture: a JPEG, base64-encoded.
    pub jpeg_base64: String,
}

/// One problem of a template module, read by the agent that wrote it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateProblem {
    /// When it was found.
    pub stage: TemplateCheckStage,
    /// What is wrong and how to fix it.
    pub message: String,
    /// Line in the module (1-based), when known.
    pub line: Option<u32>,
    /// Column in that line (1-based), when known.
    pub column: Option<u32>,
    /// Instant being rendered (relative to the instance's start), for render problems.
    pub time: Option<Flicks>,
}

/// The step of a template check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TemplateCheckStage {
    /// TypeScript could not be compiled (syntax), or the module imports something.
    Compile,
    /// The module threw while loading, or lacks `meta`, `params` or `render`, or they are not
    /// plain data.
    Exports,
    /// `render` threw, returned a promise, or used something templates may not use (`fetch`,
    /// `Date`, timers…).
    Render,
    /// A frame took longer than the budget, or never ended.
    Budget,
    /// The same instant rendered twice gave different pictures.
    Determinism,
    /// The core refused what `meta` or `params` declare.
    Definition,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_are_tagged_and_camel_case() {
        let passed = TemplateCheckOutcome::Passed {
            meta_json: "{}".to_owned(),
            params_json: "[]".to_owned(),
            frames: vec![TemplateCheckFrame {
                time: Flicks::one_second(),
                jpeg_base64: "/9j/".to_owned(),
            }],
            slowest_frame_ms: 4.5,
        };
        let json = serde_json::to_value(&passed).unwrap();
        assert_eq!(json["kind"], "passed");
        assert_eq!(json["paramsJson"], "[]");
        assert_eq!(json["frames"][0]["jpegBase64"], "/9j/");
        let failed = TemplateCheckOutcome::Failed {
            problems: vec![TemplateProblem {
                stage: TemplateCheckStage::Determinism,
                message: "differs".to_owned(),
                line: Some(3),
                column: None,
                time: None,
            }],
        };
        let json = serde_json::to_value(&failed).unwrap();
        assert_eq!(json["problems"][0]["stage"], "determinism");
        assert_eq!(json["problems"][0]["line"], 3);
    }
}
