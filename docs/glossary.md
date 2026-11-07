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
| **Gap** | Empty time on a track between items (implicit, not stored) | — |
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
| **Reframe** | How a clip fits a sequence format (fit, fill, custom crop/position, keyframed tracking) | `Reframe` |
| **Transform** | Position, scale, rotation, anchor and opacity of a visual item, each animatable | `Transform2D` |
| **Normalised position** | A point in the sequence frame where `(0, 0)` is the top-left and `(1, 1)` the bottom-right corner, whatever the format | `Vec2` |
| **Design pixel** | Size unit: a pixel of a frame whose shorter side is 1080, so 1 design px = 1 px in 1080-class formats, 2 px in 4K (ADR-0015) | `DesignPixels` |
| **Safe area** | Platform UI zones to keep clear (TikTok, Reels, Shorts, YouTube) | `SafeAreaPreset` |
| **Schema version** | Version of the `project.json` format a document was written with | `schema_version`, `CURRENT_SCHEMA_VERSION` |
| **Violation** | A broken document invariant found by validation, with a stable code, a path and a message | `Violation`, `ViolationCode` |
| **Operation** | One intent-level edit of the document, applied by the pure `apply`, with exact inverse operations | `Operation`, `apply` |
| **Inverse** | The operations that undo an edit, computed when it is applied (`RestoreItem` snapshots for item edits) | `Applied::inverse`, `Committed::inverse` |
| **Touched entities** | Ids of the items, tracks, sequences, markers and assets an edit added, changed or removed | `TouchedEntities` |
| **Item patch** | A typed change of an item's content, one patch type per item kind; `null` fields stay unchanged | `ItemPatch` |
| **Animatable property** | Position, scale, rotation, anchor, opacity (visual items) or volume (media clips), set with its keyframes or static value | `AnimatableProperty`, `PropertyAnimation` |
| **Trim** | Moving the start or end edge of an item; a media clip keeps playing the same source at the same instants | `TrimItem`, `ItemEdge` |
| **Split** | Cutting an item in two at an instant; the right part is a new item | `SplitItem` |
| **Ripple delete** | Deleting a time range on tracks and moving what follows left to close the gap | `RippleDeleteRange` |
| **Insert gap** | Opening empty time on tracks, moving what follows right | `InsertGap` |
| **Split ids** | Ids given by the caller for the new right parts of items a ripple delete or inserted gap cuts | `splitIds` |
| **Transaction** | Atomic group of operations by one actor | `Transaction` |
| **Actor** | Who performed a transaction: the user or an AI task | `Actor` |
| **Origin** | Who created an item (user or AI task), kept for traceability | `Origin` |
| **History** | The user's undo/redo stack | `History` |
| **Changeset** | All transactions of one AI task, revertable as a unit | `Changeset` |
| **Conflict** | A part of a task's changes a revert leaves in place (item changed since, items the task never touched, inverse no longer applies); for a write, a stale `expectedRevision` | `Conflict`, `REVISION_CONFLICT` |
| **Revert plan** | The inverse operations of a changeset that still apply to the current document, with the conflicts | `RevertPlan`, `plan_revert` |
| **Domain error** | Why the domain refused a change: an actionable message and a stable code | `DomainError`, `DomainErrorCode` |
| **Job** | A background processing unit (thumbnails, peaks, mezzanine, transcription…) | `Job` |
| **Analysis** | Derived data about an asset: silences, loudness, scene changes, faces | `Analysis` |
| **Transcript** | Word-level timed text of an asset's speech | `Transcript` |
| **AI provider** | An adapter for one AI CLI (Claude Code first) | `AgentProvider` |
| **Task** | One user request to an AI, with a prompt, a scope and one or more runs | `Task` |
| **Run** | One execution of an AI CLI process for a task (initial or follow-up) | `AgentRun` |
| **Scope** | The part of the project a task may edit: sequence, time range, tracks | `TaskScope` |
| **Agent session** | The AI CLI's conversation id, used to resume a task | `AgentSessionId` |
| **Tool** | An MCP function exposed to agents (read or write) | `tools::*` |
| **Agent instructions** | Provider-agnostic markdown guidance composed into the agent's system prompt | `agent-instructions/` |
| **Frame graph** | The resolved list of layers to draw at one instant | `FrameGraph` |
| **Compositor** | The component drawing a frame graph to a canvas | `Compositor` |
| **Golden frame** | Reference image used to test rendering | — |
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
| **Snapping** | The dragged playhead jumping to a nearby edit point, marker or in/out point (8 px); toggled with `S` | `snapTime`, `snapping` |
| **In/out points** | The half-open span of a sequence marked with `I`/`O` (or Maj+drag on the ruler), the scope of AI requests; a machine preference, not project data | `InOutPoints`, `useInOutSpan` |
| **Marquee selection** | Selecting the items a rectangle drawn from empty lane space touches | `itemsInRect` |
| **Linked selection** | Clicking an item also selects the items of its link group (Alt selects it alone) | `linkedItems` |
| **Overview strip (minimap)** | The thin bar under the timeline showing the whole sequence and the visible window, which scrolls and zooms the timeline | `Minimap` |
| **Peak level** | The waveform resolution fetched for a zoom: the coarsest of 256/1024/4096 samples per peak giving a peak per pixel | `peakLevelFor` |
| **Shortcut action** | A named command with default keys per OS, contexts and repeat policy, in one table | `SHORTCUT_ACTIONS`, `useShortcutAction` |
| **Shortcut context** | Where a shortcut applies: `global`, or the region (timeline, viewer) last clicked or focused | `ShortcutContext`, `ShortcutRegion` |
| **Test project** | A large generated project (500 items by default) for timeline and performance work | `pnpm generate-test-project`, `se-devtools` |
