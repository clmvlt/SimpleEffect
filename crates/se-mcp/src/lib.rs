//! # se-mcp
//!
//! The MCP server of AI agents (ADR-0003, ADR-0028): their only way to read and change a
//! project. se-core mounts [`mcp_service`] at `/mcp`; each request carries the bearer token of
//! one running task (`se_app::TaskTokenRegistry`), which decides who is calling and what it may
//! change.
//!
//! One tool per file in [`tools`]; tools call se-app use cases and never mutate documents
//! directly: write tools ([`writes`]) hand typed edits to `se_app::AgentEditService`, which checks
//! the task's scope and commits them as one AI transaction recorded in the task's changeset. Times cross the boundary as seconds or timecodes ([`time_io`]); outputs are compact
//! JSON (and JPEG images). The catalogue is documented in `docs/ai-tools.md`.
//!
//! Allowed workspace dependencies: `se-app`, `se-domain`, `se-protocol` (DTOs read through
//! se-app); see `docs/architecture.md` §3 and §9.4.

mod analyses;
pub mod context;
pub mod cuts;
pub mod describe;
pub mod output;
pub mod registry;
pub mod server;
pub mod time_io;
pub mod timing;
pub mod tools;
pub mod write_inputs;
pub mod writes;

pub use context::{McpDeps, ToolContext};
pub use output::{ToolError, ToolOutput};
pub use server::{McpServer, SERVER_NAME, bearer_token, mcp_service};
