//! # se-app
//!
//! Application layer of SimpleEffect: use cases, ports (traits implemented by adapters),
//! the `ProjectActor` (single writer of a project), the `JobQueue` and the `TaskScheduler`.
//!
//! Allowed workspace dependencies: `se-domain`, `se-protocol` (see `docs/architecture.md` §3).
//! Adapters (`se-store`, `se-media`, `se-transcribe`, `se-vision`, `se-agents`, `se-export`, `se-mcp`)
//! depend on this crate, never the other way round.
//!
//! ## Rules
//! - No concrete IO: no `std::fs`, `std::process` or network access. IO lives behind ports.
//! - Every port gets an in-memory fake behind the `test-support` feature.
//!
//! Ports and the actor arrive with roadmap prompts 06 and 07.
