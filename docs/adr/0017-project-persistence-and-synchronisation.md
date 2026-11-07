# 0017 — Project persistence, locking and UI synchronisation

- Status: Accepted
- Date: 2026-11-07

## Context
ADR-0005 (single writer) and ADR-0006 (`.seproj` layout) fixed the principles; prompt 07 implemented them across `se-app`, `se-store`, `se-core` and the renderer and had to settle details they leave open: how a port exposes an open, locked project, when a commit becomes visible, how stale locks are avoided after a crash, what the renderer receives and how it recovers from a missed patch or a core restart, and how RPC requests are scheduled now that handlers are asynchronous.

## Decision
- **Opening is locking.** `ProjectRepository` handles folders as wholes (`locate`, `create`, `duplicate`, `trash`); `open(path)` takes the project's lock and returns a `ProjectStore` (snapshot, journal, close) owned by the project's actor. The prompt's `load / save_snapshot / append_journal / read_journal_since` live on the store, so no code can write a project it has not locked.
- **The lock is an OS file lock** (`File::try_lock`, exclusive) on `<project>/.lock`. The OS releases it when the process dies, so a crash never leaves a project locked; the file itself is never deleted (deleting it would let two processes lock two different files of the same name). Inside one core, opening an already open project returns its session (renderer reloads, reconnections); another process gets `PROJECT_LOCKED`. A second folder holding the same project id (a copy made in the file manager) is refused with `PROJECT_LOCKED` too.
- **Journal before visibility.** The actor commits on a draft, appends the transaction to `op_journal` (SQLite WAL, `synchronous = FULL`), and only then installs the new state, records history and publishes `projectPatched` then `historyChanged`. A failed append leaves state and history unchanged and returns `STORAGE_FAILED`. The snapshot is written `DEFAULT_SAVE_DELAY` (2 s) after the last commit — a tokio timer, so tests drive it with a paused clock — and on close; a successful snapshot truncates the journal. Opening replays journal entries newer than the snapshot with `se_domain::commit` (deterministic) and saves the recovered snapshot at once.
- **`project.json` writes are atomic and keep 3 backups**: the current file is copied to `.bak.1` (older backups shift), then the new JSON is written to `project.json.tmp`, fsynced, renamed over `project.json`, and the folder fsynced (Unix). Documents carry `schemaVersion`; ordered pure `migrate_vN_to_vN+1(Value)` functions upgrade them; a newer version is refused (`SCHEMA_TOO_NEW`). SQLite schemas use `PRAGMA user_version` and append-only SQL migrations; journal payloads carry a `format_version`.
- **The renderer receives normalised whole entities.** A `ProjectSnapshot` lists sequences, tracks, items, markers and assets by id; parents list their children's ids (`trackIds`, `itemIds`…), children point back (`sequenceId`, `trackId`). A `ProjectPatch { projectId, revision, upserts, deletes }` replaces entities by id; the patch builder refreshes parents with their children (a moved item refreshes its old and new tracks). The renderer's `shared/project-store` (Zustand) applies the next revision only; a gap, `resyncRequired` or a reconnection reloads the snapshot (`getProjectSnapshot`), and `PROJECT_NOT_OPEN` (the core restarted) reopens the project from its folder. Features only read the store.
- **Requests run concurrently.** `se-core` runs each request in its own task, so a slow command never delays events or other requests; a client that needs ordering waits for the previous answer. Events are still forwarded before responses (`biased` select), so the patch of a command usually arrives before its answer.
- **Undo history lives in memory**, per open project (ADR-0016); it is lost when the project closes. Labels of transactions built by the core (`Create sequence`, `Rename project`) are stable identifiers translated by the renderer.
- **Operations on closed projects go through a temporary actor** (rename) or a temporary locked store read (duplicate), whose events are discarded: no document is written outside an actor.
- **App data**: `library.db` (recent projects, settings) lives in the core's data folder (`SE_DATA_DIR`, Electron's `userData`) rather than a `dirs` location, so app and core share one folder; new projects are proposed in `<Documents>/SimpleEffect` until the user picks another folder (remembered as a setting).

## Consequences
- Killing the core right after an edit loses nothing (process test `a_commit_survives_killing_the_core`); a stale `.lock` file never blocks reopening.
- Every commit costs one fsynced SQLite insert; snapshots are rare. Saving a large project serialises it on the blocking pool.
- The renderer never re-implements an operation; adding an entity kind means adding it to the snapshot, the patch builder and the mirror.
- History does not survive closing the project; persisting it is a backlog item.
- Duplicates copy `project.db` with `VACUUM INTO` (transcripts and analyses come along), empty the copy's journal and get a new project id.

## Alternatives considered
- **A lock file with the owner's PID**: needs stale-lock detection (PID reuse, other machines on network drives). Rejected for the OS lock, which the OS releases on death.
- **Installing the commit before journaling (journal asynchronously)**: faster, but a crash could show the user an edit that is then lost. Rejected.
- **Sending the document tree (`Project`) and patches as operations**: the renderer would re-implement operations (ADR-0005 forbids it) or re-normalise the tree on every patch. Rejected.
- **Processing a connection's requests one at a time**: preserves order for free, but opening a large project would block undo/redo and every other request of the window. Rejected; clients await dependent requests.
