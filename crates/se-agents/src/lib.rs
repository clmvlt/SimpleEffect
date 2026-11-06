//! # se-agents
//!
//! AI CLI providers and their stream parsing. Claude Code is the first provider
//! (see `docs/adr/0003-ai-agents-via-cli-and-local-mcp.md`).
//!
//! This is the **only** crate that knows which AI CLI is used: provider-specific code lives in
//! `src/providers/<provider>/`, and everything else sees the `AgentProvider` port and `AgentEvent`.
//! Allowed workspace dependencies: `se-app`, `se-domain`; never another adapter crate.
//!
//! Implemented by roadmap prompt 17.
