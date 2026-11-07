# se-fonts

## Purpose
The font adapter (ADR-0025): implements `se_app::ports::FontCatalog` with the fonts **bundled** in the binary (latin and latin-ext WOFF2 subsets of a few families, `fonts/<family>/` with their licences) and the fonts **installed** on the system (read once with `fontdb`). se-core lists them with `listFonts` and serves each face under `GET /media/fonts/{id}`; the renderer loads them with `FontFace` (`@se/engine/fonts`).

## Public API
- `LocalFontCatalog::new()` (bundled + installed; the installed fonts are read on the blocking pool the first time, cached), `LocalFontCatalog::bundled_only()` (tests, fallback).
- `BUNDLED_FACES` / `BundledFace` (family, path, bytes, weights, italic, unicode range; `id()` = `bundled-<file stem>`).

## Layout
| File | Role |
|---|---|
| `bundled.rs` | The embedded faces (`include_bytes!`), `bundled_families()`, `bundled_face(id)` |
| `system.rs` | `load_system_fonts(skip)`: `fontdb` scan → families (English names, hidden `.` families and bundled names skipped) and the file of each face |
| `face_id.rs` | `system_face_id(path, index)`: `system-` + xxh3 of the path and face index |
| `catalog.rs` | `LocalFontCatalog` (the port) |
| `fonts/<family>/` | WOFF2 files and `LICENSE` (SIL OFL 1.1, Apache 2.0 for Permanent Marker), copied from the Fontsource npm packages |

## Rules and invariants
- Bundled families come first, then installed ones, each sorted by name; a bundled family hides an installed one of the same name.
- Face ids are stable within a run (installed: path + index hashed; bundled: file name). Never log font paths of the user beyond debug level.
- Only redistributable fonts are bundled (OFL, Apache 2.0); keep each family's `LICENSE` next to its files.
- Adding a bundled family: copy its `*-latin-*.woff2` and `*-latin-ext-*.woff2` and `LICENSE` from its Fontsource package into `fonts/<family>/`, add one `face(…)` line per file in `bundled.rs` with the weights, style and `unicode-range` of the package's CSS (`wght.css`, `wght-italic.css` or `index.css`).
- Reading installed fonts is blocking: only on the blocking pool (`spawn_blocking`).

## Allowed dependencies
`se-app` (the port), `se-protocol` (its DTOs), `fontdb`, `tokio` (blocking pool, `OnceCell`), `async-trait`, `tracing`, `xxhash-rust`. Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-fonts`: every bundled face is WOFF2 with a unique id; Inter has upright and italic faces; installed families follow the bundled ones and every installed face has a file (reads this machine's fonts).
