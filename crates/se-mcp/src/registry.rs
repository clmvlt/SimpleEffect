//! The tool contract ([`McpTool`]) and its type-erased form ([`ToolEntry`]) the server lists and
//! calls. Each tool lives in its own file under `src/tools/` and implements [`McpTool`].

use std::{future::Future, pin::Pin, sync::Arc};

use rmcp::{
    handler::server::common::schema_for_input,
    model::{JsonObject, Tool, ToolAnnotations},
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
};

/// One MCP tool: its name, description, input type (its JSON Schema is generated) and handler.
pub trait McpTool: Send + Sync + 'static {
    /// The input, read from the call's arguments (unknown fields refused).
    type Input: DeserializeOwned + JsonSchema + Send + 'static;

    /// The tool's name (`get_timeline`): stable, `snake_case`.
    const NAME: &'static str;

    /// What it does and when to use it, for the agent.
    const DESCRIPTION: &'static str;

    /// Whether the tool only reads (`false` for write tools).
    const READ_ONLY: bool = true;

    /// Whether the tool may remove or overwrite what is there (write tools); `false` for tools
    /// that only add facts next to the project, like `transcribe_media`.
    const DESTRUCTIVE: bool = !Self::READ_ONLY;

    /// Runs the tool.
    fn call(
        context: ToolContext,
        input: Self::Input,
    ) -> impl Future<Output = Result<ToolOutput, ToolError>> + Send;
}

type ToolFuture = Pin<Box<dyn Future<Output = Result<ToolOutput, ToolError>> + Send>>;
type Handler = Arc<dyn Fn(ToolContext, JsonObject) -> ToolFuture + Send + Sync>;

/// A tool ready to be listed and called.
#[derive(Clone)]
pub struct ToolEntry {
    /// The definition sent to agents (`tools/list`).
    pub definition: Tool,
    handler: Handler,
}

impl ToolEntry {
    /// The entry of tool `T`.
    ///
    /// # Errors
    /// The schema problem when `T::Input` is not an object schema (a bug caught by tests).
    pub fn of<T: McpTool>() -> Result<Self, String> {
        let schema = schema_for_input::<T::Input>()?;
        let mut definition = Tool::new(T::NAME, T::DESCRIPTION, schema);
        definition.annotations = Some(
            ToolAnnotations::new()
                .read_only(T::READ_ONLY)
                .destructive(T::DESTRUCTIVE)
                .open_world(false),
        );
        let handler: Handler = Arc::new(|context, arguments| {
            Box::pin(async move {
                let input = parse::<T::Input>(arguments)?;
                T::call(context, input).await
            })
        });
        Ok(Self {
            definition,
            handler,
        })
    }

    /// The tool's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// Runs the tool with the call's raw arguments.
    ///
    /// # Errors
    /// [`ToolError`] for bad arguments or a failed call.
    pub async fn call(
        &self,
        context: ToolContext,
        arguments: JsonObject,
    ) -> Result<ToolOutput, ToolError> {
        (self.handler)(context, arguments).await
    }
}

/// Reads the arguments into `T`, with serde's message naming the faulty field.
pub(crate) fn parse<T: DeserializeOwned>(arguments: JsonObject) -> Result<T, ToolError> {
    serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|error| {
        ToolError::invalid_arguments(format!(
            "{error}; check the tool's input schema (field names are snake_case)"
        ))
    })
}
