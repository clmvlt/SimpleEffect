//! # se-store
//!
//! Persistence adapter: project folders, `project.json` snapshots and the SQLite databases
//! (see `docs/adr/0006-project-storage-layout.md`).
//!
//! It implements the repository ports declared in `se-app` (`ProjectRepository`,
//! `AppLibraryRepository`). Allowed workspace dependencies: `se-app`, `se-domain`;
//! it must not depend on another adapter crate (see `docs/architecture.md` §3).
//!
//! Implemented by roadmap prompt 07.
