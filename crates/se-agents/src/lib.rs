//! # se-agents
//!
//! AI CLI providers and their stream parsing (ADR-0003, ADR-0027). Claude Code is the first
//! provider: [`providers::claude_code::ClaudeCodeProvider`] implements se-app's `AgentProvider`
//! port. [`FsAgentWorkspaces`] implements `AgentWorkspaces` (the empty working folder of each
//! run).
//!
//! This is the **only** crate that knows which AI CLI is used: provider-specific code lives in
//! `src/providers/<provider>/`, and everything else sees the `AgentProvider` port and
//! `AgentEvent`. CLI processes are spawned only through [`process::CliProcess`] (isolated, whole
//! tree killed on cancel, no console window).
//!
//! Allowed workspace dependencies: `se-app`, `se-domain`, `se-protocol` (DTOs); never another
//! adapter crate.

mod platform;
pub mod process;
pub mod providers;
mod workspaces;

pub use workspaces::FsAgentWorkspaces;
