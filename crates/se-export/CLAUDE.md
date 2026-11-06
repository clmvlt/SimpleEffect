# se-export

## Purpose
Export adapter: FCP7 XML and OTIO timelines (Premiere Pro), SRT subtitles, and orchestration of final renders (`docs/adr/0008-premiere-export.md`). It implements the `Exporter` port; pixel rendering itself happens in `@se/engine` (WYSIWYG).

Currently an empty skeleton, implemented by prompts 27 and 28.

## Public API
None yet. Planned: `Exporter` implementations and pure writer functions.

## Rules and invariants
- Writers are pure functions from the document model to bytes.
- `Flicks` are converted to frame counts/timecodes only here, with the dedicated helpers.

## Allowed dependencies
`se-app`, `se-domain`, serialisation libraries. Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-export`
- `insta` snapshot tests for every writer.
