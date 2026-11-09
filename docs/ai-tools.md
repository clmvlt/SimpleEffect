# AI tools — the MCP contract with agents

> The tools AI agents use to read and change a SimpleEffect project. This file is the
> **contract**: agents' behaviour, `crates/se-app/assets/agent-instructions/` and the tool
> definitions in `crates/se-mcp/src/tools/` must agree with it. Decisions: ADR-0003, ADR-0028,
> ADR-0029, ADR-0035 (motion templates).
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
| `RENDERER_UNAVAILABLE` | No editor window is open, or it did not draw the frames within 30 s (check a motion template within 45 s) |
| `RENDER_FAILED` | The editor window could not draw the frames (its message says why) |
| `PROJECT_NOT_OPEN` | The user closed the project |
| `MEDIA_NOT_FOUND`, `MEDIA_TIME_OUT_OF_RANGE`, `MEDIA_TOOLS_MISSING`, `DERIVED_NOT_APPLICABLE` | Source frame extraction failed (missing file, time outside the media, no ffmpeg) |
| `OUT_OF_SCOPE` | A write reaches outside the task's scope; the message gives the scope, the faulty item and times, and what to do |
| `REVISION_CONFLICT` | The item changed since the agent read it (`expected_revision`): read it again |
| `LOCKED_TRACK` | The item is on a track the user locked |
| `VALIDATION_ERROR` | A write breaks a document rule; the message names it (`rule OVERLAPPING_ITEMS`) |
| `TRANSITION_NO_ROOM`, `RIPPLE_BLOCKED` | No media to cross-fade at that cut; a ripple edit would cut other content |
| `NO_TRANSCRIPTION_MODEL`, `TRANSCRIPTION_UNAVAILABLE` | No speech model is installed (the user downloads one in « Réglages → Transcription »), or whisper.cpp cannot run (the message says why) |
| `TEMPLATE_INVALID` | A motion template module cannot be used: every problem with its stage, line and instant; nothing was added |
| `TEMPLATE_NOT_FOUND` | No such motion template; the message lists the project's templates |
| `LIBRARY_TEMPLATE_NOT_FOUND` | No such entry in the user's template library (removed meanwhile): list it again with `list_templates` and `source: "library"` |
| `VISION_UNAVAILABLE` | Faces cannot be detected on this machine (the message says why: missing ONNX Runtime library, Intel Mac): reframe by hand with `update_item` / `set_keyframes` |
| `INTERNAL` | A bug: report it |

