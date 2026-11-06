//! # se-mcp
//!
//! MCP server exposing SimpleEffect tools to AI agents (the agents' only way to act).
//!
//! One tool per file; write tools call `se-app` use cases and never mutate documents directly.
//! Allowed workspace dependencies: `se-app`, `se-domain` (see `docs/architecture.md` §3 and §9.4).
//! The tool catalog is documented in `docs/ai-tools.md`.
//!
//! Implemented by roadmap prompts 18 and 19.
