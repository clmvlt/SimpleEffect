# se-export

## Purpose
Export adapter: SRT subtitles (prompt 23), later FCP7 XML and OTIO timelines (Premiere Pro) (`docs/adr/0008-premiere-export.md`, ADR-0033). Video exports (prompt 27, ADR-0038) are not here: se-app's `exports` module orchestrates them (queue, snapshot, progress, checks), se-media writes their files (`FsExportOutputs`, ffmpeg for masters and AAC) and `@se/engine/export` draws and encodes them in the renderer (WYSIWYG).

## Public API
| Item | Role |
|---|---|
| `write_srt(&[SubtitleCue]) -> String` | Pure writer: UTF-8 with a byte-order mark, CRLF, cues numbered from 1, `HH:MM:SS,mmm` to the nearest millisecond (an end at least 1 ms after its start) |
| `parse_srt(&str) -> Result<Vec<SubtitleCue>, SrtError>` | Tolerant reader: BOM, LF/CRLF/CR, `.` or `,` before milliseconds (1–3 digits), missing or wrong cue numbers, cue settings after the timing, extra blank lines; markup dropped (`<i>`, `<font …>`, `{\an8}`). `SrtError { line, message }` |
| `FsSubtitleFiles` | se-app's `SubtitleFiles` port on the file system: blocking IO on tokio's blocking pool, atomic writes (temporary file then rename), UTF-8 read with a Latin-1 fallback |

TODO(prompt-28): FCP7 XML and OTIO writers (the Premiere package reuses `write_srt` with `se_app::captions::subtitles::cues_of`).

## Rules and invariants
- Writers are pure functions from data to text.
- `Flicks` are converted to milliseconds, frame counts or timecodes only here, with the dedicated helpers.
- Error messages name the file or the line and what was expected.

## Allowed dependencies
`se-app`, `se-domain`, `async-trait`, `thiserror`, `tokio` (rt, for `spawn_blocking`). Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-export`
- `srt.rs`: exact written text, write/read round trip, loose files, rounding, errors with line numbers; `subtitle_files.rs`: real temporary files (write then read, missing file, Latin-1, invalid content).