- **Read tools change nothing** (annotation `readOnlyHint: true`); write tools are annotated
  `readOnlyHint: false`, `destructiveHint: true`. `transcribe_media` adds transcripts next to the
  project without touching the timeline, and `create_motion_template` adds a template without
  placing it: `readOnlyHint: false`, `destructiveHint: false`.

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
| `transcribe_media` | Transcribes untranscribed media locally, then waits for the result |
| `list_animation_presets` | Animation presets (fade, slides, pop…) |
| `get_brand_kit` | The project's brand kit: fonts, named palette, logos, looks of new texts and captions, favourite templates |
| `list_templates` | Motion templates of the project or of the user's library (by category; parameters, brand roles), or one with its source |
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
| `remove_words` | Cuts transcript words (text-based editing), keeping natural pauses |
| `remove_fillers` | Cuts filler words (« euh », « um »…) of a range |
| `remove_silences` | Cuts the pauses of a range, from the sound's level |
| `create_captions` | Captions from the transcript of a range, on the caption track |
| `update_caption_style` | How captions look and animate (preset, fonts, colours, position…) |
| `edit_caption_text` | Corrects a caption's text, keeping word timings |
| `emphasize_caption_words` | Puts words of a caption forward (emphasis colour) |
| `create_motion_template` | A motion template from a TypeScript module, checked by the editor (pictures or every problem) |
| `update_motion_template` | A new version of a template's module (copied when instances lie outside the scope) |
| `instantiate_template` | Places a motion template over a time range with parameter values |
| `update_template_params` | Changes the parameter values of a template instance (keyframes for numbers and colours) |
| `insert_library_template` | Places a motion template of the user's library (copied into the project) with parameter values |
| `auto_reframe` | Makes media clips follow their subject (faces), or adds a version of the sequence in another format (9:16…) whose clips do |

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
`time` (relative to the item's start, plus `sequence_time`), a caption's word `start`/`end` (same
form), and a clip's `sourceStart`, `fadeIn`, `fadeOut`. A caption also gets `caption_style`: its
style as stored (font, colours, outline, shadow, background, position, animation, line limits).
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
  media of sound clips in the range without a transcript: transcribe them with
  `transcribe_media`, then read again.

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

### `transcribe_media`

Transcribes media that have no transcript of their current content, **locally** with the speech
model the user installed (whisper.cpp, the project's language and vocabulary), then waits for the
result (ADR-0034). It changes no timeline item: transcripts are stored next to the project, like
analyses, and the task's scope does not limit it. Already transcribed media cost nothing, running
transcriptions are awaited rather than started again, failed or cancelled ones are tried again.

**Input**:

| Field | Type | Default | Meaning |
|---|---|---|---|
| `asset_ids` | string[] | media of the sound clips of `range` | Media to transcribe (`list_assets`, `get_transcript`'s `untranscribed`) |
| `range` | range | task's range, else the whole sequence | Ignored with `asset_ids` |
| `sequence` | id or name | task's sequence | |
| `wait_seconds` | number | 90 (≤ 110) | How long to wait; `0` starts and answers at once |

**Output**:
```json
{"sequence_id":"…","finished":false,"started":1,
 "assets":[{"asset_id":"…0001…","name":"interview.mp4","status":"running","progress":0.42},
           {"asset_id":"…0002…","name":"music.wav","status":"transcribed","word_count":12},
           {"asset_id":"…0003…","name":"logo.png","status":"unavailable","reason":"no_sound: this media has no sound to transcribe"}],
 "note":"still transcribing: call transcribe_media again with the same arguments to keep waiting (nothing starts twice), or do other work meanwhile"}
```
- `status`: `transcribed` (`word_count`), `queued`, `running` (`progress` 0–1 when known),
  `failed` (`reason`: whisper.cpp's or ffmpeg's message), `cancelled`, `unavailable` (`reason`:
  `media_missing` — ask the user to relink —, `no_sound`, `not_ready` — still importing, call
  again shortly), `not_started`.
- `finished: false` means a transcription is still queued or running when the wait ended: call
  again with the same arguments. A few minutes of speech take seconds on a GPU, longer on a CPU.
- `started` counts the media this call queued.

**Errors**: `NO_TRANSCRIPTION_MODEL` (tell the user to download a model in « Réglages →
Transcription »; agents never download), `TRANSCRIPTION_UNAVAILABLE`, `ASSET_NOT_FOUND`,
`SEQUENCE_NOT_FOUND`, `INVALID_TIME`, `INVALID_ARGUMENTS` (empty `asset_ids`, negative
`wait_seconds`).

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

The project's brand kit (prompt 25, ADR-0036): the user's visual identity, a copy of one of their
library kits. **Use it consistently** — heading font for titles, body font for text, the palette's
roles for colours, the logos — unless the user asks otherwise. New texts (`add_text` fields left
out), new captions (`create_captions` without `style`, or `style: "brand_kit"`) and template
parameters bound to a role already take its values.

**Input**: none.

**Output**:
```json
{"available":true,"name":"Example brand",
 "fonts":{"heading":"Montserrat","body":"Inter"},
 "colors":{"primary":"#ffcc00","secondary":"#1a1a1a","accent":"#ff3366","text":"#ffffff","background":"#1a1a1a"},
 "logos":[{"name":"Logo","asset_id":"…0003…"}],
 "text_style":{"font_family":"Montserrat","font_weight":700,"font_size":96.0,"color":"#ffffff","uppercase":false,"stroke":null,"shadow":false,"background":null},
 "caption_style":{"font_family":"Inter","text_color":"#ffffff","active_color":"#ffd60a","animation":"wordHighlight"},
 "intro_templates":[{"library_template_id":"…","name":"Lower third"}]}
```
Logos are image assets of the project (`add_image`, or a template's `asset` parameter).
`intro_templates`/`outro_templates` (absent when empty) are library templates the user prefers
for openings and endings (`insert_library_template`). Without a kit:
`{"available":false,"message":"This project has no brand kit: …"}`.

### `list_templates`

Motion templates of the project (prompt 24, below) or of the user's **template library**
(prompt 25): templates the user saved to reuse in every project. Prefer a fitting library template
(`insert_library_template`) over writing a new one.

**Input**:
| Argument | Type | Default |
|---|---|---|
| `source` | `project` or `library` | `project` |
| `category` | `title`, `lowerThird`, `callToAction`, `transition`, `other` | every category |
| `template_id` | a project `template_id` (or a `library_template_id` with `source: "library"`) | none: the list |

**Output**:
```json
{"templates":[{"template_id":"…","name":"Lower third","description":"Name and role with a sliding bar","version":2,
   "default_duration":{"timecode":"00:00:05.000","seconds":5.0},
   "params":[{"name":"name","label":"Nom","type":"string","default":"Jane Doe"},
     {"name":"size","label":"Taille","type":"number","default":72.0,"minimum":40.0,"maximum":140.0,"animatable":true},
     {"name":"accent","label":"Accent","type":"color","default":"#ffcc00","animatable":true},
     {"name":"side","label":"Côté","type":"enum","default":"left","options":["left","right"]},
     {"name":"logo","label":"Logo","type":"asset","default":null}],
   "instances":1,"category":"lowerThird","library_template_id":"…"}]}
```
`category` is the module's `meta.category`; `library_template_id` (absent when none) names the
library entry the template was copied from; a parameter's `brand` (absent when none) is the brand
kit role it starts from. With `source: "library"`:
```json
{"templates":[{"library_template_id":"…","name":"Bandeau","description":"Nom et fonction","category":"lowerThird",
   "tags":["interview"],"version":3,"default_duration":{"timecode":"00:00:05.000","seconds":5.0},
   "params":[{"name":"accent","label":"Accent","type":"color","default":"#ffcc00","animatable":true,"brand":"accent"}, …],
   "project_template_id":"…"}]}
```
(`project_template_id`: the project's copy of that version, when it has one). Caption styles of
the library are not listed. With `template_id`: `{"template":{…},"source":"export const meta = …"}`.
Types: `string` (with `multiline`), `number`, `color`, `boolean`, `enum` (`options`), `font`,
`asset`. **Errors**: `TEMPLATE_NOT_FOUND`, `LIBRARY_TEMPLATE_NOT_FOUND`, `INVALID_ARGUMENTS`
(unknown category or source).

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
assets or the project; they add tracks only when they may use every track. One exception
(prompt 26): `auto_reframe` with `target_format` adds a **new** sequence — a version of the
task's sequence — after it; everything inside a sequence the call creates is new content, so
the scope allows it, and reverting the task removes it. No other sequence can be added, changed
or removed.

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
| « Coupe les silences entre 10 s et 20 s » | `remove_silences {"range":{"start":10,"end":20},"tracks":["A1"]}` (smart cuts, below) |
| « Enlève les euh » | `remove_fillers {}` |
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
| `font_size` | design pixels (1080 = shorter side) | the brand kit's text style, else 96 |
| `font_family`, `font_weight` (1–1000), `italic` | | the brand kit's, else `"Inter"`, 600, false |
| `color` | `#rrggbb[aa]` | the brand kit's, else white |
| `align` | `left`, `center`, `right` | the brand kit's, else center |
| `max_width` | design pixels (wrapping) | the brand kit's, else none |
| `uppercase`, `shadow` | booleans | the brand kit's, else false |
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

## Smart cuts (prompt 22)

`remove_words`, `remove_fillers` and `remove_silences` cut time out of the sequence the way the
user's text-based editing does (`se_app::smart_cuts`, ADR-0032): the cuts are resolved on the
current document, made frame-exact, and removed as **one** ripple transaction
(`RippleDeleteRanges`) on the speech's sound tracks **and the tracks of their linked picture**
(`all_tracks: true`: every unlocked track, graphics and music included). Linked parts stay linked
between the same two cuts, so picture and sound never drift. The result is ordinary items: the user
can undo it, or lengthen a pause with a ripple trim. These tools are not batchable.

**Cut scope rule.** A cut removes time, so everything after it moves left and a clip crossing the
scope's edge is cut inside it only. Instead of the strict rule above, a cut is accepted when **the
time it removes lies in the scope's range** and **every track it edits is one of the task's
tracks**. Cuts are clipped to the scope's range automatically; a linked partner on a track outside
the scope makes the call fail with `OUT_OF_SCOPE` (« the cut would also edit track V1… »).

**Margins.** Words are removed with natural margins: the longer pause around them is kept, the
shorter one is cut to a breath (60 ms) before the next kept word, a removed word is cut 30 ms
before its start, never more than half a pause, never inside a kept word; when the sound's level
is known, an edge falling in sound moves back to where that sound began if it began within 150 ms
(whisper's word starts can be late), so a removed word's onset goes with it. Less than 80 ms left
between two cuts or at a clip edge is removed too. Silences keep `padding` on each side.

**Output** (all three):
```json
{"cuts":3,"removed":{"timecode":"00:00:02.336","seconds":2.336},"words_removed":3,
 "ranges":[{"start":{"timecode":"00:00:01.968","seconds":1.968},"end":{"timecode":"00:00:02.336","seconds":2.336}}],
 "tracks":["V1","A1"],"revision":52}
```
`ranges`: the cuts in sequence time **before** the call (first 50), so later cuts' times shift by
the time removed before them: read the transcript or the timeline again before cutting more.
`revision` is absent and `note` says why when nothing was cut. `untranscribed` / `unanalysed` list
media whose words or sound could not be read (`[{asset_id, name}]`).

**Errors**: `OUT_OF_SCOPE`, `VALIDATION_ERROR` (a locked speech track), `INVALID_ARGUMENTS`,
`INVALID_TIME`, `TRACK_NOT_FOUND`, `PROJECT_NOT_OPEN`, `ACCESS_REVOKED`.

### `remove_words`

`ranges` (required, 1–500, `[{start, end}]` in sequence time, as `get_transcript` gives words:
first word's start → last word's end; a word goes when its middle lies in a range), `all_tracks`
(default false). To keep the best take of a repeated sentence, compare the repetitions in
`get_transcript` and remove the others with one call.

### `remove_fillers`

`range` (default: the task's range, else the whole sequence), `words` (fillers to remove instead
of the user's lists: words or short phrases, case, accents and punctuation ignored, repeated
letters once — « euuuh » is « euh »), `all_tracks`. The user's lists are per language (French:
euh, heu, hum, hmm, mmh, bah, ben, beh, euhm; English: um, uh, uhm, erm, er, hmm, mm, you know);
occurrences longer than the user's maximum (1.5 s) are skipped. Only transcribed fillers can be
found.

### `remove_silences`

`range` (default: the task's range), `threshold_db` (-80 to -10 dBFS, default the user's -40),
`min_silence` (seconds, 0.1–10, default 0.5), `padding` (seconds kept on each side, 0–1, default
0.12), `tracks` (sound tracks listened to, ids or names; every one must be quiet; default: every
unmuted, unlocked sound track holding media, or the task's sound tracks), `all_tracks`. Silences
come from the assets' level envelopes (20 ms RMS, `get_audio_analysis`); time where no listened
track has a clip is never cut. **Leave music tracks out**: music is never silent.

## Captions (prompt 23)

A caption is an item of kind `caption` holding **timed words** (`words: [{text, start, end,
emphasis}]`, times relative to the item) and a `styleId`: a **caption style** of the project,
shared by captions (`captionStyles`). Styles say how captions look (font, size, letter case, text
colour, colour of the word being spoken, colour of emphasised words, outline, shadow, box, position
of the block's bottom centre, line limits) and animate: `none`, `word_highlight` (the spoken word
coloured; `highlight_spoken` keeps spoken words coloured, karaoke-like), `pop_per_word` (each word
pops as it starts), `box_per_line` (each line on its own box), `reveal_per_word` (words appear as
spoken). Predefined styles: `tiktokBold`, `minimal`, `karaoke`, `box`.

Captions live on the **caption track**: the highest unlocked video track holding captions only (a
video track « Sous-titres » is created on top when there is none — that needs a task over every
track). They are ordinary items: `move_item`, `trim_item`, `split_at` and `delete_items` work on
them (words follow: a word belongs to the part holding its middle), and smart cuts cut the caption
track with the speech. A caption style may change only when every caption using it lies in the
task's scope; `update_caption_style` with `item_ids` copies a shared style for the listed captions.
Agent instructions: `07-captions.md` (readability, line length, safe areas).

### `create_captions`

| Argument | Type | Meaning |
|---|---|---|
| `range` | `{start, end}` | Where (default: the task's range, else the whole sequence); captions of the caption track crossing it are replaced |
| `style` | string | A preset (`tiktokBold`, `minimal`, `karaoke`, `box`), `brand_kit` (the project's brand kit look) or a `style_id` (default: the replaced captions' style, else the brand kit's look, else `minimal`) |
| `max_chars_per_line` | integer | 4–80 (default: the style's) |
| `max_lines` | integer | 1–4 (default: the style's) |

Words (from the transcript, as `get_transcript` gives them) whose middle lies in the range are
grouped: a new caption after a 0.6 s pause, a sentence end, a clause end once half the room is used,
or when the lines are full (cut after a comma when possible); captions last 0.7–6 s, short gaps are
closed, edges are snapped to frames. Not batchable.

**Output**:
```json
{"revision":57,"style_id":"…","track_id":"…",
 "captions":[{"item_id":"…","start":{"timecode":"00:00:00.167","seconds":0.167},"end":{"timecode":"00:00:01.101","seconds":1.101},"text":"Bonjour à tous."}]}
```
Nothing spoken in the range: `captions` is empty, `revision` is `null` and `note` says why
(`untranscribed`: media to transcribe first).

**Errors**: `INVALID_ARGUMENTS` (unknown style), `VALIDATION_ERROR` (limits out of range),
`OUT_OF_SCOPE` (a caption to replace straddles the scope, a new caption track without every track),
`INVALID_TIME`.

### `update_caption_style`

`style_id` **or** `item_ids` (required, one of them), then any of: `preset` (applied first),
`font_family`, `font_size` (design pixels), `font_weight`, `italic`, `uppercase`, `text_color`,
`active_color`, `emphasis_color`, `outline`, `outline_color`, `outline_width`, `shadow`,
`background`, `background_color`, `position` (`{x, y}` or a named place: the block's bottom
centre), `animation` (`none`, `word_highlight`, `pop_per_word`, `box_per_line`,
`reveal_per_word`), `highlight_spoken`, `max_chars_per_line`, `max_lines`. With `style_id` the
style changes in place (every caption using it must be in the scope); with `item_ids` a style also
used by other captions is copied for them. Batchable. **Output**: as other write tools (`revision`,
`items`). **Errors**: `INVALID_ARGUMENTS` (neither or both targets, not a caption, unknown preset,
bad colour, nothing to change), `VALIDATION_ERROR` (values out of range), `OUT_OF_SCOPE`.

### `edit_caption_text`

`item_id` (required; `$name` in a batch), `text` (required, not empty). Words left unchanged keep
their timing; a run of changed words with as many new words keeps the old timings one for one (a
typo fix); otherwise the new words share the replaced words' time (or the pause around an
insertion) in proportion to their length. Batchable. **Errors**: `INVALID_ARGUMENTS` (empty text,
not a caption), `ITEM_NOT_FOUND`, `OUT_OF_SCOPE`.

### `emphasize_caption_words`

`item_id` (required), `words` (required: indices from 0, as `get_item` lists them), `emphasis`
(default true; false puts them back). Emphasised words are drawn in the style's emphasis colour
(the active colour wins while spoken). Batchable. **Errors**: `INVALID_ARGUMENTS`,
`VALIDATION_ERROR` (an index beyond the caption's words), `OUT_OF_SCOPE`.

## Motion templates (prompt 24)

ADR-0035. A **motion template** is a small TypeScript module an agent writes: `meta` (name,
description, default length), `params` (the values the user may change, declared as a JSON Schema
subset) and `render(ctx, time, params)`, which draws one frame on a restricted Canvas 2D in design
pixels. The editor runs it in a sandboxed Worker at every frame, in preview, `get_frames` and
export alike; the user changes its values in an inspector generated from `params` (texts,
numbers and colours with keyframes, switches, options, fonts, images) without touching the code.
The module's contract, drawing API, rules and four complete examples are in the agent
instructions `08-motion-templates.md`.

A **template instance** is an item of kind `motion_template` (`get_timeline`, `get_item`): the
template's id, the values set (unset ones read the defaults) and a transform applied to the whole
frame the template draws. It is an ordinary graphic: `move_item`, `trim_item`, `split_at`,
`delete_items`, `set_keyframes` (transform properties) and `apply_animation_preset` work on it.

**Checks.** `create_motion_template` and `update_motion_template` send the module to the editor
window, which compiles it (TypeScript types removed, `import`/`require` refused), loads it (the
three exports present), renders the instants 0, entrance, middle, exit and end with the default
values twice (each frame under **50 ms**; a frame still running after 1 s is stopped), compares
both renders (determinism) and draws three pictures. The core then reads `meta` and `params`
strictly. Any problem refuses the module with `TEMPLATE_INVALID`, listing every problem as
`- [stage] line L, column C (at T s): message` (stages `compile`, `exports`, `render`, `budget`,
`determinism`, `definition`):

```json
{"error":{"code":"TEMPLATE_INVALID","message":"the motion template cannot be used; fix these problems and send the whole module again:\n- [render] line 9, column 3, at 0.000 s: ReferenceError: fetch is not available in motion templates: …\n- [budget] at 1.500 s: the frame took longer than 1000 ms (an endless loop?): the template was stopped"}}
```

A refused module never enters the project. Without an editor window: `RENDERER_UNAVAILABLE`.

**Versions and scope.** Creating a template needs no particular scope. A template may change in
place only when every instance (before and after) lies in the task's scope; otherwise
`update_motion_template` copies it (new id) for the instances in scope and the others keep the
old version. Instance values are kept by parameter name; a value whose parameter is gone, changed
type or is outside the new range falls back to the new default (`reset`).

**Parameter values** (`instantiate_template`, `update_template_params`), by name: a string for
`string`, `enum` (one of the options), `font` (a family) and `color` (`#rrggbb[aa]`); a number for
`number` (inside its range); `true`/`false` for `boolean`; an image `asset_id` (or `null`) for
`asset`. Numbers and colours also take keyframes:
`{"keyframes":[{"time":0,"value":0},{"time":0.6,"value":1,"easing":"ease_out"}]}` (times relative
to the item's start, as `set_keyframes`). Unknown names or wrong types: `INVALID_ARGUMENTS`
naming the parameter; values outside a range: `VALIDATION_ERROR`.

**Brand kits and the library** (prompt 25). `meta.category` (optional: `title`, `lowerThird`,
`callToAction`, `transition`, `other`) files the template in the user's browser. A colour, font
or asset parameter may declare `brand` — `primary`, `secondary`, `accent`, `text`, `background`
(colours), `heading`, `body` (fonts), `logo` (assets): new instances in a project with a brand kit
start with the kit's value instead of the default (values given explicitly win), so one template
fits every brand. The user saves templates to their library and inserts them in other projects;
agents insert library templates with `insert_library_template`. A project template copied from
the library and changed by an agent keeps its link: the user may save it back as a new library
version.

### `create_motion_template`

**Input**: `source` (required): the whole module.

**Output**, then the three pictures (entrance, middle, exit, default values; each preceded by a
caption line such as `Frame 1 (entrance) at 00:00:00.400`):
```json
{"revision":61,
 "template":{"template_id":"…","name":"Lower third","version":1,"default_duration":{"timecode":"00:00:05.000","seconds":5.0},"params":[…],"instances":0},
 "slowest_frame_ms":3.2,
 "frames":[{"timecode":"00:00:00.400","seconds":0.4},{"timecode":"00:00:02.500","seconds":2.5},{"timecode":"00:00:04.600","seconds":4.6}],
 "next":"Look at the pictures; then place it with instantiate_template and check the result with get_frames."}
```
Frame times are relative to an instance's start. Not batchable. **Errors**: `TEMPLATE_INVALID`,
`RENDERER_UNAVAILABLE`, `VALIDATION_ERROR`.

### `update_motion_template`

**Input**: `template_id` (required), `source` (required: the whole new module; read the current
one with `list_templates {"template_id"}`).

**Output**: as `create_motion_template` (with the new `version`), plus `instances` (items now
rendering the new version), `copied_from` (the old template's id, when instances outside the
scope kept it; absent when changed in place) and `reset` (`[{"item_id","params":["size"]}]`,
values that fell back to the new defaults; absent when none), then the three pictures. Not
batchable. **Errors**: `TEMPLATE_NOT_FOUND`, `TEMPLATE_INVALID`, `RENDERER_UNAVAILABLE`,
`OUT_OF_SCOPE`, `VALIDATION_ERROR`.

### `instantiate_template`

| Argument | Type | Default |
|---|---|---|
| `template_id` | string | required |
| `start` | time | required |
| `end` / `duration` | time | the template's default length (give one at most) |
| `params` | object | brand kit values for parameters bound to a role, defaults for the others |
| `track` | id or name of a video track | the highest free track of the task |
| `position` | `{x, y}` or a named place | `center` (the template draws over the whole frame: keep it unless the whole graphic moves) |
| `scale`, `opacity` | number | 1 |
| `animation_in`, `animation_out` | `{preset, duration?}` | none (templates usually animate themselves) |

Batchable (`"as"` names the instance for later actions). **Output**: as other write tools.
**Errors**: `TEMPLATE_NOT_FOUND`, `INVALID_ARGUMENTS`, `VALIDATION_ERROR`, `OUT_OF_SCOPE`,
`INVALID_TIME`.

### `update_template_params`

`item_id` (required; `$name` in a batch), `params` (required: values by name, `null` resets one
to its default; others unchanged), `expected_revision`. The template's code is unchanged. Batchable.
**Output**: as other write tools. **Errors**: `INVALID_ARGUMENTS` (not a template instance,
unknown parameter, wrong type), `VALIDATION_ERROR`, `REVISION_CONFLICT`, `ITEM_NOT_FOUND`,
`OUT_OF_SCOPE`.

### `insert_library_template`

Places a motion template of the user's library (prompt 25): the template is copied into the
project (a copy of the same library version already there is reused) and an instance is placed.

| Argument | Type | Default |
|---|---|---|
| `template_id` | a `library_template_id` (`list_templates` with `source: "library"`) | required |
| `start` | time | required |
| `end` / `duration` | time | the template's default length (give one at most) |
| `params` | object | brand kit values for parameters bound to a role, defaults for the others |
| `track` | id or name of a video track | the highest free track of the task |
| `animation_in`, `animation_out` | `{preset, duration?}` | none |

Not batchable (the library is read first). **Output**: as other write tools (`created` holds the
instance; `get_item` shows its project `template_id`). **Errors**: `LIBRARY_TEMPLATE_NOT_FOUND`,
`INVALID_ARGUMENTS` (malformed id, a caption style entry, unknown parameter), `VALIDATION_ERROR`,
`OUT_OF_SCOPE`, `INVALID_TIME`.

## Automatic reframing (prompt 26)

A media clip that fills the frame (`reframe` `fill` or `tracked`) keeps the part of its picture
around a **focus** (`reframeFocus`, a point of the picture normalised in it, animatable). Automatic
reframing writes that focus for you: the faces of the clip's media are found locally (a face
detector run once per media, at 5 pictures per second; the result is kept), then the window
follows the main subject — the largest, steadiest, most central face, or the group when the
significant faces fit in the frame together — keeping the eyes on the upper third line, holding
still while the subject moves a little (a camera operator's dead zone), moving smoothly, and
jumping to the next shot's subject exactly on each shot change. The result is `reframe:
{kind: "tracked", focus: keyframes, tracking: settings}`: ordinary keyframes (eased curves, a
hold before each cut) that `set_keyframes` (property `reframeFocus`) and the user can edit. A
media without faces stays centred.

### `auto_reframe`

| Argument | Type | Default |
|---|---|---|
| `item_ids` | ids of media clips on video tracks | every media clip of a video track lying in the task's scope (ignored with `target_format`) |
| `target_format` | `"9:16"`, `"16:9"`, `"1:1"`, `"4:5"` or `{width, height}` | none: reframe clips of the task's sequence |
| `name` | string | the sequence's name and the format (`Interview 9:16`) |
| `smoothing` | number 0–1 | 0.5 (0 follows closely, 1 slow and steady) |
| `wait_seconds` | number | 90 (at most 110; 0 starts the analyses and answers at once) |

With `target_format`, a version of the task's sequence is added after it in that format, at the
sequence's frame rate (texts wider than the new frame are narrowed, every clip fills the frame and
follows its subject): the scope's one exception (see **Scope rule**). Not batchable.

**Output**:
```json
{"finished":true,"version_sequence_id":"…",
 "edit":{"revision":64,"created":["…"],"items":[{"item_id":"…","kind":"media_clip","track":"V1","start":{"timecode":"00:00:00.000","seconds":0.0},"end":{"timecode":"00:00:10.000","seconds":10.0},"revision":1}]},
 "note":"the version was created; its clips follow their subject (reframe tracked, editable reframeFocus keyframes): check with get_frames"}
```
While faces are still being analysed: `{"finished":false,"analysing":["interview.mp4"],"note":"…"}`
— nothing changed; call again with the same arguments. Media whose analysis failed are listed in
`failed` (`asset_id`, `name`, `reason`); their clips stay as they are (centred in a version).

**Errors**: `VISION_UNAVAILABLE`, `INVALID_ARGUMENTS` (unknown format, odd size, smoothing out of
range, empty `item_ids`), `VALIDATION_ERROR` (a clip that cannot follow a subject: not a media clip
of a video track, no picture, locked track), `OUT_OF_SCOPE`, `ITEM_NOT_FOUND`.

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
