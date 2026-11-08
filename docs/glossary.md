# Glossary

Use these terms consistently in code, docs, UI keys and agent instructions. Add a term here before introducing a new domain concept.

| Term | Meaning | Code name |
|---|---|---|
| **Project** | A folder `<Name>.seproj/` containing one document, its database and caches | `Project` |
| **Document** | The serialized state of a project (`project.json`) | `Project` |
| **Project folder** | A `<Name>.seproj/` folder: `project.json`, its backups, `project.db`, `.lock`, `cache/`, `exports/` | `ProjectLayout` |
| **Snapshot (file)** | The document saved in `project.json`; written 2 s after the last commit and on close, keeping 3 backups (`project.json.bak.N`) | `ProjectStore::save_snapshot` |
| **Journal** | Transactions committed since the last snapshot, in `project.db` (`op_journal`), replayed after a crash | `JournalEntry` |
| **Project lock** | The OS file lock on `.lock` held while a project is open; released by the OS if the process dies | `ProjectLock` |
| **Project store (core)** | One open, locked project folder, owned by its actor | `ProjectStore` |
| **Project session** | What a client gets for an open project: its path, snapshot of entities and undo state | `ProjectSession` |
| **Project snapshot (protocol)** | Every entity of an open project at one revision, normalised by id | `ProjectSnapshot` |
| **Entity** | A whole sequence, track, item, marker or asset as sent to clients, with its parent and children ids | `SequenceEntity`, `TrackEntity`, `ItemEntity`… |
| **Patch** | The entities a commit added or changed (whole) and the ids it removed, for one revision | `ProjectPatch` |
| **Project mirror** | The renderer's read-only copy of the open project, kept current by patches | `ProjectMirror`, `shared/project-store` |
| **Resync** | Reloading the snapshot after a missed patch, a dropped-events notice or a reconnection | `ProjectSync::resync` |
| **Recent projects** | The home screen's list of projects, most recently opened first, kept in `library.db` | `RecentProject`, `RecentProjectRecord` |
| **App library** | The app's own data shared by all projects (`library.db`: recent projects, settings) | `AppLibraryRepository` |
| **Temporary actor** | An actor started to change a closed project (rename) and closed right after; its events are discarded | `ProjectService::with_project` |
| **Revision** | Monotonic counter bumped once per committed transaction: for the project, and for each item the transaction touched | `revision` |
| **Asset** | A media file imported into a project (video, audio, image), referenced by path | `Asset` |
| **Media info** | What probing an asset revealed: duration, container, video and audio streams | `MediaInfo` |
| **Decodability** | Whether the renderer of this machine can decode each stream (picture, sound) of an asset's original with WebCodecs, as it reported: `unknown`, `decodable`, `undecodable` (needs a mezzanine) or `noStream` | `Decodability`, `StreamDecodability` |
| **Content fingerprint** | Quick identity of a file's content: size, modification time and xxh3 of its first and last MiB; same size and hash = same content (deduplication, relinking) | `ContentFingerprint` |
| **Cache key** | Name of an asset's cache folder, `<hash>-<size>-<mtime>` from its fingerprint | `ContentFingerprint::cache_key` |
| **Media cache** | `<project>/cache/media/<cache key>/`: regenerable files of a content (thumbnails, peaks, mezzanine, proxy, extracted frames); thumbnails and peaks are not recorded in the document, mezzanines and proxies are | `media_cache_dir`, `se_media::cache_layout` |
| **Missing media** | An asset whose original was found neither at its path nor through its relative hint when the project opened | `Asset::missing` |
| **Relink** | Pointing an asset at another file (a moved or renamed original); other content needs the user's confirmation | `relinkAsset`, `MediaService::relink_asset` |
| **Relative hint** | The original's path relative to the project folder, used to find media moved together with their project | `Asset::relative_hint` |
| **Thumbnail sheet** | One JPEG holding up to 40 small pictures (90 px high) sampled along a video, 8 per row, with its JSON layout and source times; hovering a card scrubs through it | `ThumbnailSheet` |
| **Poster** | One larger picture (up to 360 px high) representing an asset in the media bin | `poster.jpg` |
| **Waveform peaks** | Minimum and maximum sample values per channel over blocks of 256, 1024 or 4096 samples, in `SEPK` files | `Peaks`, `WaveformPeaks` |
| **Job** | A background task of the core (thumbnails, waveform peaks, mezzanine, proxy, audio analysis, scene detection; later transcription…) with a kind, a priority, progress and one automatic retry; at most 4 run at once | `JobQueue`, `JobSnapshot` |
| **Sidecar** | A tool run as a separate process (ffmpeg, ffprobe, later whisper), stopped with its whole process tree when cancelled | `se_media::process::Sidecar` |
| **System transaction** | A transaction recording facts the app observed (missing media, decodability), journaled but never undone | `Actor::System` |
| **Original** | The user's source file of an asset | `Asset::original_path` |
| **Mezzanine** | High-quality, WebCodecs-decodable copy of an asset this machine cannot decode (codec, or a layout outside 4:2:0 8–10 bits): HEVC Main 10 for deeper footage when decodable, H.264 High otherwise; MP4, M4A or PNG in the content's cache folder | `DerivedMedia::mezzanine` |
| **Mezzanine policy** | The pure decision of whether an asset needs a mezzanine and how to make it, from its streams, the decodability reported and this machine's decoders | `media_policy::mezzanine_decision` |
| **Proxy** | Low-resolution H.264 copy of a video (540p or 720p, fast decode, a keyframe every 15 frames) for smooth playback; automatic beyond 1080p or for undecodable footage, or on request | `DerivedMedia::proxy`, `media_policy::proxy_recipe` |
| **Derived copy** | A mezzanine or a proxy: recorded in the document with its state (pending, ready, failed + reason), the file in the cache | `DerivedFile`, `DerivedKind` |
| **Transcode recipe** | How a derived copy is made: container, picture plan (drop, copy, encode, still) and sound plan (drop, copy, AAC, FLAC) | `TranscodeRecipe`, `TranscodePlan` |
| **Decoder capabilities** | What this machine's WebCodecs decodes among the codecs of derived copies (HEVC Main 10, H.264 High, AAC, FLAC), reported by the renderer and kept in `library.db` | `DecoderCapabilities` |
| **Hardware encoder** | A GPU encoder ffmpeg drives (NVENC, VideoToolbox, VAAPI), detected once by a test encode; jobs fall back to libx264/libx265 | `HardwareEncoder`, `EncoderSupport` |
| **Playback source** | Which file the engine decodes for an asset: the original, its mezzanine, nothing yet (« en préparation ») or nothing (missing, mezzanine failed) | `playbackSource` |
| **Audio analysis** | Loudness (EBU R128 integrated, range, short-term curve), peaks and silences of an asset's first sound stream | `AudioAnalysis`, `AudioSummary` |
| **Silence** | A stretch where the RMS level of 20 ms windows stays under a threshold (−40 dBFS) for a minimum duration (0.5 s) | `SilenceSettings`, `SilenceDetector` |
| **LUFS / dBTP** | Loudness units relative to full scale (EBU R128) / true peak in decibels, measured on oversampled sound | `integrated_lufs`, `true_peak_dbtp` |
| **Scene detection** | The instants where an asset's picture changes shot (ffmpeg's scene score above 0.3) | `SceneDetection`, `SceneSummary` |
| **Analysis summary** | The small part of an analysis kept in the asset (loudness, peak, silence count, shot changes); the full result is in `project.db` | `AssetAnalysis` |
| **Frame extraction** | Writing one picture of an asset at a time as a JPEG (cached), for AI tools that look at footage | `MediaService::extract_frame`, `FrameExtractor` |
| **Media settings** | App-wide preferences of the media pipeline: mezzanine quality (CRF), proxy resolution, automatic proxies, ffmpeg folder | `MediaSettings` |
| **Sequence** | A timeline with a format (resolution, frame rate, sample rate) and tracks. A project may contain several | `Sequence` |
| **Format** | Width, height, frame rate and sample rate of a sequence | `Format` |
| **Format preset** | A named format size: 16:9 HD and UHD, 9:16, 1:1, 4:5, or custom | `FormatPreset` |
| **Frame rate** | Frames per second as a reduced fraction (30000/1001); only rates with a whole number of flicks per frame exist | `FrameRate` |
| **Nominal frame rate** | The frame rate rounded up (30 for 29.97): frames per second counted by SMPTE timecodes | `FrameRate::nominal_fps` |
| **Track** | A horizontal lane of a sequence holding non-overlapping items, sorted by start; kind video or audio | `Track` |
| **Locked track** | A track whose items cannot be added, changed or removed, and which cannot be removed; its header stays editable | `Track::locked` |
| **Item** | Anything placed on a track over a time range | `Item` |
| **Link group** | Items that move and trim together, like a clip's picture and its sound | `LinkGroupId` |
| **Marker** | A labelled instant of a sequence, with one of Premiere's eight colours | `Marker` |
| **Media clip** | An item showing/playing part of an asset | `ItemKind::MediaClip` |
| **Text / Shape / Image item** | Declarative graphic items | `ItemKind::Text`, `Shape`, `Image` |
| **Motion template** | AI-authored, parameterised render module (code + params schema) | `MotionTemplate` |
| **Template item** | An item instantiating a motion template with parameter values | `ItemKind::MotionTemplate`, `TemplateItem` |
| **Caption** | Subtitle item built from transcript words, styled by a caption style | `ItemKind::Caption` |
| **Brand kit** | Fonts, colours, logo and default caption style reused by the user and agents | `BrandKit` |
| **Gap** | Empty time on a track before an item: from the previous item's end (or 0) to its start; computed, never stored. Selected by a click, resized with its handle, closed with `Suppr` | `Gap`, `gapAt`, `resizeGap` |
| **Source range** | The part of an asset used by a media clip (`source_start` + duration) | `source_start` |
| **Timeline range** | Where an item sits in its sequence (`start` + `duration`) | `timeline_range` |
| **Time range** | A half-open interval `[start, start + duration)` with a non-negative duration | `TimeRange` |
| **Flicks** | Time unit: 1/705,600,000 s, stored as i64 | `Flicks` |
| **Timecode** | Human representation of a time (`00:00:51:12`, `51s`, `1:02.5`) | `parse_timecode`, `format_timecode` |
| **Test vectors** | JSON input → expected output tables in `docs/test-vectors/`, run by both the Rust and the TypeScript tests | — |
| **Keyframe** | A value of an animatable property at an item-relative time, with an easing to the next | `Keyframe<T>` |
| **Animatable** | A property that is either static or keyframed | `Animatable<T>` |
| **Easing** | Interpolation curve between keyframes | `Easing` |
| **Animation preset** | Named, reusable set of keyframes (fade in, slide, pop, typewriter…) | `AnimationPreset` |
| **Reframe** | How a clip's picture fits the sequence frame before its transform: `fit` (whole picture, bands), `fill` (cropped to the frame around a focus), `custom` (natural size, one picture pixel per design pixel, framed by the transform), `tracked` (a fill whose focus follows the subject, prompt 26) | `Reframe`, `reframedPicture` |
| **Reframe focus** | The point of a clip's picture (normalised in it) a fill keeps at the frame's centre as far as the picture allows; animatable, independent of the frame's aspect | `AnimatableProperty::ReframeFocus`, `fillCrop` |
| **Crop (picture)** | The part of an upright picture a filling clip shows, normalised in it; the compositor maps it to the decoded pixels | `Crop`, `VideoLayer.crop`, `codedCrop` |
| **Format adaptation** | How items follow a sequence into another format: the clips' reframe policy (keep, fit, fill) and narrowing texts wider than 90 % of the new frame | `FormatAdaptation`, `ClipReframePolicy`, `adapted_patch` |
| **Version (sequence)** | An independent copy of a sequence in another format (« Créer une version… »), with fresh ids and adapted items; later edits of one never reach the other | `sequence_version`, `duplicateSequence` |
| **Out-of-frame warning** | After a format change or a version, the items that left the frame although they were inside before, with « Sélectionner » | `itemsLeavingFrame` |
| **Reframing mode** | The viewer state where dragging moves a filling clip's picture under the frame (its whole picture outlined, its thumbnail dimmed around) | `reframeModeStore`, `ReframeOverlay` |
| **Transform** | Position, scale, rotation, anchor and opacity of a visual item, each animatable | `Transform2D` |
| **Normalised position** | A point in the sequence frame where `(0, 0)` is the top-left and `(1, 1)` the bottom-right corner, whatever the format | `Vec2` |
| **Design pixel** | Size unit: a pixel of a frame whose shorter side is 1080, so 1 design px = 1 px in 1080-class formats, 2 px in 4K (ADR-0015) | `DesignPixels` |
| **Safe area** | Zones a platform's interface covers (TikTok, Instagram Reels, YouTube Shorts) or broadcast title/action safe areas (YouTube 16:9); data with source and date, drawn by the viewer per sequence, never exported | `@se/engine/safe-areas`, `SafeAreaPlatform`, `FrameGuides` |
| **Frame guides** | What the viewer can draw over the picture: a platform's safe areas, a rule-of-thirds grid, a centre cross (machine preferences per sequence) | `ViewerGuides`, `GuidesMenu` |
| **Schema version** | Version of the `project.json` format a document was written with | `schema_version`, `CURRENT_SCHEMA_VERSION` |
| **Violation** | A broken document invariant found by validation, with a stable code, a path and a message | `Violation`, `ViolationCode` |
| **Operation** | One intent-level edit of the document, applied by the pure `apply`, with exact inverse operations | `Operation`, `apply` |
| **Inverse** | The operations that undo an edit, computed when it is applied (`RestoreItem` snapshots for item edits) | `Applied::inverse`, `Committed::inverse` |
| **Touched entities** | Ids of the items, tracks, sequences, markers and assets an edit added, changed or removed | `TouchedEntities` |
| **Item patch** | A typed change of an item's content, one patch type per item kind; `null` fields stay unchanged | `ItemPatch` |
| **Animatable property** | A property keyframes can drive: placement, opacity and blur of visual items, a clip's volume, the size, colours, spacing, reveal, outline, shadow and plate colour of texts, the fill and outline of shapes (17 in all), set with its keyframes or static value | `AnimatableProperty`, `PropertyAnimation`, `ItemKind::property` |
| **Property slot** | Typed access to one animatable property of an item (number, design pixels, pair or colour), shared by keyframe edits, presets and validation | `PropertyRef`, `PropertyMut` |
| **Animation preset** | A named animation (fade, slides, pop, blur, typewriter, bounce, shake, pulse) defined as data and expanded into ordinary keyframes of an item | `AnimationPreset`, `assets/animation-presets.json` |
| **Preset phase** | Where a preset plays: `in` (entrance, at the start), `out` (exit, at the end; the entrance played backwards) or `emphasis` (inside the item, returning to its values) | `AnimationPhase` |
| **Preset base** | The property's own value a preset animates to or from, read as a hold of existing keyframes at the far end of the span | `expand_preset` |
| **Preset sample** | A preset's expansion on a reference « Aa » text over one second, sent to clients for previews | `AnimationPresetSample` |
| **Reveal** | Share of a text's characters shown, in reading order, laid out as the whole text (the typewriter effect) | `TextItem::reveal` |
| **Stroke (outline)** | An outline around text glyphs or a shape, of a visible width outside them; kept when disabled | `Stroke` |
| **Text plate** | A rounded plate behind a text block; the item's box includes it | `TextBackground` |
| **Cover frame** | A shape whose box is the whole sequence frame whatever its format | `ShapeItem::cover_frame` |
| **Image fit** | An image's natural box: fitted into the frame, covering it, or one image pixel per design pixel | `ImageFit` |
| **Font catalogue** | The fonts texts may use: bundled with the app (embedded in se-core) and installed on the machine, each face served by `/media/fonts/{id}` | `FontCatalog`, `listFonts`, `se-fonts` |
| **Font registry** | The engine's loader of catalogue faces (`FontFace`) before texts are drawn; reports families the catalogue lacks | `FontRegistry` |
| **Frame layout** | Where each layer of a drawn frame sits (resolved transform and box): the viewer's handles and hit tests | `FrameLayout`, `LayerLayout` |
| **Transform gizmo** | The handles over the viewer's picture to move, scale, rotate and re-anchor selected items, with snapping to the frame | `TransformGizmo` |
| **Property preview** | The values of a continuous edit in progress, drawn by the viewer until its one command resolves | `propertyPreviewStore` |
| **Keyframe lane** | The rows of keyframes unfolded under an item in the timeline (select, drag, delete, easing) | `KeyframeLane`, `keyframeLaneStore` |
| **Auto-keyframing** | Editing a keyframed property at the playhead changes or adds the keyframe there | `withValueAt` |
| **Trim** | Moving the start or end edge of an item; a media clip keeps playing the same source at the same instants | `TrimItem`, `ItemEdge` |
| **Split** | Cutting an item in two at an instant; the right part is a new item | `SplitItem` |
| **Ripple delete** | Deleting a time range on tracks and moving what follows left to close the gap | `RippleDeleteRange` |
| **Insert gap** | Opening empty time on tracks, moving what follows right | `InsertGap` |
| **Split ids** | Ids given by the caller for the new right parts of items a ripple delete or inserted gap cuts | `splitIds` |
| **Transaction** | Atomic group of operations by one actor | `Transaction` |
| **Actor** | Who performed a transaction: the user or an AI task | `Actor` |
| **Origin** | Who created an item (user or AI task), kept for traceability | `Origin` |
| **History** | The user's undo/redo stack | `History` |
| **Changeset** | All transactions of one AI task, revertable as a unit; owned by the project's actor, stored in `project.db` with each journal entry, closed when the task is reverted | `Changeset`, `Task.changesetId` |
| **Conflict** | A part of a task's changes a revert leaves in place (item changed since, items the task never touched, inverse no longer applies); for a write, a stale `expectedRevision` | `Conflict`, `REVISION_CONFLICT` |
| **Revert plan** | The inverse operations of a changeset that still apply to the current document, with the conflicts | `RevertPlan`, `plan_revert` |
| **Domain error** | Why the domain refused a change: an actionable message and a stable code | `DomainError`, `DomainErrorCode` |
| **Job** | A background processing unit (thumbnails, peaks, mezzanine, transcription…) | `Job` |
| **Analysis** | Derived data about an asset: silences, loudness, scene changes, faces | `Analysis` |
| **Transcript** | Word-level timed text of an asset's speech | `Transcript` |
| **AI provider** | An adapter for one AI CLI (Claude Code first), describing itself to the UI (name, command, login command, help page) | `AgentProvider`, `ProviderInfo` |
| **Task** | One user request to an AI, with a prompt, a scope and one or more runs; its status is its latest run's (`queued → running → completed / failed / cancelled`) | `Task`, `TaskStatus` |
| **Run** | One execution of an AI CLI process for a task: the initial run, a retry or a follow-up | `TaskRun`, `RunKind`, `AgentRun` |
| **Task service** | The scheduler of AI tasks: one FIFO queue for every open project, a limit of runs at once (1–8, default 3), cancellation, retries, follow-ups, persistence (ADR-0027) | `TaskService` |
| **Follow-up** | A new prompt sent to a finished task's agent, resuming its session; it may change the task's scope (same sequence) | `followUpTask`, `RunKind::FollowUp` |
| **Retry** | The last prompt of a failed or cancelled task run again (resuming its session once an earlier run completed) | `retryTask`, `RunKind::Retry` |
| **Interrupted task** | A task a stopped core left queued or running, failed with `interrupted` when its project reopens | `AgentFailureKind::Interrupted` |
| **Task event log** | The events of a task's runs, numbered and stored in `project.db`, published as `taskEventAppended` | `TaskEventRecord`, `TaskStore` |
| **Agent event** | What an agent run reports: session started, text, tool calls, usage, then completed or failed | `AgentEvent` |
| **Agent workspace** | The empty working folder of a task's runs, `<app data>/agent-workdirs/<task id>/` | `AgentWorkspaces`, `FsAgentWorkspaces` |
| **AI status** | What is known about the AI provider: installed (path, version), logged in, capabilities | `AiStatus` |
| **Replay provider** | Test provider replaying scripted agent runs with simulated delays | `ReplayProvider`, `ReplayScript` |
| **Scope** | The part of the project a task may edit: sequence, time range, tracks | `TaskScope` |
| **Scope check** | Every entity an agent's transaction touches must lie in the task's scope before and after it (items entirely inside the range and on the task's tracks); otherwise `OUT_OF_SCOPE` | `agent_edits::scope`, `OUT_OF_SCOPE` |
| **Agent edit** | One change an agent asks for (add a text, move an item…), typed in flicks and ids; one call's edits form one AI transaction | `AgentEdit`, `AgentEditService` |
| **Write tool** | An MCP tool that changes the project through `AgentEditService` (annotated destructive) | `WriteTool`, `writes::run` |
| **Batch** | Several write actions applied as one transaction, all or nothing, with `"as"` names referred to as `"$name"` | `apply_batch`, `ItemRef::Created` |
| **Revert (task)** | Undoing an AI task's changeset against the current document as one undoable user transaction, reporting conflicts | `revertTask`, `TaskRevertReport`, `REVERT_TASK_LABEL` |
| **Revert preview** | What reverting a task would undo and keep now, planned and committed on a copy (nothing changes); the panel lists its conflicts before the user confirms | `previewTaskRevert`, `ProjectActorHandle::preview_revert` |
| **Task usage** | Tokens and cost of a task's runs, added up from their `usage` events | `TaskUsage`, `Task.usage` |
| **Patch actor** | Who made the commit a `projectPatched` reports: the user (edits, undo, redo, reverts), an AI task, or the app | `ProjectPatch.actor`, `Actor` |
| **Composer** | The prompt box at the bottom of the « IA » panel: scope chips, the prompt (Ctrl/⌘+Entrée sends, ↑ recalls earlier prompts), model and effort | `TaskComposer`, `PromptComposer`, `composerStore` |
| **Scope chip** | A chip of the composer saying one part of the prompt's scope and where it comes from (text, item, selection, in/out, whole sequence; tracks; sequence) | `useScopeChips`, `PromptScope` |
| **Time phrase** | A time range written in French in a prompt (« de 51 s à 54 s », « les 10 premières secondes »), read into the composer's scope | `findTimePhrase`, `TimePhrase` |
| **Task number** | « #3 »: a task's rank in its project, oldest first (tasks are never deleted, so it never changes) | `taskNumbers`, `useTaskNumber` |
| **Redo with AI** | « Refaire avec l'IA »: a prompt about one timeline item, with its range and track as scope, resuming (by default) the session of the task that made it, the scope widened to the item if it moved | `openComposer`, `RedoTarget`, `widenedScope` |
| **AI task mirror** | The renderer's read-only copy of the open project's tasks with their numbers, pulses and user-edit flags, fed by the AI feature and read by the timeline | `shared/ai-tasks`, `connectTaskSync` |
| **AI pulse** | The brief glow of a timeline item an AI patch just created or changed | `pulseItems`, `animate-ai-pulse` |
| **Agent session** | The AI CLI's conversation id, chosen by the app for a new run (`--session-id`) and resumed by follow-ups | `Task.agentSessionId`, `AgentSession` |
| **Tool** | An MCP function exposed to agents (read or write) | `tools::*` |
| **Agent instructions** | Provider-agnostic markdown guidance (role, time and scope, reading the project, editing rules, motion basics, audio editing) composed with the task's scope and a project summary into the agent's system prompt | `agent-instructions/`, `tasks::instructions` |
| **MCP server** | se-core's `/mcp` endpoint (rmcp, streamable HTTP): the agents' only way to read and change a project (ADR-0028) | `se_mcp::McpServer`, `mcp_service` |
| **Task token** | The 256-bit bearer token of one run, minted when it starts and revoked when it ends; `/mcp` refuses any other | `TaskToken`, `TaskTokenRegistry` |
| **Agent context** | Who calls an MCP tool: the run's project, task, run index and scope, resolved from its token | `AgentContext` |
| **Tool** | One MCP capability with an input schema, a handler and a section in `docs/ai-tools.md`; read tools change nothing | `McpTool`, `ToolEntry` |
| **Tool error** | A failed tool call returned to the agent as a result flagged `isError`, with a stable code and an actionable message | `ToolError` |
| **Time output** | How tools give times: `{"timecode": "HH:MM:SS.mmm", "seconds": n}` | `TimeOutput`, `format_clock` |
| **Time base** | Whether times count from the sequence's first frame (`sequence`) or the media file's (`source`) | `time_base` |
| **Frame render request** | Frames of a sequence an agent asked for, drawn by the renderer and uploaded as JPEGs (`renderFramesRequested`, `PUT /media/renders/…`) | `FrameRenderRequest`, `FrameRenderService` |
| **Renderer link** | How the core reaches the renderer: connected while an `/rpc` socket is open | `RendererLink` |
| **Still renderer** | The engine drawing single frames offscreen and encoding them as JPEG, through the viewer's frame renderer and compositor | `StillRenderer`, `createStillRenderer` |
| **Frame graph** | Everything visible in a sequence at one instant: layers bottom first, transforms resolved, sizes in sequence pixels; the pure output of `evaluate` | `FrameGraph` |
| **Layer** | One drawable item in a frame graph: `video`, `image`, `shape` or `text` (captions draw as text until prompt 23) | `Layer` |
| **Natural box** | The box a picture fills at scale 1: the asset's upright display size fitted into the frame, the frame itself for a filling clip (which shows a crop), or the natural size in design pixels (`custom` clips, `natural` images); independent of the decoded variant | `VideoLayer.box`, `reframedPicture` |
| **Media variant** | The file of an asset the engine decodes: original, mezzanine or proxy | `MediaVariant`, `chooseVariant` |
| **Media locator** | Gives the engine the URL of an asset's variant (se-core's `/media`, with the token and a content version) | `MediaLocator`, `createMediaLocator` |
| **Decoder pool** | Owns the engine's decoders (one per asset and variant) and every decoded picture, with an LRU cache and cancellation of obsolete requests | `DecoderPool` |
| **Picture lease** | A borrowed decoded picture, valid until released; the pool never closes a leased picture | `PictureLease` |
| **Frame lookup time** | The media time (seconds) at which a source frame is requested: the middle of the frame computed exactly from the asset's frame rate | `frameLookupTime` |
| **Open picture count** | `VideoFrame`s and `ImageBitmap`s the engine opened and has not closed: the leak check (zero after destroy) | `FrameTracker`, `openPictureCounts` |
| **Media issue** | Why a picture layer was drawn without its media (preparing, missing, mezzanine failed, decode failed), reported per asset | `MediaIssue` |
| **Compositor** | The component drawing a frame graph to a canvas (PixiJS today, behind an interface) | `Compositor`, `createPixiCompositor` |
| **Preview engine** | The viewer's engine outside React: scene, time and options in, one coalesced draw per animation frame, a state out | `PreviewEngine`, `ViewerEngineController` |
| **Master clock** | The clock playback follows: the audio context time of the sample being heard (pictures follow the sound) | `audioContextClock`, `PlaybackClock` |
| **Transport** | Where playback stands: an anchor (sequence time at a clock time) and a rate; every position is derived from it, never accumulated | `Transport`, `transportPosition` |
| **Shuttle (JKL)** | J plays in reverse, K pauses, L plays forward; repeated presses go 1×, 2×, 4×; K held with J or L steps one frame | `nextShuttleRate`, `shuttle*` actions |
| **Playback controller** | Drives playback on each display refresh: reads the master clock, reports the frame to show, skips late frames, loops, stops at the edges | `PlaybackController` |
| **Presentation lead** | Playback picks the frame for the next display refresh (the measured refresh period ahead), when what is drawn now appears | `RefreshPeriod` |
| **Dropped frame** | A frame playback skipped because it was not drawn in time (beyond those a fast speed skips anyway) | `skippedFrames`, `PlaybackStatsCounter` |
| **Sequential reader** | A decoder running through a file for playback, kept ahead of the playhead and fed by prefetch requests | `FrameReader`, `DecoderPool.prefetch` |
| **Read-ahead budget** | Frames and bytes a sequential reader may hold ahead of the playhead (its share of the picture cache) | `ReadAheadBudget` |
| **Prefetch window** | How far ahead playback decodes, adapted to late frames (0.25–2 s) | `PrefetchWindow`, `prefetchSpans` |
| **Audio chunk** | One second of an asset's decoded sound in one `AudioBuffer`, joined to its neighbours on the sample | `AudioChunk`, `AudioChunkPool` |
| **Audio piece** | Part of a chunk of a clip started as one voice at a context time | `AudioPiece`, `planPieces` |
| **Anticipation** | How far ahead of the audio render edge sound is scheduled (200 ms) | `SCHEDULE_AHEAD_SECONDS` |
| **Audio scrub** | Short snippets of sound played when the paused playhead moves | `AudioScheduler.scrub` |
| **Level meter** | Peak and RMS per channel of the master, measured in an AudioWorklet and shown in dBFS | `AudioMeter`, `Meter` |
| **Playback session** | The viewer's playback outside React: controller, audio engine, playhead store, picture engine | `PlaybackSession` |
| **Performance HUD** | The viewer overlay with real frames per second, dropped frames, decoding work and caches | `PerformanceHud` |
| **Golden frame** | A reference PNG a rendered test scene must match within a tolerance (`packages/engine/test/golden/references/`) | `expectGolden` |
| **Premiere package** | Export folder with FCP7 XML, OTIO, overlay renders and SRT | — |
| **Core** | The Rust process `se-core` | — |
| **Renderer** | The Electron renderer process (React UI + engine) | — |
| **Handshake** | The single JSON line (`port`, `token`, `version`) the core prints on stdout once it accepts connections | `CoreHandshake` |
| **Session token** | 256-bit random secret of one core run, required by every request (`?t=`); never logged | `SessionToken` |
| **Core supervisor** | Electron main component that spawns, restarts (with backoff) and stops the core | `CoreSupervisor` |
| **Bridge** | The minimal typed API the preload script exposes to the renderer as `window.simpleEffect` | `SimpleEffectBridge` |
| **Command** | A request sent to se-core over `/rpc`; answered by the command result of the same kind | `Command`, `CommandResult` |
| **Core event** | A message pushed by se-core to every connected client | `CoreEvent` |
| **Resync** | Refetching state after missed events or a reconnection | `CoreEvent::ResyncRequired` |
| **Core client** | The renderer's typed connection to se-core (requests, events, reconnection) | `CoreClient` |
| **App data folder** | Per-user folder shared by the app and se-core (logs, later settings and library) | `SE_DATA_DIR`, `AppPaths` |
| **Design token** | A named design value (colour, size, radius, duration…) defined once as a CSS variable in `packages/ui/src/tokens/` and used through Tailwind utilities | `--color-*`, `--text-*`… |
| **Accent** | The single vivid UI colour (primary actions, selection, focus ring); variants switch with `data-accent` | `--color-accent`, `data-accent` |
| **Primitive** | A generic, accessible `@se/ui` component wrapping a Radix primitive (Button, Select, Dialog…) | `packages/ui/src/primitives/` |
| **Editor widget** | A domain-agnostic `@se/ui` component specific to video editing (ScrubInput, Dock, TransportBar, Meter…) | `packages/ui/src/editor/` |
| **Panel / Dock** | A docked area of the editor (Médias, Viewer…) / the group of resizable panels it sits in, whose sizes are remembered | `Panel`, `Dock` |
| **Layout id** | Stable name under which a dock saves its panel sizes (`editor.columns`) | `layoutId` |
| **Gesture commit** | The single `onCommit` an editor widget emits at the end of a drag, key press or pick (live updates go through `onChange`), so one gesture is one undo step | `onCommit` |
| **Shell** | The renderer's frame: routes (home, editor), editor layout and status bar | `apps/desktop/src/app/` |
| **Translation key** | Dotted id of a UI string in `src/i18n/locales/fr.json` (`home.newProject`), typed for `t()` | `t('…')` |
| **Active sequence** | The sequence the editor shows (timeline, header, later the viewer): the tab chosen in the timeline, or the project's first sequence | `useActiveSequence`, `timeline-view-store` |
| **Viewport (timeline)** | The visible window of the timeline: zoom in pixels per second, time at the left edge, lanes width | `Viewport` |
| **Lane** | The row of a track in the timeline, holding its items in view | `TrackLane` |
| **Virtualisation** | Rendering only the items crossing the visible range (plus overscan), found by binary search | `visibleItems` |
| **Playhead** | The current instant of a sequence, always on a frame start; moved by the ruler, the arrows and the timecode field | `playhead-store` |
| **Edit point** | The start of a sequence or an edge (start or end) of an item: targets of ↑/↓ and of snapping | `editPoints` |
| **Snapping** | A dragged playhead or edge (moved clips, trims, gap handle, drops) jumping to a nearby edit point, the playhead, a marker or an in/out point (8 px), shown by a line; toggled with `N` | `snapTime`, `snapShift`, `snapping` |
| **In/out points** | The half-open span of a sequence marked with `I`/`O` (or Maj+drag on the ruler), the scope of AI requests; a machine preference, not project data | `InOutPoints`, `useInOutSpan` |
| **Placement mode** | How items placed on tracks (dropped, moved, copied, pasted) treat what is there: `overwrite` replaces what they cover on their tracks (removing, trimming or cutting it), `insert` (Ctrl/⌘) opens time and pushes what follows | `PlacementMode` |
| **Ripple edit** | An edit that moves what follows its edit point to close or open time: ripple delete, ripple trim, gap resize, insert | ADR-0021 |
| **Concerned tracks** | The tracks a ripple edit shifts: the edited tracks plus every unlocked track holding items linked to what moves | `concerned_tracks`, `concernedTrackIds` |
| **Ripple trim** | Moving an item edge (with its linked partners' edges at the same instant) while what follows shifts by the change; Ctrl/⌘ while trimming | `trimItems { ripple: true }` |
| **Nudge** | Moving the selected items by one frame (`,` `.`) or ten (`<` `>`) | `nudgePlacements` |
| **Active track** | The track last clicked in the timeline: pastes land on it | `selectionStore.activeTrackId` |
| **Edit plan** | The domain operations of one gesture, planned against a draft of the document and committed as one transaction | `EditPlan` |
| **Gesture preview** | What a drag shows before its command is sent: items drawn where they would go, ghosts of new items, the snap line; never sent to the core | `gesture-store`, `GesturePreview` |
| **Marquee selection** | Selecting the items a rectangle drawn from empty lane space touches | `itemsInRect` |
| **Linked selection** | Clicking an item also selects the items of its link group (Alt selects it alone) | `linkedItems` |
| **Overview strip (minimap)** | The thin bar under the timeline showing the whole sequence and the visible window, which scrolls and zooms the timeline | `Minimap` |
| **Peak level** | The waveform resolution fetched for a zoom: the coarsest of 256/1024/4096 samples per peak giving a peak per pixel | `peakLevelFor` |
| **Shortcut action** | A named command with default keys per OS, contexts and repeat policy, in one table | `SHORTCUT_ACTIONS`, `useShortcutAction` |
| **Shortcut context** | Where a shortcut applies: `global`, or the region (timeline, viewer) last clicked or focused | `ShortcutContext`, `ShortcutRegion` |
| **Test project** | A large generated project (500 items by default) for timeline and performance work | `pnpm generate-test-project`, `se-devtools` |
| **Transition** | A cross-fade on the cut between two adjacent items of one track, stored on the track; removed when its items no longer touch, kept (shortened if needed) when they move together | `Transition`, `Track.transitions` |
| **Cut** | Where one item of a track ends and the next one starts; an *open cut* has no transition yet (the timeline shows a handle there) | `Cut`, `openCuts` |
| **Cross dissolve** | A picture transition: the outgoing picture fades out as the incoming one fades in, composited as one isolated group so lower tracks never show through | `TransitionKind::CrossDissolve`, `DissolveRole` |
| **Audio cross-fade** | A sound transition at constant power (`cos`/`sin` weights) | `TransitionKind::AudioCrossfade`, `constantPowerWeights` |
| **Transition alignment** | Where a transition's span sits on its cut: centred, starting at the cut, ending at the cut | `TransitionAlignment`, `Transition::extent` |
| **Handle** | Media available beyond an item's edges (in timeline time, speed applied), which a transition plays | `item_handles`, `Handles` |
| **Transition room** | The longest transition a cut allows (handles, item lengths, neighbouring transitions); requested durations are shortened to it | `transition_room`, `fitted_transition` |
| **Transition upkeep** | The domain rule keeping transitions in step with their items: item moves drop the ones they break, composite edits and edit plans put back those whose items touch again | `drop_broken_transitions`, `TransitionSnapshot` |
| **Fade shape** | The curve of a clip fade (linear, constant power, logarithmic = even in decibels over 60 dB), shared by sound gain and picture opacity | `FadeShape`, `fadeCurve` |
| **Volume line (rubber band)** | A clip's volume drawn over its waveform (−60 to +6 dB): dragged to move the whole curve, Ctrl/⌘+click adds a keyframe, Alt+click removes one | `VolumeLine`, `volume-line.ts` |
| **Gain envelope** | The linear gain of a scheduled voice as points (every 10 ms when it varies) that Web Audio ramps between: volume curve × fades × cross-fade weight | `gainEnvelope`, `clipGainAt`, `GainPoint` |
| **Silence (display)** | A silent stretch found by the audio analysis, hatched on audio clips (toolbar toggle) | `getAudioSilences`, `useAssetSilences` |
