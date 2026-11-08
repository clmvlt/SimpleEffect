# AI tools — the MCP contract with agents

> The tools AI agents use to read and change a SimpleEffect project. This file is the
> **contract**: agents' behaviour, `crates/se-app/assets/agent-instructions/` and the tool
> definitions in `crates/se-mcp/src/tools/` must agree with it. Decisions: ADR-0003, ADR-0028,
> ADR-0029.
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
  project's `revision`) for the optimistic concurrency of write tools (`expected_revision`).
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
| `OUT_OF_SCOPE` | A write reaches outside the task's scope; the message gives the scope, the faulty item and times, and what to do |
| `REVISION_CONFLICT` | The item changed since the agent read it (`expected_revision`): read it again |
| `LOCKED_TRACK` | The item is on a track the user locked |
| `VALIDATION_ERROR` | A write breaks a document rule; the message names it (`rule OVERLAPPING_ITEMS`) |
| `TRANSITION_NO_ROOM`, `RIPPLE_BLOCKED` | No media to cross-fade at that cut; a ripple edit would cut other content |
| `INTERNAL` | A bug: report it |

- **Read tools change nothing** (annotation `readOnlyHint: true`); write tools are annotated
  `readOnlyHint: false`, `destructiveHint: true`.

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
| `get_transcript` | Spoken words of a sequence range, in sequence time (segments, word times, paginated) |
| `search_transcript` | Where words are said in the sequence (full-text search, word times) |
| `list_animation_presets` | Animation presets (fade, slides, pop…) |
| `get_brand_kit` | Brand kit (not available until prompt 25) |
| `list_templates` | Motion templates (not available until prompt 24) |
| `get_task_changes` | What the calling task changed so far |
| `add_text` | A styled text, with entrance/exit animations |
| `add_shape` | A rectangle, rounded rectangle, ellipse or line |
| `add_image` | An image asset (logo, sticker) |
| `update_item` | Static properties of an item (text, style, clip speed, fades, reframing, placement) |
| `set_keyframes` | Keyframes or a static value of one animatable property |
| `apply_animation_preset` | A preset expanded into keyframes (entrance, exit, emphasis) |
| `set_volume` | A clip's gain (static or automation) and fades |
| `move_item` | Moves an item (linked partners follow) |
| `trim_item` | Moves an item's edge |
| `split_at` | Cuts items at an instant |
| `delete_items` | Removes items, leaving gaps |
| `ripple_delete_range` | Removes a time range and closes it |
| `insert_gap` | Opens empty time |
| `add_transition` | A cross-fade on a cut |
| `add_marker` | A labelled marker |
| `apply_batch` | Several write actions as one change (all or nothing) |

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

The words spoken in a range of the sequence, in **sequence time** (prompt 21). Words come from
the transcripts of the media played by the sequence's **sound** clips: a passage cut from the
sequence has no words, a clip at 2× speed has its words twice as fast, the same sound on two
tracks counts once. A word belongs to the clip that plays its middle.

**Input**:

| Field | Type | Default | Meaning |
|---|---|---|---|
| `range` | range | task's range, else the whole sequence | Words whose middle lies in it |
| `sequence` | id or name | task's sequence | |
| `words` | bool | `true` | `false`: segments without word lists (shorter for long ranges) |
| `offset` | int | 0 | First word of the page (`next_offset` of the previous answer) |
| `limit` | int | 400 (≤ 2000) | Words per page |

**Output**:
```json
{"sequence_id":"…","time_base":"sequence","range":{…},"available":true,
 "segments":[{"start":{"timecode":"00:00:00.500","seconds":0.5},"end":{"timecode":"00:00:01.400","seconds":1.4},
              "text":"Bonjour à tous.","item_id":"…","asset_id":"…",
              "words":[{"text":"Bonjour","start_seconds":0.5,"end_seconds":0.9},
                       {"text":"à","start_seconds":0.9,"end_seconds":1.0},
                       {"text":"tous.","start_seconds":1.0,"end_seconds":1.4,"unsure":true}]}],
 "total_words":3,"next_offset":null,
 "untranscribed":[{"asset_id":"…","name":"music.wav"}]}
```
- Word times are plain seconds (millisecond precision) to keep long transcripts short; segment
  times follow the usual `{timecode, seconds}` form. Punctuation is part of its word (`pensez ?`).
