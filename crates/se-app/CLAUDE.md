# se-app

## Purpose
The application layer: use cases, ports (traits implemented by adapters), the `ProjectActor` (single writer of a project), and later the `JobQueue` (prompt 08) and the `TaskScheduler` (prompt 17). It orchestrates; it never performs concrete IO.

## Public API
| Module | Contents |
|---|---|
| `ports` | `ProjectRepository` (`locate`, `create`, `duplicate`, `open` → `Box<dyn ProjectStore>`, `trash`), `ProjectStore` (one open, locked folder: `load_snapshot`, `read_journal_since`, `append_journal`, `save_snapshot`, `close`), `JournalEntry`, `AppLibraryRepository` (recent projects, settings) + `RecentProjectRecord`, `EventSink`, `Clock` (wall clock, Unix ms), `IdGenerator` (UUIDv7), `StorageError` |
| `project_actor` | `ProjectActorHandle::spawn(project, store, ActorDeps)`, `apply`, `undo`, `redo`, `view` (→ `ProjectView { project: Arc<Project>, history }`), `close`; `DEFAULT_SAVE_DELAY` (2 s) |
| `project_service` | `ProjectService::new(ProjectServiceDeps, ProjectServiceConfig)`: `new_project_defaults`, `create_project`, `open_project`, `close_project`, `close_all`, `project_session`, `list_recent`, `rename_project`, `duplicate_project`, `trash_project`, `create_sequence`, `undo`, `redo` |
| `project_view` | `project_snapshot(&Project)` (normalised entities), `project_patch(before, after, touched)`, `history_state(&History)` |
| `new_project` | `new_project(ids, name, format)` and `default_sequence(ids, name, format)` (tracks V1, V2, A1, A2) |
| `recovery` | `replay(snapshot, journal)`: crash recovery with `se_domain::commit` |
| `new_ids` | `NewIds(&dyn IdGenerator)`: typed ids (`project()`, `sequence()`, `track()`, `transaction()`) |
| `AppError` | Use-case errors; `code()` → `se_protocol::ErrorCode`; `From<AppError> for ProtocolError` (domain errors carry `details.domainCode`) |
| `test_support` | feature `test-support`: `InMemoryProjectRepository` (shared folders, failure injection, `folder(path)` to inspect), `InMemoryAppLibrary`, `RecordingEventSink`, `FixedClock`, `SequentialIds` |

## Rules and invariants
- No `std::fs`, `std::process` or network access: IO only through ports.
- Only `ProjectActor` mutates a `Project`, by committing `Transaction`s. The service never edits a document: it sends transactions (a temporary actor, events discarded, for closed projects). Building a new document (create, duplicate) is not a mutation.
- Commit flow (ADR-0017): `commit` on a draft → `append_journal` → install, record history → publish `projectPatched` then `historyChanged` → schedule the snapshot. A journal failure must leave state and history untouched (undo/redo restore the cloned history).
- Delays use tokio's timer (testable with `start_paused`); timestamps stored with data use the `Clock` port.
- The registry mutex (`ProjectService::open`) is held for the whole of open/close/rename/duplicate/trash; edits only clone a handle.
- Labels of transactions built here (`Create sequence`, `Rename project`) are stable identifiers translated by the renderer (`apps/desktop/src/app/editor/history-label.ts`): change both sides together.
- Every port has an in-memory fake behind the `test-support` feature so use cases are tested without IO.
- CPU-heavy work goes to `spawn_blocking`.

## Allowed dependencies
`se-domain`, `se-protocol`, `tokio`, `async-trait`, `tracing`, `thiserror`. Never an adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-app`
- Unit tests in-file (patch building, recovery, new projects, error codes).
- `tests/project_actor.rs`: actor against the in-memory store with `#[tokio::test(start_paused = true)]` (commit order, events, journal failure, debounced and retried snapshots, close).
- `tests/project_service.rs`: use cases against the fakes (create, open with replay, locks, copies, close, rename open/closed, duplicate, trash, missing folders, undo/redo).
- Integration tests reach the fakes through the self dev-dependency with `test-support`.
