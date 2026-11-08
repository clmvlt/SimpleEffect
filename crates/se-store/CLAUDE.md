# se-store

## Purpose
Persistence adapter: project folders, `project.json` snapshots, the project and app-library SQLite databases (`docs/adr/0006-project-storage-layout.md`, ADR-0017). It implements the repository ports of `se-app`; it does not decide *when* to save (the actor does).

## Public API
- `FsProjectRepository` (implements `ProjectRepository`): `locate` (canonical path via `dunce`, needs a `project.json`), `create` (`<parent>/<name>.seproj`, refuses an existing folder), `duplicate` (numbered folder next to the source, `VACUUM INTO` copy of `project.db` without journal), `open` (takes the OS lock, opens `project.db`), `trash` (OS trash).
- `FsProjectStore` (implements `ProjectStore`, returned by `open`): snapshot read (migrated) and atomic write, journal append/read/truncate, `append_journal_with_changeset` (journal row + changeset change in one SQLite transaction), `load_changesets` (open ones), `analyses()` (an `FsAnalysisStore` sharing its connection), `tasks()` (an `FsTaskStore` sharing it too), `transcripts()` (an `FsTranscriptStore`, prompt 21), `close` releases the lock.
- `SqliteAppLibrary::open(path)` (implements `AppLibraryRepository`): `library.db` with `recent_projects` and `settings`.
- `ProjectLayout` (paths inside a folder), `ProjectDatabase` (direct access, tests and diagnostics), `BACKUP_COUNT`, `PROJECT_EXTENSION`.

## Layout
| File | Role |
|---|---|
| `layout.rs` | `ProjectLayout`: `project.json`, `.tmp`, `.bak.N`, `project.db`, `.lock`, `cache/`, `exports/` |
| `folder_name.rs` | Folder names from project names (Windows-forbidden characters → `-`, reserved names, length) |
| `snapshot_file.rs` | `read_snapshot` (JSON → migrations → `Project`), `write_snapshot` (backup rotation + `write_atomically`) |
| `document_migrations.rs` | `MIGRATIONS` (`migrate_vN_to_vN+1`; v1 → v2 gives assets `fingerprint: null` and a per-stream `decodability`; v2 → v3 gives them empty `analysis` summaries; v3 → v4 gives tracks empty `transitions` and media clips linear fade shapes; v4 → v5 wraps the bare font size and colour of texts and the fill of shapes into static animatable values; v5 → v6 gives media clips that fill the frame a static focus on the centre; v6 → v7 gives the project its transcription settings (`fr`, no vocabulary) and assets an empty `transcript` summary; v7 → v8 turns each caption's `text` into words spread equally over its item with a `minimal` caption style (`MIGRATED_CAPTION_STYLE_ID`, only when the document holds captions) and gives the project `captionStyles`), `migrate_document` — the recipe for a new version is in its doc comment |
| `sqlite.rs` | `open_database(path, migrations)`: WAL, `synchronous = FULL`, foreign keys, busy timeout, `user_version` migrations |
| `project_db.rs`, `project_db/analyses.rs` | `project.db` schemas (v1: `op_journal`, `tasks`, `task_events`, `transcripts`, `transcript_search` FTS5, `analyses`; v2: `analyses.content_key` and `format_version`; v3: `tasks.format_version`, `task_events.run` and `format_version`; v4: `changesets`, `changeset_entries`; v5: the transcript tables), journal queries, `save_analysis`/`load_analysis` (rows of a newer payload format are ignored) |
| `fs_analysis_store.rs` | `FsAnalysisStore`: the `AnalysisStore` port on the blocking pool |
| `project_db/changesets.rs` | AI changesets (schema v4): `append_with_changeset` (journal row, then `changesets` row created if new and `changeset_entries` row numbered `MAX(sequence) + 1`, or `reverted_at` set — one `unchecked_transaction`), `open_changesets` (not reverted, oldest first, entries in order; entries of a newer `format_version` refused with `SchemaTooNew`) |
| `project_db/tasks.rs`, `fs_task_store.rs` | AI tasks (schema v3): `save_task` (upsert of the `Task` JSON + status kind), `load_tasks` (oldest first), `append_task_event` (numbered with `MAX(sequence) + 1` in the insert, `RETURNING`), `load_task_events`; rows of a newer `format_version` are skipped; `FsTaskStore` is the `TaskStore` port on the blocking pool |
| `project_db/transcripts.rs`, `fs_transcript_store.rs` | Transcripts (schema v5, prompt 21, ADR-0031): `transcripts` (asset, content key, language, model, backend, created, format version), `transcript_words`, `transcript_segments` (cascading), FTS5 `transcript_search` (`unicode61 remove_diacritics 2`); `save_transcript` replaces an asset's rows and index entries in one `unchecked_transaction`, `load_transcript` (newer format → absent; broken invariants → `Corrupted`), `search_transcripts` (every word quoted, the last as a prefix, `bm25` order); `FsTranscriptStore` is the `TranscriptStore` port on the blocking pool |
| `library.rs` | `SqliteAppLibrary` |
| `project_lock.rs` | `ProjectLock`: `File::try_lock` on `.lock` |
| `fs_repository.rs`, `fs_store.rs` | The port implementations |
| `platform.rs` | OS specifics: directory fsync (Unix), trash method (NSFileManager on macOS) |
| `io_errors.rs` | `StorageError` mapping, `blocking()` (spawn_blocking), UTF-8 paths |

## Rules and invariants
- Writes are atomic: `project.json` is written to `project.json.tmp`, fsynced, renamed, and the folder fsynced; the previous file is copied (not renamed) to `.bak.1` first, so `project.json` exists at every instant. Three backups are kept.
- `project.json` is written last when a folder is created or duplicated, so a folder without it is never a project; a half-made folder is removed (only ever the one this call just created).
- The `.lock` file is never deleted; the OS lock is what matters (released when the process dies).
- SQLite schemas only grow: append a migration, never edit a released one. Journal payloads carry `format_version` (bump it and migrate old rows if the `Transaction` JSON changes).
- Paths are built with `Path`/`PathBuf` APIs only; paths stored as text must be UTF-8 (`utf8_path`).
- Blocking filesystem and SQLite calls run in `spawn_blocking` (`io_errors::blocking`).
- Never delete user data: trashing uses the OS trash.

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (the task DTOs it stores as JSON) and storage libraries (`rusqlite` bundled, `serde_json`, `dunce`, `trash`, `tokio` for `spawn_blocking`). Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-store`
- Unit tests in-file (folder names, migrations runner, SQLite migrations).
- `tests/project_folders.rs` (temporary folders): layout, atomic writes and backups, journal survives a dropped store, lock, version fixtures (`tests/fixtures/project-v1.json` to `project-v7.json` migrated, `project-v8.json` as is: the body of the domain's example-project snapshot), newer/broken documents, duplication, FTS5. `tests/app_library.rs`: recent list and settings. `tests/project_tasks.rs`: tasks and their event logs survive a dropped store and a reopening. `tests/project_changesets.rs`: changesets survive a crash with their journal entries and close when reverted; a failed changeset write leaves no journal entry. `tests/project_transcripts.rs`: transcripts survive a reopening, search ignores accents and case, a new transcription replaces rows and index.
- `trashing_moves_the_folder_to_the_os_trash` is `#[ignore]`d because it uses the real OS trash: `cargo test -p se-store -- --ignored`.