- `unsure` marks words the model was not sure of (confidence < 0.5): check them with
  `get_frames`/the context before relying on them.
- `available: false` when no sound clip in the range is transcribed; `untranscribed` lists the
  media of sound clips in the range without a transcript. Agents cannot transcribe: ask the user
  to click « Transcrire » (media bin or transcript panel).

**Errors**: `SEQUENCE_NOT_FOUND`, `INVALID_TIME`, `INVALID_ARGUMENTS`.

### `search_transcript`

Where words are said in the sequence (prompt 21): the project's transcripts are searched with a
full-text index (accents and case ignored, every word required, the last one as a prefix), then
each matching phrase is placed where the sequence plays it (nowhere when it was cut, several
times when it is used twice).

**Input**: `query` (required: `"abonnez-vous"`, `"prix euros"`), `sequence` (default: the
task's), `range` (only phrases intersecting it), `limit` (default 20, ≤ 50).

**Output**:
```json
{"sequence_id":"…","time_base":"sequence",
 "results":[{"start":{"timecode":"00:00:03.000","seconds":3.0},"end":{"timecode":"00:00:04.600","seconds":4.6},
             "text":"Abonnez-vous à la chaîne !","item_id":"…","asset_id":"…",
             "matches":[{"text":"Abonnez-vous","start_seconds":3.0,"end_seconds":3.8},
                        {"text":"chaîne !","start_seconds":4.0,"end_seconds":4.6}]}],
 "elsewhere":0}
```
`elsewhere` counts matching phrases of the project's media that this sequence does not play.
Read around a result with `get_transcript` before editing.

**Errors**: `INVALID_ARGUMENTS` (a query without words), `SEQUENCE_NOT_FOUND`, `INVALID_TIME`.

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

## Write tools (prompt 19)

Write tools change the project **live**, inside the task's scope, through
`se_app::AgentEditService` (ADR-0029): the tool's input becomes typed edits, planned on the current
document (each edit sees the previous ones), checked against the scope, and committed as **one**
transaction of actor `aiTask` by the project's actor, which records it in the task's changeset.
They are annotated `readOnlyHint: false`, `destructiveHint: true`.

**Scope rule.** Everything an edit touches — items, markers, transitions, tracks — must lie in the
task's scope **before and after** the edit: on the task's sequence, entirely inside its time range
(`null`: the whole sequence) and on its tracks (`null`: every track). Items crossing the scope's
edges cannot be changed; new items must lie inside it. The check runs on the whole transaction, so
it also covers what an edit drags along: linked partners, items shifted by a ripple edit,
transitions dropped by a move. AI tasks never change track headers, sequences (name, format),
assets or the project; they add tracks only when they may use every track.

```json
{"error":{"code":"OUT_OF_SCOPE","message":"the edit would leave text « Late » 0193… on V3 of « Main edit », 00:00:08.000–00:00:10.000 outside this task's scope (00:00:01.000–00:00:09.000 of « Main edit », every track): keep every change between 00:00:01.000 and 00:00:09.000"}}
```

**Common rules.**
- Items are given by `item_id` (from `get_timeline`), or in `apply_batch` by `"$name"` for an item
  created by an earlier action named with `"as": "name"`.
- `expected_revision` (tools changing one item; `expected_revisions` for `delete_items`): the
  revision the agent read. If the item changed since, the edit is refused with
  `REVISION_CONFLICT`: read it again (`get_item`) and redo the change.
- Moves, trims, splits and deletes keep **linked items** together (a clip's picture and sound), as
  the timeline does.
- Times are sequence times, except keyframe times (`set_keyframes`, `set_volume` automation,
  `apply_animation_preset` `start`), which are **relative to the item's start**.
- If the project changes between planning and commit (the user edits), the call is planned again
  on the new state (a few times), so its scope check always holds.

**Output** (every write tool except `apply_batch`):
```json
{"revision":48,"created":["…0f42…"],
 "items":[{"item_id":"…0f42…","revision":2,"kind":"text","track_id":"…","start":{"timecode":"00:00:51.000","seconds":51.0},
   "end":{"timecode":"00:00:54.000","seconds":54.0},"label":"Abonne-toi","animated":["scale","opacity"],"origin":"ai_task","task_id":"…"}],
 "removed":["…"]}
```
`created`: items the call created, in order. `items`: summaries (as in `get_timeline`) of the items
added or changed, with their new revisions. `removed` appears when items were removed.

**Errors of every write tool**: `OUT_OF_SCOPE`, `REVISION_CONFLICT`, `LOCKED_TRACK`,
`VALIDATION_ERROR` (a document rule: overlap, trim beyond the media, keyframe outside the item…;
the message names the rule), `ITEM_NOT_FOUND` / `TRACK_NOT_FOUND` / `ASSET_NOT_FOUND`,
`INVALID_ARGUMENTS`, `INVALID_TIME`, `PROJECT_NOT_OPEN`, `ACCESS_REVOKED`.

Examples of requests and the calls an agent makes:

| Request | Calls |
|---|---|
| « De 51 s à 54 s, affiche “Abonne-toi” au centre avec un pop » | `add_text {"text":"Abonne-toi","start":51,"end":54,"position":"center","font_size":140,"font_weight":800,"stroke":{},"animation_in":{"preset":"pop"}}`, then `get_frames {"times":[51.2,52.5]}` |
| « Un bandeau noir derrière le titre » | `apply_batch` with `add_shape` (`"as":"bar"`) then `add_text` over it, both animated |
| « Coupe les silences entre 10 s et 20 s » | `get_audio_analysis {"range":{"start":10,"end":20}}`, then `apply_batch` of `split_at` + `delete_items` (or `ripple_delete_range` when the scope reaches the end) |
| « Baisse la musique sous la voix » | `get_audio_analysis`, then `set_volume` with an `automation` |
| « Fondu enchaîné à la coupe de 10 s » | `add_transition {"track":"V1","at":10,"duration":0.5}` |

### `add_text`

An editable text over a time range, with optional entrance and exit animations, in one edit.

**Input**:
| Field | Type | Default |
|---|---|---|
| `text` | string (`\n` for line breaks) | required |
| `start` | time | required |
| `end` / `duration` | time (exactly one) | required |
| `position` | `"center"`, `"top"`, `"bottom"`, `"left"`, `"right"`, `"lower_third"` or `{x, y}` (0–1) | `"center"` |
| `font_size` | design pixels (1080 = shorter side) | 96 |
| `font_family`, `font_weight` (1–1000), `italic` | | `"Inter"`, 600, false |
| `color` | `#rrggbb[aa]` | white |
| `align` | `left`, `center`, `right` | center |
| `max_width` | design pixels (wrapping) | none |
| `uppercase`, `shadow` | booleans | false |
| `stroke` | `{color?, width?}` (outline on) | off |
| `background` | `{color?}` (box on) | off |
| `track` | video track (id or name) | the highest free track of the task (a new one on top when the task may use every track) |
| `animation_in` / `animation_out` | `{preset, duration?}` (`list_animation_presets`) | none |

Named positions: centre (0.5, 0.5), top (0.5, 0.15), bottom (0.5, 0.8), left (0.2, 0.5), right
(0.8, 0.5), lower third (0.3, 0.75). Output: the common write output (`created` holds the text).

### `add_shape`

A rectangle, rounded rectangle, ellipse or line.

**Input**: `shape` (`rectangle`, `rounded_rectangle`, `ellipse`, `line`; required), `start`,
`end`/`duration`, `position`, `width`/`height` (design pixels; a line's length/thickness),
`corner_radius`, `color` (fill), `stroke` `{color?, width?}`, `cover_frame` (stretch over the
frame), `opacity` (0–1), `track`, `animation_in`, `animation_out`. Defaults: 600 × 338
(rectangles), 400 × 400 (ellipse), 600 × 8 (line), white, centred.

### `add_image`

An image asset of the project (logo, sticker, picture).

**Input**: `asset` (an `asset_id` of kind `image`; required), `start`, `end`/`duration`,
`position`, `fit` (`fit`, `fill`, `natural`; default fit), `scale` (1 = as fitted), `opacity`,
`track`, `animation_in`, `animation_out`. **Errors** also `INVALID_ARGUMENTS` for an asset that is
not an image.

### `update_item`

Static properties of one item, in one edit; give only the fields to change.

**Input**: `item_id` (required), `expected_revision`, and per kind:
| Kind | Fields |
|---|---|
| text | `text`, `font_family`, `font_size`, `font_weight`, `italic`, `color`, `align`, `max_width` (0 removes wrapping), `uppercase`, `stroke`, `shadow`, `background` (booleans switch them on or off) |
| shape | `color` (fill), `stroke` (boolean), `width`, `height`, `corner_radius` |
| image | `fit` |
| media clip | `speed`, `fade_in`, `fade_out` (lengths), `reframe` (`fit`, `fill`, `custom`) |
| any visual item | `position`, `scale` (number or `[x, y]`), `rotation` (degrees), `opacity`, `blur` — static values replacing keyframes |

Fields of another kind are refused (`INVALID_ARGUMENTS`: « a text has no speed; a text accepts …»).
Sizes and colours given here are static (keyframes of those properties are replaced). Timing:
`move_item`, `trim_item`; animations: `set_keyframes`, `apply_animation_preset`.

### `set_keyframes`

The keyframes (or a static value) of one animatable property.

**Input**:
| Field | Type |
|---|---|
| `item_id` | string (required) |
| `property` | `position`, `scale`, `rotation`, `anchor`, `opacity`, `blur`, `volumeDb`, `reframeFocus`, `fontSize`, `fillColor`, `letterSpacing`, `reveal`, `strokeColor`, `strokeWidth`, `shadowColor`, `shadowBlur`, `shadowOffset`, `backgroundColor` (the document's names) |
| `keyframes` | `[{time, value, easing?}]`, time **relative to the item's start**, sorted, within the item |
| `value` | a static value (instead of `keyframes`) |
| `expected_revision` | integer |

Values: numbers; `[x, y]` or `{x, y}` for position, scale, anchor, reframeFocus, shadowOffset;
`"#rrggbb[aa]"` for colours. Easing: `linear` (default), `hold`, `ease_in`, `ease_out`,
`ease_in_out`, or `[x1, y1, x2, y2]` (cubic Bézier). A value of the wrong type is refused
(`INVALID_ARGUMENTS`: « values of opacity are numbers »); ranges (opacity 0–1, volume −96–24 dB…)
are document rules (`VALIDATION_ERROR`).

### `apply_animation_preset`

A preset (`list_animation_presets`) expanded into keyframes: `item_id`, `preset`, `phase` (`in`,
`out`, `emphasis`; required), `duration` (default: the preset's), `start` (relative to the item;
default: its start for `in`/`emphasis`, ending at its end for `out`), `expected_revision`.
Keyframes of the same properties inside the animation's span are replaced. **Errors** also
`VALIDATION_ERROR` for an unknown preset (the message lists them), a phase the preset lacks, or an
item without the preset's properties.

### `set_volume`

The gain of a clip with sound and its fades: `item_id` (required), `gain_db` (static, −96–24 dB)
**or** `automation` (`[{time, gain_db, easing?}]`, times relative to the clip), `fade_in`,
`fade_out` (lengths; 0 removes), `expected_revision`. One edit (volume keyframes + fade patch).

### `move_item`

`item_id` (required), `start` (sequence time) and/or `track` (id or name, same kind),
`expected_revision`. Linked partners move by the same amount on their own tracks. Refused
(`VALIDATION_ERROR`, `OVERLAPPING_ITEMS`) onto another item.

### `trim_item`

`item_id`, `edge` (`start` or `end`), `to` (sequence time), `expected_revision`. A clip plays more
or less of its media (`TRIM_BEYOND_SOURCE` past it); nothing else moves. Linked partners whose
same edge sits at the same instant follow.

### `split_at`

`at` (required), `item_ids` (default: every item crossing `at` on the task's unlocked tracks).
Linked partners crossing `at` are cut too and the right parts stay linked. `created` lists the
right parts.

### `delete_items`

`item_ids` (required), `expected_revisions` (`{"<item_id>": revision}`, optional). Linked
partners go too; the time stays empty (nothing moves).

### `ripple_delete_range`

`range` (required), `tracks` (default: the task's tracks; every unlocked track when it may use them
all). Items crossing the range are cut, what follows moves left; tracks holding linked partners
are added. Since later items move, it is refused (`OUT_OF_SCOPE`) when that reaches outside a
time-limited scope.

### `insert_gap`

`at`, `duration` (required), `tracks` (as for `ripple_delete_range`). Opens empty time; items
crossing `at` are cut, what follows moves right. Same scope rule as `ripple_delete_range`.

### `add_transition`

A cross-fade on the cut between two adjacent items of one track (picture dissolve on video
tracks, sound cross-fade on audio tracks): `left_item_id` + `right_item_id`, **or** `track` + `at`
(the cut within half a frame); `duration` (default 0.5 s, shortened to the media both items have
beyond the cut); `alignment` (`center`, `start`, `end`). **Errors** also `TRANSITION_NO_ROOM` (no
media to cross-fade, or the cut already has a transition), `INVALID_ARGUMENTS` (no cut there).

### `add_marker`

`time` (required), `label` (default empty), `color` (`green`, `red`, `purple`, `orange`,
`yellow`, `white`, `blue`, `cyan`; default green). Markers must lie inside the scope's range.

### `apply_batch`

Several write actions as **one** transaction: all applied, or none.

**Input**: `actions` (1–50), each `{tool, arguments, as?}`: `tool` is one of the write tools
above (not `apply_batch`), `arguments` exactly its input, `as` names the item the action creates
(`add_text`, `add_shape`, `add_image`, `split_at`) so later actions can use `"$name"` where an item
id is expected.

```json
{"actions":[
  {"tool":"add_shape","as":"bar","arguments":{"shape":"rounded_rectangle","start":51,"end":54,"position":"bottom","width":900,"height":160,"color":"#000000aa"}},
  {"tool":"add_text","as":"title","arguments":{"text":"Abonne-toi","start":51,"end":54,"position":"bottom"}},
  {"tool":"apply_animation_preset","arguments":{"item_id":"$title","preset":"pop","phase":"in"}}]}
```

**Output**: `{revision, actions: [{tool, created: [ids]}], items, removed?}`.

**Errors**: those of the actions, prefixed with the failing action
(`"action 2 (move_item): item … would overlap …"`); nothing is applied then. `INVALID_ARGUMENTS`
for an unknown tool, `as` on an action that creates nothing, or more than 50 actions.

### `get_task_changes`

What the calling task changed so far (its open changeset), to check and summarise its work.
Read-only.

**Input**: none. **Output**:
```json
{"changeset_id":"…","edits":2,"created":[<item summary>],"changed":[<item summary>],"removed":["…"]}
```
`edits`: write calls committed. `created`: items the task added that still exist; `changed`: items
that existed before, changed by the task; `removed`: items it removed. `changeset_id` is `null`
while the task changed nothing (or after a revert).

## Changesets and reverts

Every committed write call of a task is one entry of the task's **changeset**
(`se_domain::Changeset`, stored in `project.db` with its journal entry in one SQLite transaction).
The user reverts a finished task with the protocol command `revertTask { projectId, taskId }`: the
project's actor plans the revert against the current document (`se_domain::plan_revert`), commits
what still applies as one **user** transaction (undoable, label « Revert AI task ») and answers
`TaskRevertReport { taskId, reverted, revertedItems, conflicts }`. Conflicts: `itemChanged` (someone
changed the item after the task: its edits stay), `foreignItems`, `cannotApply`. A running task
cannot be reverted (`TASK_STILL_ACTIVE`); a task with nothing left gets `NOTHING_TO_REVERT`. Each
change of a changeset publishes `taskChangesetUpdated { projectId, taskId, changesetId,
touchedItems }` (the items the task added or changed that still exist), for the UI to highlight
them.

## Adding a tool

Recipe in `crates/se-mcp/CLAUDE.md`: a file in `src/tools/`, an entry in `tools::catalogue()`, a
unit or integration test, the schema snapshot accepted, a section here, and a line in
`crates/se-app/assets/agent-instructions/` when agents need guidance to use it. A write tool also
implements `WriteTool` (its input as `AgentEdit`s) and gets its arm in `apply_batch`.
