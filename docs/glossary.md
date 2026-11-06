# Glossary

Use these terms consistently in code, docs, UI keys and agent instructions. Add a term here before introducing a new domain concept.

| Term | Meaning | Code name |
|---|---|---|
| **Project** | A folder `<Name>.seproj/` containing one document, its database and caches | `Project` |
| **Document** | The serialized state of a project (`project.json`) | `Project` |
| **Revision** | Monotonic counter of committed transactions (project-wide and per item) | `revision` |
| **Asset** | A media file imported into a project (video, audio, image), referenced by path | `Asset` |
| **Original** | The user's source file of an asset | `DerivedMedia::original` |
| **Mezzanine** | High-quality, WebCodecs-decodable copy of an asset whose codec Chromium cannot decode | `DerivedMedia::mezzanine` |
| **Proxy** | Low-resolution copy of an asset for smooth playback | `DerivedMedia::proxy` |
| **Sequence** | A timeline with a format (resolution, frame rate, sample rate) and tracks. A project may contain several | `Sequence` |
| **Format** | Width, height, frame rate and sample rate of a sequence; presets 16:9, 9:16, 1:1, 4:5 | `Format` |
| **Track** | A horizontal lane of a sequence holding non-overlapping items; kind video or audio | `Track` |
| **Item** | Anything placed on a track over a time range | `Item` |
| **Media clip** | An item showing/playing part of an asset | `ItemKind::MediaClip` |
| **Text / Shape / Image item** | Declarative graphic items | `ItemKind::Text`, `Shape`, `Image` |
| **Motion template** | AI-authored, parameterised render module (code + params schema) | `MotionTemplate` |
| **Template item** | An item instantiating a motion template with parameter values | `ItemKind::MotionTemplate` |
| **Caption** | Subtitle item built from transcript words, styled by a caption style | `ItemKind::Caption` |
| **Brand kit** | Fonts, colours, logo and default caption style reused by the user and agents | `BrandKit` |
| **Gap** | Empty time on a track between items (implicit, not stored) | — |
| **Source range** | The part of an asset used by a media clip (`source_start` + duration) | `source_start` |
| **Timeline range** | Where an item sits in its sequence (`start` + `duration`) | `timeline_range` |
| **Flicks** | Time unit: 1/705,600,000 s, stored as i64 | `Flicks` |
| **Timecode** | Human representation of a time (`00:00:51:12`, `51s`, `1:02.5`) | `Timecode` |
| **Keyframe** | A value of an animatable property at an item-relative time, with an easing to the next | `Keyframe<T>` |
| **Animatable** | A property that is either static or keyframed | `Animatable<T>` |
| **Easing** | Interpolation curve between keyframes | `Easing` |
| **Animation preset** | Named, reusable set of keyframes (fade in, slide, pop, typewriter…) | `AnimationPreset` |
| **Reframe** | How a clip fits a sequence format (fit, fill, custom crop/position, keyframed tracking) | `Reframe` |
| **Safe area** | Platform UI zones to keep clear (TikTok, Reels, Shorts, YouTube) | `SafeAreaPreset` |
| **Operation** | One intent-level edit of the document, with a computable inverse | `Operation` |
| **Transaction** | Atomic group of operations by one actor | `Transaction` |
| **Actor** | Who performed a transaction: the user or an AI task | `Actor` |
| **Origin** | Who created an item (user or AI task), kept for traceability | `Origin` |
| **History** | The user's undo/redo stack | `History` |
| **Changeset** | All transactions of one AI task, revertable as a unit | `Changeset` |
| **Conflict** | A revert or write that cannot apply because another actor changed the same item | `Conflict` |
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
