# se-store

## Purpose
Persistence adapter: project folders, `project.json` snapshots, the project and app-library SQLite databases (`docs/adr/0006-project-storage-layout.md`). It implements the repository ports of `se-app`; it does not decide *when* to save (the actor does).

Currently an empty skeleton, implemented by prompt 07.

## Public API
None yet. Planned: implementations of `ProjectRepository` and `AppLibraryRepository`.

## Rules and invariants
- Writes are atomic (write to a temporary file, then rename) so a crash never corrupts a project.
- Paths are built with `Path`/`PathBuf` APIs only; store absolute media paths plus a project-relative hint.
- Blocking filesystem and SQLite calls run in `spawn_blocking`.

## Allowed dependencies
`se-app`, `se-domain` and storage libraries. Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-store`
- Integration tests in `tests/` against temporary directories.
