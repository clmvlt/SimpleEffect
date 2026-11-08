//! What tools return: [`ToolOutput`] (compact JSON text, then images) and [`ToolError`] (a stable
//! code and an actionable message), both as MCP tool results the agent reads (`docs/ai-tools.md`
//! « Conventions »).

use base64::Engine as _;
use rmcp::model::{CallToolResult, ContentBlock};
use se_app::AppError;
use se_protocol::ProtocolError;
use serde::Serialize;

/// Stable error codes of the tools themselves (errors of se-app use cases keep their protocol
/// code, e.g. `PROJECT_NOT_OPEN`, `RENDERER_UNAVAILABLE`).
pub mod codes {
    /// The arguments do not match the tool's input schema.
    pub const INVALID_ARGUMENTS: &str = "INVALID_ARGUMENTS";
    /// A time or range cannot be read, or lies outside what the tool accepts.
    pub const INVALID_TIME: &str = "INVALID_TIME";
    /// No sequence has that id or name.
    pub const SEQUENCE_NOT_FOUND: &str = "SEQUENCE_NOT_FOUND";
    /// No track of the sequence has that id or name.
    pub const TRACK_NOT_FOUND: &str = "TRACK_NOT_FOUND";
    /// No item has that id.
    pub const ITEM_NOT_FOUND: &str = "ITEM_NOT_FOUND";
    /// No asset has that id.
    pub const ASSET_NOT_FOUND: &str = "ASSET_NOT_FOUND";
    /// The asset has not been analysed (yet), or has no sound/picture to analyse.
    pub const NOT_ANALYSED: &str = "NOT_ANALYSED";
    /// Too many frames asked at once.
    pub const TOO_MANY_FRAMES: &str = "TOO_MANY_FRAMES";
    /// The caller's run is over: its token no longer gives access.
    pub const ACCESS_REVOKED: &str = "ACCESS_REVOKED";
    /// A bug in the tool.
    pub const INTERNAL: &str = "INTERNAL";
}

/// A successful tool result: content blocks in order.
#[derive(Debug, Default)]
pub struct ToolOutput {
    blocks: Vec<ContentBlock>,
}

impl ToolOutput {
    /// A result holding `value` as compact JSON.
    ///
    /// # Errors
    /// [`codes::INTERNAL`] if `value` cannot be serialised (a bug).
    pub fn json<T: Serialize>(value: &T) -> Result<Self, ToolError> {
        let mut output = Self::default();
        output.push_json(value)?;
        Ok(output)
    }

    /// Appends `value` as compact JSON text.
    ///
    /// # Errors
    /// [`codes::INTERNAL`] if `value` cannot be serialised (a bug).
    pub fn push_json<T: Serialize>(&mut self, value: &T) -> Result<(), ToolError> {
        let text = serde_json::to_string(value)
            .map_err(|error| ToolError::new(codes::INTERNAL, error.to_string()))?;
        self.blocks.push(ContentBlock::text(text));
        Ok(())
    }

    /// Appends a line of text (captions of images).
    pub fn push_text(&mut self, text: impl Into<String>) {
        self.blocks.push(ContentBlock::text(text));
    }

    /// Appends a JPEG picture.
    pub fn push_jpeg(&mut self, jpeg: &[u8]) {
        let data = base64::engine::general_purpose::STANDARD.encode(jpeg);
        self.blocks.push(ContentBlock::image(data, "image/jpeg"));
    }

    /// The content blocks, in order.
    #[must_use]
    pub fn blocks(&self) -> &[ContentBlock] {
        &self.blocks
    }

    /// The MCP result.
    #[must_use]
    pub fn into_result(self) -> CallToolResult {
        CallToolResult::success(self.blocks)
    }
}

/// A failed tool call, returned to the agent as an error result it can act on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolError {
    /// Stable code (`ITEM_NOT_FOUND`, `RENDERER_UNAVAILABLE`…).
    pub code: String,
    /// What is wrong and what to do instead.
    pub message: String,
}

impl ToolError {
    /// An error with `code` and `message`.
    #[must_use]
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
        }
    }

    /// The arguments are wrong.
    #[must_use]
    pub fn invalid_arguments(message: impl Into<String>) -> Self {
        Self::new(codes::INVALID_ARGUMENTS, message)
    }

    /// The MCP result: `{"error":{"code":…,"message":…}}`, flagged as an error.
    #[must_use]
    pub fn into_result(self) -> CallToolResult {
        let text = serde_json::json!({ "error": self }).to_string();
        CallToolResult::error(vec![ContentBlock::text(text)])
    }
}

impl From<AppError> for ToolError {
    fn from(error: AppError) -> Self {
        let protocol = ProtocolError::from(error);
        let code = serde_json::to_value(protocol.code)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| codes::INTERNAL.to_owned());
        Self {
            code,
            message: protocol.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use se_domain::ProjectId;

    use super::*;

    #[test]
    fn app_errors_keep_their_protocol_code() {
        let error = ToolError::from(AppError::ProjectNotOpen {
            project_id: ProjectId::from_test_index(1),
        });
        assert_eq!(error.code, "PROJECT_NOT_OPEN");
        assert!(error.message.contains("open"));
    }

    #[test]
    fn errors_become_flagged_json_results() {
        let result = ToolError::new(codes::ITEM_NOT_FOUND, "no item x").into_result();
        assert_eq!(result.is_error, Some(true));
        let text = serde_json::to_value(&result.content).unwrap().to_string();
        assert!(text.contains("ITEM_NOT_FOUND"), "{text}");
    }

    #[test]
    fn outputs_hold_json_then_images() {
        let mut output = ToolOutput::json(&serde_json::json!({ "a": 1 })).unwrap();
        output.push_jpeg(&[0xFF, 0xD8]);
        let blocks = serde_json::to_value(output.blocks()).unwrap();
        assert_eq!(blocks[0]["text"], r#"{"a":1}"#);
        assert_eq!(blocks[1]["type"], "image");
        assert_eq!(blocks[1]["mimeType"], "image/jpeg");
        assert_eq!(blocks[1]["data"], "/9g=");
    }
}
