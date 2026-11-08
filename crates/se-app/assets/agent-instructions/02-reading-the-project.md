# Reading the project

Start broad, then zoom in. Read-only tools change nothing: call them as often as you need.

1. `get_project_overview`: sequences (format, duration), tracks, media count, your scope. Call it
   first when you do not know the project.
2. `get_timeline`: the items of a sequence that intersect a range (your scope's range by default),
   track by track, with their key properties. Large timelines are paginated: follow `next_offset`.
3. `get_item`: everything about one item (style, transform, keyframes, origin).
4. `list_assets`: the media of the project (kind, duration, size, analysis summaries).
5. `get_audio_analysis` and `get_scene_changes`: silences, loudness, peaks and shot changes, for a
   range of the sequence (in sequence time) or for an asset (in source time). Prefer these over
   guessing from pictures for anything about sound or cuts.
6. `get_frames`: **look at the sequence** as the viewer shows it (texts, transformations and
   animations included), at given times or spread over a range. Use it whenever the answer depends
   on what is on screen; a few frames (4–8) are usually enough, at the default width.
7. `get_source_frames`: pictures of a media file itself, at source times (what a clip could show
   beyond its edges).
8. `get_transcript`, `list_animation_presets`, `get_brand_kit`, `list_templates`: transcripts,
   animation presets, brand kit and motion templates (some are not available yet and say so).

Describe what you saw precisely (who or what, where, when). If a tool fails, read its error: it
says what to change (an id, a time out of range, the editor window to open).
