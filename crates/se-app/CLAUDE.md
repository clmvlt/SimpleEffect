# se-app

## Purpose
The application layer: use cases, ports (traits implemented by adapters), the `ProjectActor` (single writer of a project), the `JobQueue` and the `TaskScheduler`. It orchestrates; it never performs concrete IO.

Currently an empty skeleton: ports and the actor arrive with prompts 06–07, the job queue with 08, the task scheduler with 17.

## Public API
None yet. Planned ports (`docs/architecture.md` §3): `ProjectRepository`, `AppLibraryRepository`, `MediaProber`, `MediaJobRunner`, `Transcriber`, `SubjectDetector`, `AgentProvider`, `Exporter`, `EventSink`, `Clock`, `IdGenerator`.

## Rules and invariants
- No `std::fs`, `std::process` or network access: IO only through ports.
- Only `ProjectActor` mutates a `Project`, by applying `Operation`s inside `Transaction`s.
- Every port has an in-memory fake behind the `test-support` feature so use cases are tested without IO.
- CPU-heavy work goes to `spawn_blocking`.

## Allowed dependencies
`se-domain`, `se-protocol`, `tokio`, `async-trait`, `tracing`, `thiserror`. Never an adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-app`
- Use cases are tested against the `test-support` fakes; integration tests in `tests/`.
