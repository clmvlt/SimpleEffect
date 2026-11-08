# AI tools — the MCP contract with agents

> The tools AI agents use to read (and, from prompt 19, change) a SimpleEffect project. This file
> is the **contract**: agents' behaviour, `crates/se-app/assets/agent-instructions/` and the tool
> definitions in `crates/se-mcp/src/tools/` must agree with it. Decisions: ADR-0003, ADR-0028.
> Schema snapshots: `crates/se-mcp/tests/snapshots/` (a test fails when a tool has no section here).

## Endpoint and access

- se-core serves MCP (streamable HTTP, `rmcp`, sessions in memory) at `http://127.0.0.1:<port>/mcp`.
- Every request carries `Authorization: Bearer <task token>`. The token is minted when a run of an
  AI task starts and revoked when it ends (`se_app::TaskTokenRegistry`); it stands for an
  `AgentContext { project_id, task_id, run, scope }`. Unknown or revoked tokens get **401** before
  any MCP message is read; a `Host` other than `127.0.0.1:<port>` / `localhost:<port>` gets **403**.
  A run that ends between the HTTP check and the tool call gets `ACCESS_REVOKED`.
- The server is named `simpleeffect`: Claude Code names its tools `mcp__simpleeffect__<tool>` and
  is started with `--allowedTools "mcp__simpleeffect__*"`, no built-in tool.
- Tokens are never logged; tool calls are logged with the tool name, task id, duration and error
  code.

## Conventions

- **One tool per file** in `crates/se-mcp/src/tools/`; input schema generated with `schemars`
  (draft 2020-12); unknown argument fields are refused (`INVALID_ARGUMENTS` naming the field).
- **Names** of tools, arguments and output fields are `snake_case`. The one exception is
  `get_item`'s `properties`: the item's document form, with the document's camelCase fields.
- **Times in**: a number of seconds (`51.5`) or a string — `"51.5s"`, `"00:51"`, `"1:02.500"`,
  `"00:00:51.500"`, or `"00:00:51:12"` (frames at the sequence's rate; the asset's rate for source
  times). A bare number in a string (`"51"`) is refused as ambiguous.
- **Times out**: `{"timecode": "HH:MM:SS.mmm", "seconds": 51.5}` (seconds rounded to the
  millisecond). Ranges: `{"start": …, "end": …}`, end excluded. Flicks never cross the boundary.
- **Time bases**: sequence time starts at the sequence's first frame; source time at the media
  file's first frame. Tools that can use either say which (`time_base`).
- **Defaults**: tools taking a sequence default to the task's sequence; tools taking a range
  default to the task's range on that sequence, else the whole sequence (to the end of its last
  item).
