# 0006 — Project storage layout

- Status: Accepted
- Date: 2026-11-06

## Context
Projects must survive crashes, stay inspectable by humans and AI developers, store large structured data (operation journal, AI task logs, word-level transcripts) and regenerable binary caches.

## Decision
A project is a folder `<Name>.seproj/`:
- `project.json` — canonical document, pretty-printed, with `schema_version` and `revision`; written atomically (temp file + fsync + rename) on a debounce and on close.
- `project.db` — SQLite in WAL mode: `op_journal` (transactions since the last snapshot), `tasks`, `task_events`, `transcripts` (FTS5 full-text search), `analyses`.
- `cache/` — regenerable derived files (mezzanines, proxies, thumbnails, waveform peaks, overlay renders), keyed by content hash.
- `exports/` — default export destination.

Media files are referenced by absolute path (plus a relative hint), never copied by default; a "collect media" action and a relink flow exist.
Application-level data lives in the OS app data dir: `library.db` (recent projects, settings, provider config), `templates/`, `brand-kits/`, `models/`, `logs/`.
Schema evolution uses ordered pure migration functions on JSON values, covered by fixture tests.

## Consequences
- Crash recovery = load snapshot + replay journal entries with a higher revision.
- The document is human/AI-readable; heavy data stays queryable.
- Two storage technologies to maintain, each behind the `ProjectRepository` port.

## Alternatives considered
- **Everything in SQLite**: robust but the document becomes opaque to humans and AI tools. Rejected.
- **Everything in JSON/JSONL**: simple, but no full-text search and weaker crash safety for logs. Rejected.
- **Copying media into the project**: wastes disk space with large camera files. Offered as an explicit action only.