- **Outputs** are one compact JSON text block (images follow as MCP image blocks, `image/jpeg`,
  each preceded by a caption line). They carry ids and revisions (`revision` of items, the
  project's `revision`) for the optimistic concurrency of write tools (prompt 19).
- **Pagination**: long lists take `offset` / `limit` and answer `total` (or `total_items`) and
  `next_offset` (`null` on the last page). Other lists are capped (and say their total).
- **Errors** are tool results flagged `isError`, text `{"error":{"code":"…","message":"…"}}`. The
  message says what is wrong and what to do. Codes are stable:

| Code | Meaning |
|---|---|
| `INVALID_ARGUMENTS` | Arguments do not match the schema (unknown field, wrong type, incompatible options) |
| `INVALID_TIME` | A time cannot be read, a range ends before it starts, or a time lies outside the sequence |
| `SEQUENCE_NOT_FOUND` / `TRACK_NOT_FOUND` / `ITEM_NOT_FOUND` / `ASSET_NOT_FOUND` | No such entity; the message lists what exists or which tool lists ids |
| `NOT_ANALYSED` | The asset's sound or pictures are not analysed (yet, or it has none) |
| `TOO_MANY_FRAMES` | More frames asked than one call returns |
| `ACCESS_REVOKED` | The task's run is over |
| `RENDERER_UNAVAILABLE` | No editor window is open, or it did not draw the frames within 30 s |
| `RENDER_FAILED` | The editor window could not draw the frames (its message says why) |
| `PROJECT_NOT_OPEN` | The user closed the project |
| `MEDIA_NOT_FOUND`, `MEDIA_TIME_OUT_OF_RANGE`, `MEDIA_TOOLS_MISSING`, `DERIVED_NOT_APPLICABLE` | Source frame extraction failed (missing file, time outside the media, no ffmpeg) |
| `INTERNAL` | A bug: report it |

- **Read tools change nothing** (annotation `readOnlyHint: true`).

## Catalogue

| Tool | Purpose |
|---|---|
| `get_project_overview` | Sequences, tracks of the task's sequence, media counts, scope |
| `get_timeline` | Items intersecting a range, in time order, with transitions and markers (paginated) |
| `get_item` | Everything about one item (style, transform, keyframes, fades, reframing, origin) |
| `list_assets` | Media of the project with analysis summaries (filtered, paginated) |
| `get_frames` | Composited pictures of the sequence, drawn by the editor window |
| `get_source_frames` | Pictures of a media file at source times (ffmpeg) |
| `get_audio_analysis` | Silences, loudness, peaks — of an asset or of a sequence range |
| `get_scene_changes` | Shot changes — of an asset or of a sequence range, plus the edit's cuts |
| `get_transcript` | Spoken words (not available until prompt 21) |
| `list_animation_presets` | Animation presets (fade, slides, pop…) |
| `get_brand_kit` | Brand kit (not available until prompt 25) |
| `list_templates` | Motion templates (not available until prompt 24) |

Ids below are shortened (`…0001…`); real ones are UUIDs.

### `get_project_overview`

The project at a glance; the first call of an agent that does not know the project.

**Input**: none (`{}`).

**Output**:
```json
{"project":{"project_id":"…","name":"Example","revision":47},
 "scope":{"task_id":"…","sequence_id":"…0001…","sequence_name":"Main edit","range":null,"tracks":null},
 "sequences":[{"sequence_id":"…0001…","name":"Main edit","width":1920,"height":1080,"fps":29.97,
   "duration":{"timecode":"00:00:14.000","seconds":14.0},"track_count":5,"item_count":9,"marker_count":1,"is_task_sequence":true}],
 "tracks":[{"track_id":"…","name":"V1","kind":"video","item_count":2}, …,
   {"track_id":"…","name":"A1","kind":"audio","item_count":1,"muted":true}],
 "assets":{"total":3,"video":1,"audio":1,"image":1,"missing":0},
 "brand_kit":true,"templates":1}
```
`range`/`tracks` of the scope are `null` for the whole sequence / every track. Track flags
`muted`, `hidden`, `locked` appear only when true.

**Errors**: `PROJECT_NOT_OPEN`.

### `get_timeline`

Items of a sequence that intersect a range, in time order (then bottom track first), with the
range's transitions and markers.

**Input**:
| Field | Type | Default |
|---|---|---|
| `sequence` | string (id or exact name) | the task's sequence |
| `range` | `{start, end}` (times) | the task's range, else the whole sequence |
| `tracks` | string[] (ids or names) | every track |
| `offset` | integer | 0 |
| `limit` | integer (1–200) | 50 |

**Output**:
```json
{"sequence":{"sequence_id":"…","name":"Main edit","fps":29.97,"duration":{"timecode":"00:00:14.000","seconds":14.0}},
 "range":{"start":{"timecode":"00:00:00.000","seconds":0.0},"end":{"timecode":"00:00:04.000","seconds":4.0}},
 "tracks":[{"track_id":"…","name":"V1","kind":"video","item_count":2}, …],
 "items":[{"item_id":"…0001…","revision":1,"kind":"media_clip","track_id":"…",
   "start":{"timecode":"00:00:00.000","seconds":0.0},"end":{"timecode":"00:00:10.000","seconds":10.0},
   "label":"interview.mp4","asset_id":"…",
   "source":{"start":{"timecode":"00:00:05.000","seconds":5.0},"end":{"timecode":"00:00:15.000","seconds":15.0}},
   "origin":"user","link_group":"…"},
   {"item_id":"…0009…","kind":"media_clip","label":"music.wav","animated":["volumeDb"], …}],
 "total_items":5,"next_offset":3,
 "transitions":[{"transition_id":"…","track_id":"…","kind":"crossDissolve","left_item_id":"…","right_item_id":"…",
   "at":{"timecode":"00:00:10.000","seconds":10.0},"duration":{"timecode":"00:00:01.000","seconds":1.0}}],
 "markers":[{"marker_id":"…","time":{"timecode":"00:00:03.000","seconds":3.0},"label":"Intro ends"}]}
```
Item `kind`: `media_clip`, `text`, `shape`, `image`, `motion_template`, `caption`. `label`: the
media file name, the text (≤ 80 characters), the shape (`rectangle`, `ellipse`, `line`) or the
template's name. Clips give `source` (source range played) and `speed` when not 1. `animated`
lists properties with keyframes. `origin` is `user` or `ai_task` (+ `task_id`). Transitions are
listed when their cut lies in the range.

**Errors**: `SEQUENCE_NOT_FOUND`, `TRACK_NOT_FOUND`, `INVALID_TIME`, `INVALID_ARGUMENTS`.

### `get_item`

Everything about one item.

**Input**: `item_id` (string, required).

**Output**: `{sequence_id, track_name, item: <summary as in get_timeline>, properties}` where
`properties` is the item's document form (camelCase fields) with times made readable: keyframe
`time` (relative to the item's start, plus `sequence_time`), and a clip's `sourceStart`, `fadeIn`,
`fadeOut`.
```json
{"sequence_id":"…","track_name":"V2",
 "item":{"item_id":"…0003…","kind":"text","label":"Bonjour","start":{"timecode":"00:00:01.000","seconds":1.0}, …},
 "properties":{"kind":"text","content":"Bonjour",
   "style":{"fontFamily":"Inter","fontWeight":600,"fontSize":{"kind":"static","value":72.0},"color":{"kind":"static","value":"#ffffff"},
     "stroke":{"enabled":true,"color":{"kind":"static","value":"#000000"},"width":{"kind":"static","value":4.0}}, …},
   "transform":{"position":{"kind":"static","value":{"x":0.5,"y":0.8}},
     "opacity":{"kind":"keyframed","keyframes":[
       {"time":{"timecode":"00:00:00.000","seconds":0.0,"sequence_time":{"timecode":"00:00:01.000","seconds":1.0}},"value":0.0,"easing":{"kind":"preset","name":"easeOut"}},
       {"time":{"timecode":"00:00:00.500","seconds":0.5,"sequence_time":{"timecode":"00:00:01.500","seconds":1.5}},"value":1.0,"easing":{"kind":"preset","name":"easeOut"}}]}, …},
   "reveal":{"kind":"static","value":1.0}}}
```
Positions and anchors are normalised (0–1, `(0.5, 0.5)` is the centre); sizes are design pixels
(ADR-0015); colours `#rrggbb[aa]`.

**Errors**: `ITEM_NOT_FOUND`.

### `list_assets`

The media files of the project.

**Input**: `kind` (`video` | `audio` | `image`), `query` (text in the file name, case ignored),
`offset` (0), `limit` (1–200, default 50).

**Output**:
```json
{"assets":[{"asset_id":"…","name":"interview.mp4","kind":"video","duration":{"timecode":"00:01:00.000","seconds":60.0},
   "width":1920,"height":1080,"fps":29.97,"audio_channels":2,"used_by":3,
   "integrated_lufs":-16.0,"silence_count":1,"scene_changes":2}],
 "total":1,"next_offset":null}
```
Sizes are upright (rotation applied). `missing: true` appears when the original cannot be found.
Analysis fields appear once the background analyses are done.

**Errors**: `INVALID_ARGUMENTS`.

### `get_frames`

Pictures of the sequence exactly as the viewer shows them — texts, shapes, transformations,
animations, transitions and reframing composited by `@se/engine` (golden rule 7). The core asks
the editor window (event `renderFramesRequested`); it draws each time offscreen and uploads JPEGs
(`PUT /media/renders/{request_id}/{index}`) or reports a failure (ADR-0028).

**Input**:
| Field | Type | Default |
|---|---|---|
| `times` | times[] (≤ 12) | — (give `times` or `range`) |
| `range` | `{start, end}` | the task's range, else the whole sequence |
| `count` | integer (≤ 12), with `range` | 6 |
| `max_width` | integer (64–1920) | 768 |
| `sequence` | string | the task's sequence |

With `range`, frames sit in the middle of `count` equal slices (`0–14 s`, 4 frames →
1.75, 5.25, 8.75, 12.25 s). Pictures keep the sequence's aspect, never wider than the sequence.

**Output**: a JSON header, then per frame a caption and an image:
```json
{"sequence_id":"…","width":768,"height":432,"frames":[{"timecode":"00:00:01.750","seconds":1.75}, …]}
```
`Frame 1 at 00:00:01.750` · *image/jpeg* · `Frame 2 at 00:00:05.250` · *image/jpeg* …

**Errors**: `INVALID_TIME` (outside the sequence), `TOO_MANY_FRAMES`, `INVALID_ARGUMENTS` (both
`times` and `range`), `RENDERER_UNAVAILABLE` (no window, or none answered within 30 s),
`RENDER_FAILED`.

### `get_source_frames`

Pictures of a media file itself (not the edit), extracted with ffmpeg and cached.

**Input**: `asset` (string, required), `times` (source times, 1–8, required), `max_width`
(16–3840, never enlarged; default 768).

**Output**: a JSON header, then per frame a caption (`Source frame 1 at 00:00:01.000`) and an
image:
```json
{"asset_id":"…","name":"interview.mp4","time_base":"source","frames":[{"timecode":"00:00:01.000","seconds":1.0}]}
```
Images give one picture whatever the time.

**Errors**: `ASSET_NOT_FOUND`, `INVALID_ARGUMENTS` (a sound file), `TOO_MANY_FRAMES`,
`INVALID_TIME`, `MEDIA_NOT_FOUND`, `MEDIA_TIME_OUT_OF_RANGE`, `MEDIA_TOOLS_MISSING`.

### `get_audio_analysis`

Silences, loudness (EBU R128) and peaks, from the analyses run in the background after import.

**Input**: either `asset` (its whole analysis, source time), or `range` (+ `sequence`) — default:
the task's range, else the whole sequence, in sequence time. `max_points` (1–200, default 40):
points of an asset's loudness curve.

**Output, asset**:
```json
{"asset_id":"…","name":"interview.mp4","time_base":"source","integrated_lufs":-15.9,"loudness_range_lu":6.0,
 "true_peak_dbtp":-1.0,"sample_peak_dbfs":-1.2,"silence_threshold_dbfs":-40.0,"min_silence":{"timecode":"00:00:00.500","seconds":0.5},
 "silences":[{"start":{"timecode":"00:00:07.000","seconds":7.0},"end":{"timecode":"00:00:08.000","seconds":8.0}}, …],"silence_total":2,
 "loudness":[{"time":{"timecode":"00:00:00.000","seconds":0.0},"lufs":-21.0}, …],
 "loudest":[{"time":{"timecode":"00:00:04.500","seconds":4.5},"lufs":-21.0}, …]}
```
`loudness`: the loudest short-term value (3 s window) of each equal slice; `loudest`: the five
loudest moments.

**Output, range** (sequence time):
```json
{"sequence_id":"…","time_base":"sequence","range":{…},
 "silences":[{"start":{"timecode":"00:00:02.000","seconds":2.0},"end":{"timecode":"00:00:03.000","seconds":3.0}}],"silence_total":1,
 "clips":[{"item_id":"…","track_id":"…","asset_id":"…","name":"interview.mp4","range":{…},"analysed":true,
   "integrated_lufs":-15.9,"loudest_lufs":-21.0,"silences":[{…2–3 s…},{…7–8 s…}]}, …],
 "unanalysed":[]}
```
`silences` (top level): where nothing is heard — no sound clip on an unmuted audio track, or only
their silences — at least 0.5 s long. Each clip's source silences are brought back through its
source window and speed. Unanalysed clips count as sound and are listed in `unanalysed`; muted
tracks' clips show `muted: true` and do not count.

**Errors**: `NOT_ANALYSED` (asset), `ASSET_NOT_FOUND`, `SEQUENCE_NOT_FOUND`, `INVALID_TIME`,
`INVALID_ARGUMENTS` (both `asset` and `range`).

### `get_scene_changes`

Shot changes found by analysis.

**Input**: either `asset` (source time), or `range` (+ `sequence`) — default: the task's range,
else the whole sequence, in sequence time.

**Output, asset**: `{asset_id, name, time_base: "source", threshold, changes: [time…], total}`.

**Output, range**:
```json
{"sequence_id":"…","time_base":"sequence","range":{…},
 "changes":[{"time":{"timecode":"00:00:02.000","seconds":2.0},"item_id":"…","asset_id":"…"},
            {"time":{"timecode":"00:00:11.000","seconds":11.0},"item_id":"…","asset_id":"…"}],
 "total":2,"cuts":[{"timecode":"00:00:10.000","seconds":10.0}],"unanalysed":[]}
```
`changes`: shot changes inside the media of visible picture clips (on visible video tracks),
brought back through each clip's source window and speed. `cuts`: where the edit itself cuts (a
picture clip starts or ends inside the range).

**Errors**: `NOT_ANALYSED` (asset), `ASSET_NOT_FOUND`, `SEQUENCE_NOT_FOUND`, `INVALID_TIME`,
`INVALID_ARGUMENTS`.

### `get_transcript`

The words spoken in a range. **Not available yet** (transcription arrives with prompt 21).

**Input**: `range`, `sequence` (accepted for the future contract).

**Output**: `{"available":false,"message":"Transcription is not available yet in this version of SimpleEffect. …"}`

### `list_animation_presets`

The animation presets items can get (the library of `se-domain`, `assets/animation-presets.json`).

**Input**: none.

**Output**:
```json
{"presets":[{"preset_id":"fade","name":"Fade","description":"Fades the item in from transparent (out: to transparent).",
   "phases":["in","out"],"default_duration":{"timecode":"00:00:00.500","seconds":0.5},"properties":["opacity"]},
  {"preset_id":"pulse","name":"Pulse","phases":["emphasis"], …}, …]}
```
Phases: `in` (entrance), `out` (exit: the entrance played backwards), `emphasis`. Preset ids are
stable.

### `get_brand_kit`

The project's brand kit. **Not available yet** (prompt 25).

**Input**: none. **Output**: `{"available":false,"message":"Brand kits are not available yet …"}`

### `list_templates`

Motion templates. **Not available yet** (prompt 24).

**Input**: none. **Output**: `{"available":false,"templates":[],"message":"Motion templates are not available yet …"}`

## Adding a tool

Recipe in `crates/se-mcp/CLAUDE.md`: a file in `src/tools/`, an entry in `tools::catalogue()`, a
unit or integration test, the schema snapshot accepted, a section here, and a line in
`crates/se-app/assets/agent-instructions/` when agents need guidance to use it.
