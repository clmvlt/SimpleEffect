# Journal de progression

Journal de bord partagé entre les sessions. **Chaque session qui exécute un prompt ajoute une entrée en haut de la liste** (la plus récente en premier). Les entrées sont rédigées en anglais (comme le reste de la documentation technique) pour rester lisibles par toutes les IA.

## Format d'une entrée

```markdown
## YYYY-MM-DD — Prompt NN: <title> — <Done | Partial>

**Branch**: `feature/NN-<slug>`, merged into `dev` at `<short sha>`

**Done**
- What was implemented (modules, commands, components).

**Deviations from the prompt**
- What differs from the spec and why (link the ADR if a decision was recorded).

**Follow-ups**
- Remaining work, known issues, TODO(prompt-NN) items left in code.

**Notes for the next agent**
- Anything non-obvious discovered (gotchas, empirical findings, commands).
```

---

<!-- Add new entries below this line, most recent first. -->

## 2026-11-07 — Prompt 16: Formats, séquences multiples, safe areas — Done

**Branch**: `feature/16-formats-sequences`, merged into `dev` at `94148f7`

**Done**
- se-domain (ADR-0026): `Reframe = fit | fill { focus } | custom | tracked { focus }` — fill crops to the frame around a focus (a point of the picture, independent of the frame's aspect), custom is the natural size (one picture pixel per design pixel), tracked is prepared for prompt 26 —; the focus is the 18th animatable property `ReframeFocus` (keyframes, validation of each component in `[0, 1]`). New `reformat` module: `FormatAdaptation { clipReframe: keep | fit | fill, fitTextWidths }`, `adapted_patch` (texts wider than 90 % of the new frame get that wrapping width), `format_change` (operations of an in-place change, locked tracks skipped), `sequence_version` / `copy_sequence` (fresh ids from a fallible `SequenceIds`, link groups and transitions remapped); `FRAME_RATE_LOCKED`, `LAST_SEQUENCE`. Trims and splits now retime **every** animatable property (blur and text styles were left outside the item and made the edit fail). Schema version 6 + migration + fixture; property tests for format changes (valid, exact undo) and versions (valid, no shared id); generators produce reframe patches and focus keyframes.
- se-app / se-protocol / se-core: `renameSequence`, `setSequenceFormat`, `duplicateSequence` (copy, or version in another format placed after its source; label « Create version »), `removeSequence` (refuses the last one); `updateItems` takes optional `animations` in the same step (a mode switch and its compensating scale); label « Reframe items ».
- @se/engine: `reframedPicture` (box + crop per mode), `fillCrop`, `reframeScale`; `PictureNode` frames its texture on the crop mapped back through display rotation and mirroring (`codedCrop`); `FrameLayout` carries crops; captions wrap at 90 % of narrow frames. `@se/engine/safe-areas`: TikTok, Instagram Reels, YouTube Shorts (zones the apps cover) and YouTube 16:9 (title/action safe), one JSON file each with source, URL and measurement date. Golden frames: fill around a left focus and on rotated phone footage (each checked against the same view placed by hand in custom mode), animated focus (pixel-equal to its held value), custom framing; the existing fit/fill references still pass unchanged.
- Renderer: `shared/format` (format fields of the new-project form, now shared), feature `sequences` (format dialog with the frame rate locked once items exist, clip policy defaulting to fill when the aspect changes, « Adapter la largeur des textes »; « Créer une version… » with the name following the ratio and the independence notice; duplicate; removal confirmation; toast listing the items **newly** out of the frame with « Sélectionner », computed after the change with `evaluate` and a canvas text measurer); timeline tabs always shown with a context menu, a « … » menu and inline rename; inspector « Cadrage » (Ajuster / Remplir / Personnalisé with a compensating scale for the natural size, « Zone gardée » keyframable, « Recadrer dans le viewer »); viewer guides per sequence (safe areas, thirds, centre cross; never exported) and reframing mode (double click on a filling clip: whole picture outlined, hidden parts shown from the thumbnail sheet, drag moves the picture under the frame, one undo step, Enter/Escape/« Terminer »).
- Tests: se-domain 231 unit + 6 property suites; se-app `item_edits/sequences.rs` (6); engine Node 217 + browser 31; desktop 410 (sequence services and dialogs, tabs, reframe inspector and switch service, viewer guides and reframing).
- Verified in the built app on Windows (isolated profile, CDP, a 1920 × 1080 test pattern): « Créer une version… » makes « Formats 9:16 » with the clip filling the frame around its centre and the independence toast; the inspector shows « Cadrage »; reframing shows the whole picture with its dimmed thumbnail exactly aligned, a drag moves the kept area (focus 50 % → 33 %); a 16:9 sequence with a wide title changed to 9:16 without text adaptation warns « 1 élément sort du cadre » with « Sélectionner », and with adaptation the title wraps at 972 px without warning; TikTok zones, thirds and centre cross draw over the picture. Three visual issues found there were fixed (legibility of covered zones, a hint wrapping word by word, text selected while dragging).
- Docs: ADR-0026, architecture §4.2/§4.3/§5.3/§6/§8/§11, glossary (9 terms), backlog, CLAUDE.md of se-domain, se-app, se-protocol, se-core, se-store, @se/engine, desktop.

**Deviations from the prompt**
- `Tracked { focus: Animatable<Vec2> }` instead of `Tracked { keyframes }`: the same animatable property as `fill`, so prompt 26 only writes keyframes and editing, retiming and the UI already work.
- Fill is a real crop (the layer's box is the frame), not an overflowing box clipped by the frame: a focus, a zoom around the visible centre and the gizmo's anchors stay consistent. Existing fill goldens are unchanged.
- `custom` = natural size (user's choice), with a compensating scale when switching to or from it.
- The out-of-frame warning is computed after the change (comparing the real states), as a toast with « Sélectionner », rather than inside the dialog.
- The safe-area choice is a machine preference per sequence (`se.ui.viewer.v1`), not project data.
- Extras accepted by the user: text width adaptation, « Sélectionner », thumbnail in the reframing mode, thirds and centre guides. The frame-rate lock lives in the planners (not in `UpdateSequence`) so undo always works.
- No platform publishes official safe areas for organic posts: the files use the best-documented third-party measurements (TikTok's In-Feed template via Socialync, Meta's Reels ad guidance, Socialync for Shorts) and SMPTE/EBU for 16:9; each file says so.

**Follow-ups**
- Prompt 19: MCP tools map onto `setSequenceFormat`, `duplicateSequence`, `renameSequence`, `removeSequence`, `updateItems` (reframe modes) and `setItemAnimations` (`reframeFocus`).
- Prompt 26: write `tracked` focus keyframes; the « Créer une version… » dialog gets its automatic-reframing checkbox.
- Prompt 27: export renders crops through the same `PictureNode` (nothing to add).
- Backlog: out-of-frame preview inside the dialog, adapting shapes/images, propagating edits to versions, reframing several clips at once, live picture in the hidden parts, more platforms and re-measured safe areas.

**Notes for the next agent**
- After rebuilding the renderer for a CDP session, reload with `Page.reload { ignoreCache: true }`: `location.reload()` kept the old bundle (as `apps/desktop/CLAUDE.md` warns).
- Heredocs with long Python edit scripts sometimes fail in this shell (« unexpected EOF »): write the script with the file tool and run it.
- Sonner renders each toast twice in jsdom (list and live region): query with `findAllByText(...)[0]`.
- `SequenceIds` is fallible (`type Error: From<DomainError>`) because se-app's id generator returns `Result`; tests use `reformat::copy::CountingIds`.

## 2026-11-07 — Prompt 15: Textes, graphiques et keyframes — Done

**Branch**: `feature/15-textes-keyframes`, merged into `dev` at `62780fc`

**Done**
- se-domain (ADR-0025): full `TextStyle` (weight, italic, animatable size and colour, `Stroke`, `TextShadow`, `TextBackground`, letter spacing, line height, alignment, wrapping width, case) and `TextItem.reveal`; shapes `Line`, outline, `cover_frame`; images `ImageFit` (fit, fill, natural); `Transform2D.blur`. `AnimatableProperty` (17 properties) with typed slots `ItemKind::property`/`property_mut`, shared by `SetKeyframes`, presets and validation (`allowed_range`, `graphic_rules.rs` for static styles); `PropertyAnimation` generated per property. **Presets as data** (`assets/animation-presets.json`: fade, 4 slides, pop, blur → sharp, typewriter, bounce, shake, pulse) expanded by `expand_preset` / `ApplyAnimationPreset { item, preset, phase, duration, start }` (span placement and clamping, base read as a hold, keyframes inside the span replaced, exits mirrored); `preset_samples`; errors `UNKNOWN_ANIMATION_PRESET`, `PRESET_PHASE_NOT_SUPPORTED`; `Animatable` reads bare values (old journals). Schema version 5 + migration + fixture; property tests generate presets; insta snapshot of every expansion.
- se-app / se-protocol / se-core: `addGraphicItem` (highest free video track above what plays, or a new one on top), `setItemAnimations` (one step for several properties/items), `applyAnimationPreset`, `listAnimationPresets` (with samples), `listFonts`; port `FontCatalog`; new adapter **se-fonts** (Inter, Montserrat, Oswald, Playfair Display, Bebas Neue, Permanent Marker latin/latin-ext WOFF2 embedded with their licences + installed fonts via `fontdb`, read in the background at startup); route `GET /media/fonts/{id}`; labels `Transform items`, `Add text/shape/image`, `Apply animation`.
- @se/engine: text layers with their whole style; Pixi text node laid out once (whole text, no outline/shadow) and drawn line by line (reveal never moves the text, outline outside), plates; outlined shapes, line capsules, frame-covering boxes; natural images; blur filter; `Compositor.render` returns boxes → `FrameLayout` published by the preview engine; `@se/engine/fonts` `FontRegistry` (catalogue faces as `FontFace`, loaded before drawing, missing families reported). Golden frames `shapes-strokes-lines`, `rotation-anchor-blur`, `slide-in-midway`, `text-styles` (bundled Inter, `TEXT_TOLERANCE` = 4 × 4 block averages + pixel checks of outline, shadow, plate, reveal).
- Renderer: `shared/animation` (property access mirror, keyframe edits with auto-keyframing, easing ↔ curve, `propertyPreviewStore`), `shared/fonts`; inspector rewritten (sections Texte, Effets du texte, Forme, Image, Transformation/Opacité; `KeyframeButton` with previous/next on every animatable row; one command per gesture; multi-selection patches); **transform gizmo** in the viewer (click/Shift selection, move with snapping to centre/edges, side/corner scale with Shift proportional, rotation with Shift 15°, anchor drag, Escape cancels, double-click text editing in place, hidden while playing); missing-font badge; **keyframe lanes** in the timeline (toggle on animated items, rows per property, select/drag by frames/`Suppr`, easing popover with `EasingCurveEditor`); feature `graphics`: Texte (`T`) and Forme tools + length of new graphics in the viewer toolbar, « Ajouter à la tête de lecture » for images in the media bin, tab « Animations » (presets by phase, hover previews from the core's samples, custom duration, applicability per selection); CSP `font-src` allows the core and `data:` (inlined UI font subsets were blocked before).
- Tests: se-domain 220 unit + property tests; se-app `item_edits/graphics.rs` (4) and unit tests; se-fonts 7; se-core font route + router; engine Node 201 + browser 27; desktop 375 (shared animation, fonts, gizmo geometry/gestures, inspector, gizmo, keyframe lane, graphics tools and presets).
- Verified in the built app on Windows (isolated profile, CDP): a project made through the core with a text (Montserrat 800 from the catalogue, yellow) and a rounded rectangle on a new V3; click selects the text (outline, 8 handles, rotation and anchor handles, inspector « Texte »); a drag previews live and leaves one undo step « Transformer les éléments »; the keyframe lane shows the pulse and fade-out keyframes; « Pop » from the Animations tab overshoots at 0.36 s; `T` adds « Titre » at the playhead on a new V4, selected.
- Docs: ADR-0025, architecture §2/§3/§5.3/§5.4/§6/§7/§8, glossary (19 terms), backlog, CLAUDE.md of root, se-domain (preset format), se-app, se-protocol, se-core, se-store, se-fonts (new), @se/engine, @se/protocol, desktop.

**Deviations from the prompt**
- A rounded rectangle is a rectangle with a corner radius (`ShapeKind::RoundedRectangle` only picks the default); shape dimensions stay static (animate `scale`), gradients in the backlog.
- « Flou → net » uses a new animatable `blur` on every visual item (a Pixi blur filter).
- `ApplyAnimationPreset` takes an optional `start` (emphasis at the playhead from the panel); preset bases read existing keyframes as holds (the domain never interpolates).
- Graphics go on the highest unlocked, visible video track free over their span *with nothing above it*, else on a new track on top, so they always show over the picture.
- The keyframe lane unfolds per item (diamond button) inside the track row (rows grow), not as a separate track.
- `setItemAnimations` and `listFonts`/`/media/fonts` were added alongside the prompt's operations; the font catalogue is a new adapter crate (`se-fonts`) with fonts embedded in the binary (no packaging change).
- Text goldens compare block averages (platform rasterisers differ at glyph edges even with the same font).

**Follow-ups**
- Prompt 16: formats — `coverFrame` shapes already follow the frame; texts may need re-layout in 9:16.
- Prompt 19: MCP tools map onto `addGraphicItem`, `updateItems`, `setItemAnimations`, `applyAnimationPreset`, `listAnimationPresets`, `listFonts`.
- Prompt 23: caption styles replace `CAPTION_LOOK` (now a full `TextItem` style).
- Prompt 27: export must set the `FontRegistry` on its `FrameRenderer` (texts wait for fonts) and reuse the returned layouts only if needed.
- Backlog: marquee/copy of keyframes, value graph editor, motion paths, gizmo snapping to other items, per-character text animations, user presets, embedding installed fonts in projects.

**Notes for the next agent**
- Edit Rust files with LF endings: Python's text mode on Windows writes CRLF (`newline=''` keeps LF); `.gitattributes` normalises on commit but rustfmt/prettier see the working copy.
- ts-rs warns on `#[serde(alias)]` (a build-time warning): keep field names stable instead.
- The raw-colour guard also scans tests: build document colours with `documentColor('ffffff')` (`shared/project-store/testing/graphics-fixture.ts`).
- jsdom measures 0 px: viewer tests needing a display scale save `{ "zoom": 0.5 }` under `se.ui.viewer.v1` before rendering (the viewer restores its preferences on mount).
- Pixi v8 leaves `container.filters` undefined until assigned (`LayerBlur` tracks attachment itself).
- CDP recipe used here: `scratchpad`-style `drive.mjs` (core socket from `getCoreConnection()`, `createProject`, `location.hash`, commands) and `Input.dispatchMouseEvent` / `Input.dispatchKeyEvent` for real pointer and key input.

## 2026-11-07 — Prompt 14: Édition audio, transitions, blancs — Done

**Branch**: `feature/14-edition-audio`, merged into `dev` at `7bb9573`

**Done**
- se-domain (ADR-0024): `Transition { id, kind: crossDissolve | audioCrossfade, leftItem, rightItem, duration, alignment: center | start | end }` on `Track.transitions`; `document::transition_fit` (`item_handles`, `transition_problem`, `transition_room`, `fitted_transition`); codes `INVALID_TRANSITION`, `INSUFFICIENT_HANDLES`, transition ids in `DUPLICATE_ID`, clip volume within −96..+24 dB; operations `AddTransition`, `UpdateTransition`, `RemoveTransition` (exact inverses, fit checked by validation at the end of the transaction); **transition upkeep**: `place_item`/`remove_item` drop the transitions an item move breaks, `SplitItem`/`RippleDeleteRange`/`InsertGap` put back those whose items touch again (`TransitionSnapshot`, shortened to fit, following the right part of a split); `FadeShape` (`linear`, `constantPower`, `logarithmic`) on `MediaClip` + `MediaClipPatch`; schema version 4 (migration and fixture in se-store; journals replay through serde defaults); property tests generate the three new operations.
- se-protocol / se-app / se-core: `updateItems` (typed patches, planned like item edits), `setItemAnimation`, `addTransitions` (durations shortened to each cut's room, cuts without room skipped, `TRANSITION_NO_ROOM`), `updateTransition` (capped), `removeTransitions`, `getAudioSilences` (`MediaService::audio_silences`); `EditPlan::finish` restores transitions after whole gestures (both items moved, ripple trims); `TrackEntity.transitions`; labels and French history strings.
- @se/engine: `fadeCurve` / `constantPowerWeights`; fades along their shapes for opacity and gain (holding at the item edges); `evaluate` gives both sides of a cross dissolve as a dissolve group (weights `1 − p` / `p`), the Pixi compositor draws it isolated with the incoming side added (`(1 − p)·(A over below) + p·(B over below)`); prefetch includes dissolve extensions; audio clips extended through sound cross-fades; `clipGainAt` + `gainEnvelope` (one point when constant, else every 10 ms) → `scheduleVoiceGain` with `linearRampToValueAtTime` (replaces the per-piece sampling of prompt 13).
- Renderer: volume line on audio clips (drag = whole curve, Ctrl/⌘+click adds a keyframe, keyframes drag in time and dB, Alt+click removes), fade handles at clip corners with shaded ramps, transitions drawn on their cut (edges resize, click selects, `Suppr` removes, menu realigns), cut handles (drag sizes a cross-fade, click adds the default one), `Ctrl/⌘+T` (`addTransition`) at the cut nearest the playhead, toolbar default transition length + silences toggle (`se.ui.timeline-edit.v1`), hatched silences, hovered gap with its duration, audio track headers with volume (dB) and pan fields (≥ 56 px), inspector « Audio » section (gain, fades, shapes) for one selected audio clip; the viewer's scene carries transitions to the engines.
- Tests: se-domain 195 unit (geometry, rules, ops, upkeep incl. split/ripple/removal between two transitions) + property tests; se-app `item_edits/sound.rs` (6), silences; se-core routing; engine Node 185 (fade curves, dissolve layers, gain envelopes dB → gain → ramps, cross-fade scheduling, prefetch) + browser `dissolve.browser.test.ts` (pixel blend of both renders, no dip over a lower track; reference `cross-dissolve-midway.png`); desktop 337 (volume keyframe add/move, fade handle, Ctrl+T, cut-handle drag, transition select/delete, services, inspector).
- Docs: ADR-0024, architecture §5.3/§5.4/§6/§8, glossary (12 terms), backlog, CLAUDE.md of se-domain, se-app, se-protocol, se-core, se-store, @se/engine, desktop.

**Deviations from the prompt**
- Clip volume stays `Animatable<f64>` (the prompt said `f32`): same type as track volume and every other animatable number.
- « Glisser la ligne = gain global » moves the whole curve (every keyframe by the same dB) when the volume is keyframed; the static value otherwise.
- Fade shapes apply to the picture opacity too (one curve per fade, WYSIWYG).
- Transitions shorter than requested: the core shortens durations to what the cut allows instead of refusing; a gesture never fails for asking too much.
- Track volume and pan show in the header only from 56 px (default height 52 px: enlarge the track, or use « Hauteur des pistes »); no popover yet (backlog).
- Not verified in the built app this time (only automated tests, including the browser golden frames): the listening checks of the acceptance criteria are to be done by hand.

**Follow-ups**
- Backlog: shrink instead of remove a transition on plain trims that steal handles, freeze-frame handles, linked picture/sound fades together, volume/pan for short headers, marquee keyframe selection.
- Prompt 15: the inspector's other sections (transform keyframes, easing of volume keyframes), « Propriétés » for transitions.
- Prompt 19: MCP tools for transitions, fades and volume map onto `addTransitions`, `updateTransition`, `removeTransitions`, `updateItems`, `setItemAnimation`.
- Prompt 22: automatic silence removal can read `getAudioSilences`.
- Prompt 27: export reuses `gainEnvelope` (offline graph) and the dissolve compositing.

**Notes for the next agent**
- An item move dropping several transitions of one track must record their `AddTransition` inverses last removal first (the property test found the swapped order; seed kept in `tests/proptest-regressions/main.txt`).
- Inverses are replayed one by one by the property test: a composite edit cannot simply clean transitions at the end of `apply_all` (intermediate inverse states would remove valid ones).
- In jsdom, read data attributes of SVG elements with `getAttribute` (the timeline's fade handles and volume dots are SVG).
- The test sequence runs at 25 fps: whole-frame positions in gesture tests (0.5 s is half a frame and rounds up).
- `playwright uninstall` removes the browsers of the **current** Playwright version, not the old ones: delete old `ms-playwright/chromium-*` folders by hand.

## 2026-11-07 — Prompt 13: Moteur B : lecture temps réel, audio, synchro A/V — Done

**Branch**: `feature/13-moteur-lecture`, merged into `dev` at `e81ea72`

**Done**
- `@se/engine/playback` (pure, ADR-0023): `Transport` (anchor + rate; `transportPosition` never accumulates, so no drift; loops wrap, non-looping playback stops at the edges), `playbackSegments` (clock windows split at loop wraps), `nextShuttleRate` (JKL 1/2/4× each way), frame pacing (`frameIndexAt`, `skippedFrames`), `PlaybackStatsCounter` (real fps, dropped frames), `RefreshPeriod` (presentation lead: the frame for the next display refresh), `PlaybackController` (play/pause, stop back to the run's start, shuttle, step, seek, loop span, content end, anchors set `leadTime` ahead so sound starts on its first sample).
- Decoding for playback: `VideoFrameSource.frames(start)` (Mediabunny `samples()`, one decoder through the file), `FrameReader` + `planRead` (forward: keep/extend, restart after a jump or when 0.5 s behind; reverse: chunks of the window before the playhead), `DecoderPool.prefetch(requests)` / `playbackFrame` (waits for the clip's reader, else random access) with a **read-ahead budget in frames and bytes** and **passed frames closed** during forward playback (`FrameCache.dropBefore`); `PreviewEngine.setPlayback` (draws in the caller's refresh, never supersedes a frame still decoding, no state emission per frame), `PlaybackPrefetcher` (`prefetchSpans` incl. upcoming clips, adaptive `PrefetchWindow` 0.25–2 s), `stats()` → `PreviewStats` (readers, queued decodes, fps, dropped, prefetch window).
- `@se/engine/audio`: `AudioGraph` (voice → clip gain → track gain with mute/solo/volume → `StereoPannerNode` → master, 10 ms smoothing), `AudioChunkPool` + `MediabunnyAudioSource` (one-second chunks, contiguous across buffers and chunks even with millisecond-rounded Matroska timestamps, decoded in order per file; media time 0 = first video frame; original or mezzanine, never the proxy), `AudioScheduler` (25 ms ticks, pieces starting within 200 ms, cut at chunks, clip edges and loop points, start times from the anchor, chunks decoded 2 s ahead, late chunks trimmed, silent at rates ≠ 1, audio scrub: 80 ms faded snippets at most every 50 ms), `audioContextClock` (heard time from `getOutputTimestamp`), `AudioMeter` (AudioWorklet `meter-processor.ts`, peak/RMS ~30 reports/s, bundled with `?worker&url`, CSP unchanged), `createAudioEngine`.
- Renderer: `features/viewer` → `ViewerWorkspace`, `PlaybackSession` (audio clock, playhead store writes, other moves = seeks with a scrub snippet when paused, picture engine in playback mode; silent on the performance clock without Web Audio), `ViewerTransport` (`TransportBar`: timecode + duration leaf, start/end, frame steps with sound, play/pause, stop, loop, master `Meter` with RMS bars and held peaks, audio scrub toggle), `PerformanceHud` (toolbar toggle), preferences `loop`/`scrubAudio`/`hud`; `shared/shortcuts` actions `togglePlay` (Space), `shuttleReverse`/`shuttlePause`/`shuttleForward` (J/K/L, K held + J/L steps through the registry's new held-key tracking), `toggleLoop` (Ctrl/⌘+L); `shared/playback` status store; timeline page-follow while playing (`pageTo`, `usePlayheadFollow`), toolbar timecode isolated in a leaf component.
- Tests: engine 163 Node (playback, readers and budgets, preview playback mode, audio scheduler incl. 10 minutes without drift, chunk assembly, pool, meter maths) + 21 browser (new `test/av-sync/`: « flash + beep » media generated in Chromium with Mediabunny; offline render puts every beep within 2 samples, clip at 0 s and 600 s; real-time run draws each flash within 45 ms of its beep — measured −10 to +7 ms); desktop: playback session (Node), viewer panel (Space, transport, JKL/K+L/J, Ctrl+L, scrub, meter, HUD), held keys, page scrolling.
- Verified in the built app on macOS (isolated profile, CDP, a 1080p 29.97 project made through the core with two ffmpeg clips on V1/A1 and V2/A2): **29.97 fps displayed, 0 dropped frame** over 12 s (two 1080p H.264 tracks + two AAC tracks; 9 fps before the byte budget and drop-behind, found with a CPU profile: main thread 83 % idle, decoders stalled on held frames); **10 minutes of looped playback: 29–30 fps throughout, 8 frames dropped out of ≈ 18,000 (0.04 %), the playhead within 2 frames of the wall clock after 10 minutes** (picture and sound share the audio clock, so no A/V drift; the wall clock is another crystal); no process left after quitting; meter at −24 dB RMS and silent at once when the tracks are muted during playback; L L = 2×, K pauses, K+L+L steps two frames, J plays in reverse, Cmd+L toggles the loop; the timeline pages (playhead line 146 → 1044 px of 1072, then back near the left); the meter worklet loads from the production bundle.
- Docs: ADR-0023, `docs/architecture.md` §4.2/§8, glossary (16 terms), backlog (Rendering engine), CLAUDE.md of root (worklet gotcha), `@se/engine` (entry points, clock, scheduling, limits, A/V test), desktop.

**Deviations from the prompt**
- Frame dropping is counted beyond what a fast speed skips anyway (at 2× every other frame is expected); a late frame is never superseded during playback (drawn, then the latest time), otherwise slow decoding would freeze the picture.
- « Stop » returns to where the playback run started (pause already exists as K/Space).
- The prompt's manual checklist alternative was not needed: the A/V offset is measured automatically (offline sample positions + real-time flash/beep).
- Extras: presentation lead (frame chosen for the next refresh), sample-exact chunk joins for Matroska/WebM, frame steps play a sound snippet, the format line moved from the footer into the HUD (the transport bar replaced the footer).

**Follow-ups**
- Prompt 14: volume keyframes as gain ramps and fades on sound (`TODO(prompt-14)` in `audio-scheduler.ts`: keyframed volume is sampled once per piece today).
- Backlog (Rendering engine): timers throttled in a hidden window, sound at 2×/4×/reverse and pitch-preserving speed, smoother reverse on long GOPs, several sound streams.
- Prompt 27: export can reuse `AudioGraph` on an `OfflineAudioContext` (the graph takes any destination) with `AudioScheduler` driven by a simulated clock, as the offline A/V test does.

**Notes for the next agent**
- Held frames stall hardware decoders: a decoder running through a file needs the frames it produced closed soon after display. Watch the HUD's « en attente » (random-access decodes during playback mean readers are not keeping up).
- Mediabunny stamps the same audio buffer differently depending on where decoding started (block timestamp rounded to the millisecond vs accumulated samples): never identify buffers by exact timestamp.
- In floating point, a clock time exactly on a frame boundary may land on either side: tests use times inside frames. 2.5 s is not a frame boundary at 25 fps (62.5 frames).
- Vitest browser: `?worker&url` modules load in AudioWorklets (dev serves ES modules); browser files run one at a time (`fileParallelism: false`) so the real-time test keeps its CPU.
- CDP-driven playback checks: `scratchpad`-style scripts evaluating in the page (core socket from `getCoreConnection()`), `keydown` events on `window`, `Profiler.start/stop` for CPU profiles; recipe in `apps/desktop/CLAUDE.md`.

## 2026-11-07 — Prompt 12: Moteur A : évaluation, décodage, compositing, viewer — Done

**Branch**: `feature/12-moteur-rendu`, merged into `dev` at `164e60e`

**Done**
- `@se/engine` (ADR-0022): root entry pure (`animation/`, `evaluate/`); subpaths `composite`, `decode`, `preview` load PixiJS 8.22 / Mediabunny 1.65 (`dependencies` of the engine).
  - `animation/`: linear, hold, CSS cubic Bézier (Newton + bisection), presets = CSS ease-in/out/in-out (same curves as `@se/ui`'s editor), number/vector/colour interpolation (OKLab, premultiplied alpha), `valueAt` (binary search over keyframes). **Shared vectors `docs/test-vectors/interpolation.json`** (computed by an independent bisection reference).
  - `evaluate(sequence, assets, time) → FrameGraph` (structural input: a domain `Sequence` or mirror entities): visible video tracks bottom first, item by binary search, source time `sourceStart + (time − start) × speed` rounded to the flick, resolved transforms, fades as opacity, natural boxes (asset display size with rotation, fit/fill; images fit), shapes and text in sequence pixels, captions as basic text (fixed look), templates skipped, invisible layers dropped.
  - `decode/`: `DecoderPool` (one Mediabunny `UrlSource` source per asset+variant over `/media`, LRU picture cache keyed by source and URL with 256 MiB / 48 pictures limits, **leases**, per-source `DecodeQueue` skipping requests aborted before they start, idle sources closed beyond 8), `FrameTracker` (open `VideoFrame`/`ImageBitmap` counter), `frameLookupTime` (exact frame index in flicks, lookup at mid-frame), `chooseVariant` (ready proxy when asked — even offline originals —, else original/mezzanine via `playbackSource`), `MediabunnyVideoSource` (media time 0 = first frame, display rotation and mirroring), `fetchImageBitmap`, `DecodeError`.
  - `composite/`: `Compositor { resize, setBackground, render, destroy }`, `createPixiCompositor` (WebGL 2, WebGPU optional, `HTMLCanvasElement`/`OffscreenCanvas`, `pixi.js/unsafe-eval` for the CSP, one node per item, textures from `VideoFrame`s), pure `layerPlacement`/`pictureFit`/`outputScale`.
  - `preview/`: `FrameRenderer` (synchronous draw when cached, else `loading` + previous image kept, superseded requests dropped, per-asset `MediaIssue`s), `PreviewEngine` (`createPreviewEngine`, `createEngineWith`: setters coalesced into one draw per animation frame, state, `stats()`, `settled()`).
- `@se/protocol`: `AssetMediaFile` gains `mezzanine` and `proxy`.
- Renderer `features/viewer`: canvas at the sequence ratio fitted to the panel or at 50/100/200 % (scrolls), « Lire les proxys », transparency checkerboard (preferences in `se.ui.viewer.v1`), spinner after 150 ms of decoding, media-issue badge, « Aperçu indisponible » without WebCodecs/WebGL, footer timecode + format; `ViewerEngineController` owns the engine outside React (fresh canvas per engine, StrictMode-safe), `@se/engine/preview` imported lazily, playhead store followed directly; `ShortcutRegion context="viewer"` (time shortcuts work in the viewer). Dev builds expose `window.simpleEffectPreviewStats()`.
- Golden frames (`packages/engine/test/golden/`, project `engine-browser`, Vitest browser mode + Playwright Chromium, SwiftShader, software decoding): frame-number readback (29.97 H.264 with B-frames at every tested frame and one flick before a boundary, offsets, speed 2, 25 fps VP9, proxy, rotated phone clip), **9 reference scenes** (shapes/opacity, test pattern, video frame, multitrack transforms + mirror + alpha, 9:16 fit and fill, keyframes midway, alpha over video, proxy framing toggled on one engine) plus OffscreenCanvas = canvas, transparent background, text placement; real-WebCodecs scrub leak test. Fixtures committed (≈35 kB) from `scripts/generate-golden-fixtures.mjs`.
- Tests: engine 107 Node + 18 browser; desktop viewer 17 (services, controller, panel with a fake engine); app shell updated.
- Verified in the built app on macOS (isolated profile, CDP, a project made through the core with a 720p 29.97 `testsrc` clip, a rotated phone clip and colour bars): frame 0 and typed timecodes show the right frames (phone frame 45 read back upright from its bits at 00:00:11:15), bars composited over the clip on V3, proxy generated then toggled on/off with identical framing, 100 % zoom draws a 1920×1080 canvas in a scrolling stage, checkerboard on; **60 s of intensive arrow-key scrubbing within the content (7,461 key presses) held at most 47 frames + 1 image (cache limit 48) and left 0 open pictures once the editor closed**.
- Docs: ADR-0022, `docs/architecture.md` §2/§4.2/§8, glossary (13 terms), backlog (Rendering engine section), CLAUDE.md of root (Chromium command and gotcha, fixtures script), `@se/engine` (rewritten: entry points, internal architecture, frame lifetime rules, golden workflow), `@se/protocol`, desktop; CI installs and caches Playwright Chromium.

**Deviations from the prompt**
- Reframe modes are the domain's `fit` and `fill` (the prompt's « custom » does not exist in the document; a transform on top is the custom framing). Images (no `reframe` field) fit the frame at scale 1 (ADR-0022).
- Frame lookup uses the exact frame index at mid-frame instead of the raw time: robust to millisecond-rounded containers (found while testing: an epsilon either switched frames one flick early or missed Matroska frames).
- Interface `Compositor` also has `setBackground` (black for preview/export, transparent behind the viewer's checkerboard).
- Golden tests force SwiftShader and software decoding (pixels identical across machines); fixtures are committed rather than generated by ffmpeg at test time; text is checked by placement only (fonts differ per OS).
- Decoding and compositing run on the renderer's main thread (WebCodecs decodes off-thread); the engine is Worker-ready (`OffscreenCanvas` tested) — moving it is in the backlog.
- The dev counter is exposed only by the Vite dev server (`import.meta.env.DEV`), not by builds.

**Follow-ups**
- Prompt 13: sequential decoding for playback (`samples`/`samplesAtTimestamps`, one decoder per run instead of one per `getSample`), prefetch window, `AudioEngine`, transport bar in `ViewerPanel` (TODO in code).
- Prompt 15: full text styles in `TextNode` (TODO), fonts in Workers. Prompt 23: caption styles replace `CAPTION_LOOK`. Prompt 24: template layers. Prompt 26: automatic reframe variant. Prompt 27: export reuses `FrameRenderer` + `PixiCompositor` on an `OffscreenCanvas`.
- Backlog (Rendering engine): Worker hosting, WebGL context-loss recovery, pixel aspect ratio, runtime fallback to the mezzanine when `canDecode()` refuses, scrub prefetch.

**Notes for the next agent**
- Vitest browser-mode file commands resolve paths from the project root (`packages/engine`), not from the test file. `SE_UPDATE_GOLDEN=1` rewrites references: look at every PNG before committing.
- A Pixi `ImageSource` whose resource changes size in place kept the old frame size (the proxy toggle zoomed the picture): `PictureNode` creates a new texture when the size changes. A golden test toggles the proxy on one engine for this.
- Mediabunny's `getSample` configures a new `VideoDecoder` per call; its `UrlSource` retries failed requests forever by default.
- `electron-vite build --mode development` still compiles `import.meta.env.DEV` to false. In CDP checks, `[role=status]` also matches the status bar's connection indicator: scope selectors to the viewer.
- macOS VideoToolbox kept decoding with 47 held frames (no stall at the default limits).

## 2026-11-07 — Prompt 11: Timeline B : édition — Done

**Branch**: `feature/11-timeline-edition`, merged into `dev` at `60ddd95`

**Done**
- se-domain: `apply_all(&Project, &[Operation])` (one draft, `validate_project` once after the last operation); `commit` and `apply` use it. A 500-item move went from 0.8 s (release) / 1.8 s (debug) to 3 ms: validation (~1 ms for 500 items, mostly path strings) ran after every operation.
- se-protocol: `editing.rs` (`PlacementMode`, `ItemPlacement`, `PastedItem`, `ItemTrim`, `TrackTarget`, `PlaceMediaRequest`); commands `placeMedia`, `moveItems`, `duplicateItems`, `pasteItems`, `trimItems`, `splitItems`, `deleteItems`, `linkItems`, `unlinkItems`, `resizeGap`; error code `RIPPLE_BLOCKED`.
- se-app: `edit_plan` (crate-private `EditPlan`: operations queued, applied to a draft with `apply_all` when read, committed as one transaction by `commit_plan`; `lookup`, `placement` (lift, place overwrite/insert), `overwrite::clear_range`, `ripple` (`concerned_tracks`, `open_time`, `remove_time`, grouped `ripple_delete`), `new_items` (`media_parts`, `copies_of`, `DEFAULT_IMAGE_DURATION` = 5 s)); use cases `place_media` (`placed_media.rs`, new tracks on demand), `move_items`, `duplicate_items`, `paste_items`, `delete_items`, `link_items`, `unlink_items` (`items.rs`), `trim_items` (plain and ripple), `split_items` (right parts of linked items relinked), `resize_gap` (`trims.rs`); `item_labels`; `AppError::{InvalidEdit, RippleBlocked}`; `NewIds::{item, link_group}`. se-core: `rpc/handlers/items.rs` and router arms.
- Renderer: gestures in `features/timeline` — one pointer handler for the lanes (`use-lane-pointer.ts`: selection with click meaning on release so a drag moves the whole selection, rectangle, gap selection), controllers `hooks/gestures/` (move with linked items across tracks of their kind by the same count, Alt copies, Ctrl/⌘ inserts; edge trims with linked partners aligned, Ctrl/⌘ ripples; gap handle), Escape cancels; drop of bin assets with ghosts (`use-media-drop.ts`, new audio track below the last one, Ctrl/⌘ inserts); shortcuts `S` and Ctrl/⌘+K (cut at the playhead: selection or every unlocked item there), `Suppr`/`⌫` (delete, or close the selected gap), Maj+`Suppr` (ripple delete), Ctrl/⌘+C/X/V and Ctrl/⌘+Maj+V (insert paste at the playhead on the active track), `,` `.` `<` `>` (nudge 1/10 frames), `N` (snapping, moved from `S`); context menu on the lanes (« Couper ici », « Dupliquer », « Lier »/« Délier », « Propriétés », « Afficher dans le panneau Médias », « Supprimer », « Supprimer avec ripple »; « Fermer le blanc » on a gap; « Coller » on empty space); snapping by whole frames to the playhead, item edges, markers and in/out points with a snap line; trim handles; selected gap shaded with its length and a handle. Ephemeral `gesture-store` preview, one command on release. Pure services (`snapping`, `move-placement`, `track-numbers`, `trim-placement`, `gaps`, `ripple-preview`, `drop-placement`, `clipboard`, `item-edits`, `preview`).
- Renderer elsewhere: media bin entries draggable (`use-asset-entry.ts`, `shared/drag`), `revealAsset` exported; `shared/side-panel` (controlled inspector/AI tabs); `errors.edit.<DOMAIN_CODE>` messages for overlaps, locked tracks, trims beyond the media, edit points; history labels and French strings.
- Verified in the built app on macOS (isolated profile, CDP, the domain's example project): move (« Annuler : Déplacement d’éléments »), trim (« Rognage »), gap selected (« Blanc de 0:02 ») and lengthened by its handle (« Ajustement d’un blanc », logo pushed), three undos back to the start; ripple trim of the linked interview clip by −1 s (V1 and A1 shortened, the next V1 clip pulled in, music on A2 untouched); `S` after 50 frames cut the selected linked pair (« Coupe »); Maj+⌫ on « Bonjour » closed its time on V2; Alt+drag of « Forme » overwrote « Merci d’être là »; `.` twice = two undo steps; Ctrl/⌘+C, click on V3, Fin, Ctrl/⌘+V pasted the logo at 14 s on V3; context menu « Supprimer avec ripple »; music.wav dragged from the bin with ⌘ over V2 at 12 s inserted on A2 (existing music cut at 12 s, the rest pushed); snap line on Bonjour’s end while dragging the logo, Escape cancelled without command. No process left after closing.
- Tests: se-domain (single validation per transaction), se-app 15 planner unit tests + 12 integration tests (`tests/item_edits/`), se-protocol shape tests, se-core routing test; desktop: `TimelineEditing.test.tsx` (9: move with preview, linked insert move, Alt copy + Escape, trims and ripple trim, `S` + ripple delete, nudge + copy/paste, gap handle + close, bin drop, context menu), services (`edit-gestures.test.ts`, `edit-actions.test.ts`), error keys.
- Docs: ADR-0021, `docs/architecture.md` §4.2/§5.4/§5.6/§6, glossary (placement mode, ripple edit, concerned tracks, ripple trim, nudge, active track, edit plan, gesture preview; gap and snapping updated), backlog (Timeline items), CLAUDE.md of se-domain, se-app, se-protocol, se-core, desktop.

**Deviations from the prompt**
- Commands per intent rather than a generic `ApplyEdit` (ADR-0021), all actor `User`.
- Overwrite rule (the prompt left « refuse ou rogne » open): placed items **trim** what they cover on their own tracks (removing covered items, cutting spanning ones); moves and copies use the same placement modes as drops (Ctrl/⌘ inserts).
- Ripple scope: the edited tracks plus the tracks of items linked to what moves (no sync-lock setting in the document); removing time over other content of such a track is refused (`RIPPLE_BLOCKED`) instead of cutting it; locking a track leaves it out.
- `S` now cuts (prompt 10 had it toggle snapping): snapping moved to `N` as the prompt asks. Ten-frame nudges are `<` / `>` (Maj+, / Maj+. on QWERTY; own key on AZERTY, where Maj+, is `?`).
- Domain: `commit` validates once per transaction (performance; intermediate states are no longer validated, stored states always are).
- « Dupliquer » in the menu places copies right after the selection in insert mode (pushing what follows); copies are `origin: user`.
- Extras: insert paste (Ctrl/⌘+Maj+V), « Coller » and « Fermer le blanc » in the menu, precise error toasts for common refusals.

**Follow-ups**
- Prompt 14: transitions, volume/fade handles plug into the same lanes and `gesture-store`; gap visualisation (« durée affichée au survol ») can reuse `GapHighlight`.
- Prompt 15: « Propriétés » opens the inspector tab (`shared/side-panel`); the inspector must read `selectionStore`.
- Prompt 19: MCP write tools call the `ProjectService` item use cases (same ripple and overwrite rules).
- Backlog (Timeline): per-track sync lock, roll/slip/slide, auto-scroll while dragging, multi-asset drags from the bin, cross-project paste.
- Media priority bump when a clip starts using an asset (prompt 09 follow-up) is still open.

**Notes for the next agent**
- `EditPlan::push` only queues: refusals surface at the next `plan.project()?` or at commit. Read everything you need from one `plan.project()?` borrow before pushing (it takes `&mut self`).
- se-core publishes `projectPatched` before answering a command, so the mirror is current when `client.send` resolves: gestures clear their preview then.
- CDP on macOS: modifiers bits are Alt 1, Ctrl 2, Meta 4, Shift 8. `getCoreConnection().url` is the HTTP base, not the socket URL. The emulated viewport resets when the CDP socket closes; measure positions with `getBoundingClientRect`, never from screenshots (Retina screenshots are 2×).
- jsdom: no `DragEvent` (set `clientX`/`clientY` on the created event), `scrollIntoView` stubbed in `vitest.setup.ts`.

## 2026-11-07 — Prompt 10: Timeline A : affichage et navigation — Done

**Branch**: `feature/10-timeline-affichage`, merged into `dev` at `75296a9`

**Done**
- se-protocol: commands `addTrack { projectId, sequenceId, trackKind, index }` → `trackId`, `removeTrack`, `updateTrack { patch: TrackPatch }`, `reorderTracks`, `addMarker { time, label, color }` → `markerId`, `updateMarker { patch: MarkerPatch }`, `removeMarker` (`Command` is no longer `Eq`); `TrackPatch`/`MarkerPatch` fields optional in TypeScript (`#[ts(optional_fields)]`).
- se-app: `ProjectService::{add_track, remove_track, update_track, reorder_tracks, add_marker, update_marker, remove_marker}`, each one user transaction through `commit_user_edit`, new tracks named after the next free `V<n>`/`A<n>`, labels per changed field (`track_labels`, `marker_labels`: `Mute track`, `Rename marker`…), `NewIds::marker`, public `new_project::empty_track`. se-core: `rpc/handlers/timeline.rs` and router arms.
- `@se/ui`: `ClipShell` + `ClipBadge` (story, test, axe), marker colour tokens `marker-*` (Premiere's eight colours).
- Renderer `shared/time`: flicks, frame rates, rounding modes, `parseTimecode`/`formatTimecode`, pixels at a zoom — **parity tests replay `docs/test-vectors/flicks.json` and `timecode.json`** (same error codes as Rust). `shared/canvas`: cached `tokenValue`, `prepareCanvas`.
- Renderer `shared/shortcuts` rebuilt: `SHORTCUT_ACTIONS` (named actions, contexts `global`/`timeline`/`viewer`, OS-adapted keys, repeat policy), registry bound by `useShortcutAction`, active context from the `ShortcutRegion` last clicked or focused, conflict detection (default table tested conflict-free), key caps in tooltips (`useShortcutHint`), « Raccourcis clavier » help dialog (`?`, header button). Undo/redo moved to the actions (`Ctrl+Y` also redoes off Apple).
- `features/timeline`: sequence tabs with « + » (creates and shows « Séquence N »), toolbar (playhead timecode field accepting typed times, add video/audio track, track height presets, snapping toggle, zoom buttons/slider/fit), ruler (adaptive graduations on round timecodes, canvas; click/drag moves the playhead with snapping; Maj+drag marks in/out), markers (flags, label on hover, `M` at the playhead, double-click to rename, context menu: colour, go to, delete), track headers (kind icon, inline rename, hide/lock on video, mute/solo/lock on audio, bottom edge resize, Alt+wheel, context menu add above/below, rename, delete, drag to reorder within a kind), lanes with virtualisation, clips (`ClipShell`, thumbnail strip from the sheet, waveform canvas at the peak level of the zoom, AI badge, selected/dimmed states), selection (click with linked items, Maj adds, Ctrl/⌘ toggles, Alt alone, rectangle from empty space, Ctrl/⌘+A, Ctrl/⌘+Maj+A), in/out points (`I`, `O`, `Alt+X`, shaded on lanes and ruler, `useInOutSpan` exported for prompt 20), playhead (own store, moves by style; ←/→, Maj+←/→, ↑/↓ edit points, Début/Fin), zoom (Ctrl/⌘+wheel around the pointer, `+`/`-`, `Maj+Z` fit) and scroll (Maj+wheel, swipe), overview strip (minimap: drag to scroll, edges to zoom, click to jump), active panel outline. The editor header shows the active sequence (`useActiveSequence`; `selectActiveSequence` is gone).
- `crates/se-devtools` + `pnpm generate-test-project`: a valid project (500 items by default, V1–V3/A1–A3, linked clips, texts, captions, music, AI-made items, 12 markers) built from the domain types over six ffmpeg-generated media probed like an import (reused by prompt 29).
- Verified in the built app on Windows (isolated profile, CDP) with the 500-item project: thumbnails, waveforms, markers and minimap appear; frame times measured over CDP while wheel-zooming/scrolling the lanes: **zoom and scroll at 144 Hz (p95 7 ms)**, horizontal swipe with all 500 items in view p95 14 ms; `I`/`O` shade the span, `M` adds a marker (« Annuler : Ajout de marqueur »), rename (double-click + Entrée), mute (« Piste rendue muette »), add above (V4 on top), drag reorder (« Réorganisation des pistes »), Ctrl+Z, a rectangle selected 20 items, `?` opens the help, « Nouvelle séquence » adds a tab and the header follows; zoom, scroll and track state survive a reload.
- Tests: se-app 3 integration (`timeline_edits.rs`) + unit labels/names, se-protocol shape test, se-core routing test, se-devtools 2; desktop 265 (time parity 98, shortcuts 13, timeline services 38, persistence 3, `TimelinePanel` 7: order of tracks, click/Maj/Ctrl/Alt selection, rectangle, I/O/Alt+X, M, rename and mute commands, edit-point navigation); ui 475 (ClipShell + axe).
- Docs: ADR-0020, `docs/architecture.md` §2/§3/§4.2/§6, glossary (15 terms), backlog (Timeline items), CLAUDE.md of root, desktop, `@se/ui`, se-protocol, se-app, se-domain, se-devtools (new).

**Deviations from the prompt**
- Extras chosen by the user: timecode jump field, shortcuts help dialog, playhead snapping (`S`), overview strip (minimap), marker colours menu, « + » in the sequence tabs, quick track heights (Alt+wheel, presets).
- View state (zoom, scroll, in/out points, playhead, track heights, active sequence, snapping) is a **machine preference saved per project in `localStorage`**, not project data (user's choice, ADR-0020): no undo step for it.
- Header toggles depend on the track kind (video: hide + lock; audio: mute + solo + lock): mute/solo mean nothing for picture tracks.
- `ClipShell` was missing from prompt 04 (ADR-0009 listed it with Ruler, Playhead and TrackHeader): only `ClipShell`/`ClipBadge` became `@se/ui` widgets; the ruler, playhead and headers stay timeline components (domain-shaped).
- `addTrack` takes `trackKind` (a struct-variant field cannot be named `kind`, the tag).
- The out point is placed after the frame under the playhead (`I` then `O` on one frame gives a one-frame span).

**Follow-ups**
- Prompt 11: clip gestures plug into `useLaneSelection` (pointer-down on an item stops before the marquee), ephemeral drag state in the timeline; maybe a generic edit command for clip operations (ADR-0020 alternatives).
- Prompts 12/13: the viewer reads `useActiveSequence` and `playheadStore` (`index.ts`); playback moves the playhead store (lines already update by style); the time actions already list the `viewer` context, wrap the viewer in `ShortcutRegion context="viewer"`.
- Prompt 20: `useInOutSpan(sequenceId, contentEnd)` is the AI scope; the selection store holds the selected items.
- Prompt 29: `pnpm generate-test-project` and the CDP frame-time measurement (recipe in `apps/desktop/CLAUDE.md`).
- Backlog (Editing): peaks decoding in a worker, keyboard navigation between clips, user-defined shortcuts, waveforms on video tracks, view state across machines.

**Notes for the next agent**
- `getComputedStyle(...).getPropertyValue` on every canvas draw forced style recalculation and cost more than all of React during zoom: use `tokenValue`. Resizing a canvas reallocates it: `prepareCanvas` only resizes on change (waveform canvases in 128 px steps). `contain: strict` on clip boxes removed the remaining layout spikes.
- CDP-synthesised drags get a `pointercancel` when the real mouse moves inside the window: keep hands off the test window (real drags work).
- Heredocs in the Bash tool break on backticks: write edit scripts with the Write tool (as noted before). Vite `?raw` imports read JSON outside the TS project (`docs/test-vectors`) without adding it to `tsconfig`.
- `ShortcutRegion claimOnMount` makes the timeline the active context when the editor opens; tests rely on it to press `I`, `M`, arrows.

## 2026-11-07 — Prompt 09: Médias 2 : codecs pro, mezzanines, proxies, analyses — Done

**Branch**: `feature/09-medias-pro-analyses`, merged into `dev` at `f755755`

**Done**
- se-domain: `analysis` module (`AudioAnalysis` with integrated loudness, range, true/sample peaks, short-term `LoudnessCurve` every 0.5 s, `SilenceSettings::SPEECH` −40 dBFS/0.5 s and silences; `SceneDetection` with `DEFAULT_THRESHOLD` 0.3; `AnalysisResult`, `AnalysisKind`; summaries `AudioSummary`, `SceneSummary`, `AssetAnalysis`), `Asset.analysis` (**schema version 3**, `migrate_v2_to_v3`, `#[serde(default)]` so v2 journals replay), `AssetPatch` with one field per derived copy and summary (`mezzanine`, `proxy`, `audioAnalysis`, `sceneDetection`) instead of `derived`, `Flicks::from_millis`.
- se-protocol: commands `reportDecoderCapabilities`, `generateDerivedMedia`, `getMediaSettings`, `updateMediaSettings`, `getMediaToolsStatus`, `getProjectCache`, `clearProjectCache`; DTOs `MediaSettings`, `ProxyResolution` (`540p`/`720p`), `DecoderCapabilities`, `MediaToolsStatus`, `DerivedKind`, `GenerateDerivedRequest`, `ProjectCacheInfo`; job kinds `mezzanine`, `proxy`, `audioAnalysis`, `sceneDetection`; codes `DERIVED_NOT_APPLICABLE`, `MEDIA_TIME_OUT_OF_RANGE`.
- se-app: pure `media_policy` (`mezzanine_decision` → `Waiting`/`NotNeeded`/`Needed(TranscodeRecipe)`, `auto_proxy_wanted`, `proxy_recipe`, decision tables); ports `MediaTask`/`MediaJobOutput`, `TranscodePlan`/`TranscodeRecipe` (container, `VideoPlan`, `AudioPlan`), `FrameExtractor`, `MediaTools`, `AnalysisStore` (from `ProjectStore::analyses()`, exposed by `ProjectActorHandle::analyses()`), `MediaFiles::{directory_size, remove_directory}`; `JobQueue` with an `Idle` priority, a global limit of 4 running jobs (priorities now order kinds) and `JobAttempt { number, last }`; `MediaService` scheduling (`schedule_asset` with `Trigger::{Change, Opening}`: cache files, analyses once per content key, derived copies recorded `pending` then `ready`/`failed` by system transactions, retried at opening), `generate_derived`, media settings and decoder capabilities in `library.db` (`settings::keys`), `tools_status`, `project_cache`/`clear_project_cache`, `extract_frame`; `MediaTestbed` + fakes (`FakeFrameExtractor`, `FakeMediaTools`, files written by jobs, analyses in the in-memory folders).
- se-media: `FfmpegTools` (`MediaTools`, shared `ToolLocator` whose user folder changes at once, version, hardware encoders), `encoders` (NVENC / VideoToolbox / VAAPI validated by a 5-frame test encode per ffmpeg, cached, warmed up at startup), `transcode_args` (pure; CRF → `-cq`/`-q:v`/`-qp`, VAAPI upload, proxy scaling never enlarging, `-noautorotate` keeping coded size and rotation), transcode job with software fallback and `.part.<ext>` outputs, audio analysis job (PCM → `AudioAnalyzer` on the blocking pool through a channel: `ebur128` + 20 ms RMS silence detector), scene detection job (`select='gt(scene,T)',showinfo` split graph, stderr parsed), frame extraction (upright JPEG, last-frame fallback), `FsMediaFiles` folder size/removal past files in use, `Sidecar::wait_with_stderr`, shared `jobs/pcm.rs` reader.
- se-store: `project.db` schema v2 (`analyses.content_key`, `format_version`), `FsAnalysisStore`, fixture `project-v3.json`.
- se-core: wiring (runner as job runner and frame extractor, tools warm-up task, library for settings), handlers `media.rs` + `media_settings.rs`, `/media/assets/{id}/mezzanine` and `/proxy`, **Windows kill-on-close job for the core itself** (`platform.rs`, `win32job`): no sidecar outlives a killed core (the backlog item; the startup warm-up made the race frequent in process tests).
- `@se/engine`: `checkDecoderCapabilities` (capabilities) and `@se/engine/sources` (`playbackSource`, `isPlayable`).
- Renderer: `features/settings` (« Réglages » modal from the home and editor headers: mezzanine quality slider, proxy resolution, automatic proxies, ffmpeg folder with picker/reset and status — version, hardware encoders —, project cache location/size, « Afficher dans le dossier », « Nettoyer le cache du projet » with confirmation); media bin: decoder capabilities reported before decodability, playback badges (« En préparation », « Illisible », « Manquant »), « Lecture » group (source, mezzanine and proxy states with progress, failure reason and « Réessayer »), « Analyses » group (loudness LUFS, true peak dBTP, range LU, silences, shot changes), « Générer un proxy », job labels for the new kinds; `shared/text/byteSize`.
- Verified in the built app on Windows (isolated profile, CDP): « Réglages » from the home screen shows CRF 14, 540p, automatic proxies, ffmpeg 9.0.2 and NVENC; in a new project, dropping a ProRes 422 HQ 10-bit clip (with a 1.5 s silence) and a 1080p H.264 clip imports both; the ProRes gets an HEVC Main 10 mezzanine and a 540p proxy (« Source : Mezzanine », « Prête », « Prêt »), loudness −22,2 LUFS, true peak −20,0 dBTP, range 3,0 LU and « 1 silence (1,5 s) » — the same figures as ffmpeg’s `ebur128` filter; the H.264 clip plays from its original with neither copy; « Nettoyer le cache du projet » drops the cache to 0 o and every derived file is back within seconds while the analyses keep their import timestamps; « Générer un proxy » on the H.264 clip writes `proxy-540.mp4`; closing the window leaves no Electron, se-core or ffmpeg process.
- Tests: se-domain 168 + properties (generators cover the new patch fields), se-store migrations/fixtures + analyses table, se-app 40 unit (decision tables, priorities across kinds) + 9 derived-media scenarios + adapted media/job tests, se-media 46 unit (silence, loudness on sines, `showinfo` fixture, transcode args, folder ops) + 8 real-ffmpeg tests (ProRes → HEVC Main 10 mezzanine probed, rotated proxy at 960×540 keeping rotation, frames, silence 2–3 s + loudness, cuts at 2 s and 4 s), se-core 34 unit + 18 process, engine 15, desktop 114 (settings dialog, details panel with derived states/analyses/retry, services, capabilities report).
- Docs: ADR-0019, `docs/architecture.md` §3/§4.3/§5.3/§6/§7/§11, glossary (16 terms), backlog (Media section), CLAUDE.md of se-domain, se-protocol, se-app, se-media, se-store, se-core, `@se/engine`, desktop.

**Deviations from the prompt**
- Mezzanine target: HEVC Main 10 only for sources deeper than 8 bits (when the machine decodes it); 8-bit sources become H.264 High (user's choice: faster, decoded everywhere, nothing lost). The pixel-layout rule also covers 4:2:2/4:4:4/12-bit footage the browser reports decodable.
- Settings « emplacement et taille du cache »: the location and size of the project's cache are shown (with reveal and cleanup), not configurable (user's choice; a global cache folder and a size limit are in the backlog). The ffmpeg folder (`media.toolsDir`, prompt 08 follow-up) is in the same screen. Settings is a modal dialog (user's choice).
- Hardware encoders: NVENC, VideoToolbox, VAAPI as listed (AMF/QSV in the backlog); detection starts at core startup in the background and is cached per ffmpeg path.
- Extra: `generateDerivedMedia` also retries a failed mezzanine (« Réessayer »); failed copies are retried at project opening; `/media/…/mezzanine` and `/proxy` routes; `@se/engine/sources` for the « en préparation » state; mezzanines keep only the first sound stream (as playback does).
- Analysis data: short-term loudness every 0.5 s; silences use RMS of 20 ms windows over all channels; scene score computed on a 320 px copy.
- Priorities needed a global running limit (4): with per-kind slots alone, every kind ran at once and priorities only ordered jobs of one kind.
- The Windows kill-on-close job for se-core (backlog item) was done here: the warm-up encode at every core start left suspended ffmpeg processes behind killed test cores.

**Follow-ups**
- Prompts 12/13: decode what `playbackSource` says, switch preview to the proxy (`/media/…/proxy`), and keep originals for Premiere export.
- Prompt 11: raise the mezzanine priority when a clip starts using an asset (backlog).
- Prompt 18: MCP tools reading `project.db` analyses (`AnalysisStore::load_analysis`) and `MediaService::extract_frame`.
- Prompt 22: silence cutting can reuse the stored silences or recompute from cached peaks.
- Backlog (Media): configurable cache folder and size limit, VFR conformance, several sound streams in mezzanines, AMF/QSV, regenerate HEVC mezzanines on machines without HEVC, cleanup of unused derived files, sidecars outliving a `SIGKILL`ed core on Linux/macOS.

**Notes for the next agent**
- ffmpeg keeps the display matrix of a transcoded stream only with `-noautorotate` before `-i` (verified with 9.0.2: NVENC and libx264 outputs keep `rotation=90`).
- `showinfo` logs at `info` level: the scene job runs `-v info` and parses stderr; `-progress pipe:1` advances only because the graph also sends every frame to the null output.
- libebur128 reports short-term loudness before 3 s of audio (the missing time counts as silence), and a stereo 1 kHz sine at amplitude A measures 20·log10(A) LUFS; ffmpeg's mono → stereo upmix is −3 dB per channel.
- PCM chunks from a pipe can end mid-frame: carry the partial frame (`LoudnessMeter`).
- Windows: never drop the core's job handle (closing it kills the core itself); `win32job::Job::into_handle` keeps it for the process lifetime.
- Heredocs in the Bash tool choke on backticks: write edit scripts to the scratchpad. Python on Windows writes CRLF unless `newline='\n'`.

## 2026-11-07 — Prompt 08: Médias 1 : import, bin, jobs, miniatures, waveforms — Done

**Branch**: `feature/08-medias-import`, merged into `dev` at `b097993`

**Done**
- `pnpm fetch-binaries` (`scripts/fetch-binaries.mjs` + pure `scripts/lib/binaries.mjs`, tested): FFmpeg 9.0.2 pinned by URL, size and SHA-256 in `scripts/binaries.manifest.json` (Gyan.dev essentials for Windows, Martin Riedl for macOS arm64/x64, BtbN GPL 9.0 for Linux), plan shown (size, source, licence) and confirmation asked (`--yes` for CI, `--platform`, `--all`), extraction with system `tar`/`unzip`, stamp file for idempotence. `ci.yml` and `release.yml` fetch them (cached by manifest hash); electron-builder bundles them in `resources/bin` next to se-core.
- se-domain: `ContentFingerprint` (validated sample hash, `same_content`, `cache_key`), per-stream `Decodability { video, audio }` (`StreamDecodability`), `Asset.fingerprint`, `MediaInfo` codec `level`/`codecTag`/`pixelFormat` and audio `profile`, `AssetPatch.fingerprint`, `Actor::System`, `JobId`; **schema version 2** with `migrate_v1_to_v2` (se-store) and fixtures `project-v1.json` (migrated) / `project-v2.json`.
- se-protocol: commands `importMedia`, `relinkAsset`, `removeAsset`, `listJobs`, `cancelJob`, `reportDecodability`; events `jobQueued`, `jobProgress`, `jobCompleted`, `jobFailed`, `jobCancelled`; `ImportReport`, `RelinkOutcome`, `JobSnapshot`/`JobState`/`JobKind`, `ThumbnailSheet` (codegen root); codes `ASSET_NOT_FOUND`, `MEDIA_NOT_FOUND`, `MEDIA_UNSUPPORTED`, `MEDIA_KIND_MISMATCH`, `MEDIA_TOOLS_MISSING`, `JOB_NOT_FOUND`.
- se-app: media ports (`MediaProber`, `MediaFiles`, `MediaJobRunner`, `ProgressSink`, `MediaError`) with fakes; generic `JobQueue` (priorities, concurrency per kind, cancellation by aborting the task, one retry, per-percent progress, `cancel_project` waiting for tasks to stop); `MediaService` (import with fingerprint dedup and 4 concurrent probes, one `Import media` transaction, relink with content check and `force`, undoable removal, decodability as system transactions, missing-media check at opening with silent relink through the relative hint, jobs for missing outputs only); `ProjectObserver` hook on `ProjectService` (opened before the session is returned, closing before saving).
- se-media: `ToolLocator` (user folder → executable folder → `resources/bin/<platform>` in an ancestor → `PATH`), `Sidecar` spawn helper over `process-wrap` (job object + `CREATE_NO_WINDOW` / process group, tree killed on drop, stderr tail in errors), `FfprobeProber` + pure `parse_probe_output`, `FsMediaFiles` + xxh3 fingerprints, `FfmpegJobRunner`: thumbnails (≤ 40 tiles 90 px high, keyframes beyond 30 s, real tile times through the `metadata` filter, 8-column sheet, 360 px poster) and waveform peaks (PCM f32 from stdout, 256/1024/4096 levels, `SEPK` files); atomic outputs in `cache/media/<key>/`.
- se-core: `services.rs` wiring (setting `media.toolsDir`), media handlers, `/media` routes (original, thumbnails JSON/sheet/poster, peaks) with tower-http `ServeFile` (Range 206/416, conditional requests), explicit content types, `no-cache`, behind the token guard.
- `@se/protocol`: `CoreClient.endpoint`, `assetMediaPath`, `mediaUrl`, `PEAK_LEVELS`, `decodeWaveformPeaks`. `@se/engine/capabilities`: codec strings (avc1, hvc1/hev1, vp09, av01, vp8, mp4a, opus, flac, PCM…) and `checkDecodability`.
- Electron: `chooseFiles` (native multi-select with filter, validated IPC) and `getPathForFile` (`webUtils`). Renderer: `shared/core` `useCoreEndpoint`, `shared/text` (`searchable`, moved from projects), `shared/time` (duration display); `features/media-bin` (import button and OS drop with overlay, grid/list, poster + hover scrub through the sheet, waveform previews, kind filter, accent-insensitive search, job progress/failure indicators, details panel, « Manquant » badge, « Relier… » with confirmation for other content, remove/reveal menus, decodability reporter, toasts); history labels and error messages translated.
- Tests: se-domain 165 + properties (generator covers fingerprints/decodability), se-store migrations, se-app 19 unit + 5 job-queue + 9 media-service, se-media 26 unit + 5 fixtures (ffprobe JSON recorded from ffmpeg-generated H.264, HEVC Main 10 rotated, ProRes 422 HQ + PCM, WAV, PNG) + 3 with real ffmpeg (sheet/poster/peaks of a video, an image, a refused text file, cancelling a 4K job), se-core router/route unit tests + 3 process tests (`/media` token 401, import → thumbnails/peaks served with Range 206/416, rename → missing on reopening → relink), protocol/engine/desktop Vitest (codec strings, decodability, peaks decoder, URLs, job reducer, scrub/waveform maths, jobs sync, panel behaviour with drop).
- Verified in the built app on Windows (isolated profile, CDP): dropping « Plage 4K.mp4 » (3840×2160), a WAV and a PNG imports 3 media with a visible progress bar on the 4K card, then poster, hover scrub through the 12-tile sheet, waveform and image thumbnail; dropping the same file and a copy reports « 2 médias déjà présents » and imports only the new one; WebCodecs reports H.264/AAC decodable (« Lecture directe : Oui »); renaming the WAV then reopening shows « Manquant » (relinking itself checked by the process test: the native dialog cannot be driven over CDP). No CSP or core warnings in the logs.
- Docs: ADR-0018, `docs/architecture.md` §3/§4.2/§4.4/§5.3/§5.4/§6/§7/§11, glossary (13 terms), backlog (Media section), CLAUDE.md of se-media (cache formats, `SEPK`, jobs), se-app, se-core, se-protocol, se-domain, se-store, `@se/protocol`, `@se/engine`, desktop and root.

**Deviations from the prompt**
- Thumbnails and peaks are **not** recorded in the document: they are cache keyed by content, regenerated when missing (ADR-0018); mezzanines/proxies stay in `DerivedMedia` for prompt 09.
- Decodability is per stream (the prompt 05 TODO), which changed the document format: schema version 2 with a migration. `ReportDecodability` also carries the `projectId`.
- Job events carry `{ job: JobSnapshot }` (a flattened snapshot's `kind` overwrote the event tag — caught by the e2e process test); an extra `jobCancelled` event.
- Peaks progress is computed from the samples read: ffmpeg's stdout carries the PCM, so `-progress pipe:1` is used only for thumbnails. Peaks keep at most 2 channels.
- `/media` thumbnail routes are `/thumbnails` (JSON), `/thumbnails/sheet` and `/thumbnails/poster` rather than `/thumbnails/{index}`.
- The user-configured tools folder is the library setting `media.toolsDir` read at startup (no UI before prompt 09's settings).
- Removing an asset does not cancel its jobs (their cached outputs serve an undo).

**Follow-ups**
- Prompt 09: mezzanine policy from the per-stream decodability, proxies, analyses (new job kinds in `JobKind` + runner), the settings screen (`media.toolsDir`, cache cleanup).
- Prompts 10/11/14: draw waveforms in the timeline from `decodeWaveformPeaks` (`media-bin/services/waveform-path.ts` can move to `shared`), drag media from the bin to the timeline.
- Backlog (Media section): bump the Linux ffmpeg build before 2028-10, macOS signing/notarization and GPL source offer for distributed installers, several sound streams.
- `release.yml` macOS/Linux packaging with the bundled tools is still unverified (needs a tag run).

**Notes for the next agent**
- ffmpeg's `metadata=mode=print` only prints frames that carry metadata: `metadata=mode=add:key=…` first. Filter file paths are relative to the run's working folder (no Windows escaping).
- `-display_rotation` on a lavfi input rotates the pictures themselves (autorotate); to generate a rotated test file, remux with `-display_rotation:v:0 N -i in.mp4 -c copy`.
- Real OS drag and drop over CDP: `Input.dispatchDragEvent` with `data.files` (absolute paths) — the page gets `File`s that `webUtils.getPathForFile` resolves.
- Heredocs in the Bash tool can collapse backslashes and choke on backticks: write scripts with the Write tool instead.
- The ffprobe fixtures were recorded with ffprobe 9.0.2; `format.filename` is stripped to the base name.

## 2026-11-07 — Prompt 07: Persistance et gestion des projets — Done

**Branch**: `feature/07-persistance-projets`, merged into `dev` at `b17b02a`

**Done**
- `se-protocol`: commands `getNewProjectDefaults`, `createProject`, `openProject`, `closeProject`, `listRecentProjects`, `renameProject`, `duplicateProject`, `trashProject`, `getProjectSnapshot`, `createSequence`, `undo`, `redo`; events `projectOpened` (session), `projectPatched`, `projectClosed`, `historyChanged`; normalised DTOs (`ProjectSnapshot`, `ProjectHeader`, `SequenceEntity { trackIds, markerIds }`, `TrackEntity { sequenceId, itemIds }`, `ItemEntity`/`MarkerEntity` with the domain type flattened, `ProjectPatch { upserts, deletes }`, `ProjectSession`, `HistoryState`, `RecentProject`, `NewProjectDefaults`); error codes `PROJECT_NOT_FOUND`, `PROJECT_LOCKED`, `SCHEMA_TOO_NEW`, `PROJECT_CORRUPTED`, `PROJECT_ALREADY_EXISTS`, `PROJECT_NOT_OPEN`, `STORAGE_FAILED`, `EDIT_REJECTED` (`details.domainCode`), `NOTHING_TO_UNDO`, `NOTHING_TO_REDO`.
- `se-app`: ports `ProjectRepository` + `ProjectStore`, `AppLibraryRepository`, `EventSink`, `Clock`, `IdGenerator` (+ `StorageError`, `JournalEntry`) with in-memory fakes (`test-support`); `ProjectActorHandle` (commit on a draft → journal → install → `projectPatched` + `historyChanged`; 2 s debounced snapshot, retried on failure, saved on close or when every handle drops); `ProjectService` (create with V1/V2/A1/A2, open with journal replay + immediate recovered snapshot, idempotent re-open, same-id copy refused, close, close all, recent list with availability, rename/duplicate through temporary silent actors, trash, create sequence, undo, redo); `project_view` (snapshot and patch builders refreshing parents), `recovery::replay`, `AppError` → protocol codes.
- `se-store`: `FsProjectRepository`/`FsProjectStore` (`<Name>.seproj/`: atomic pretty `project.json` with 3 rotating backups, `project.db` WAL + `synchronous=FULL` with schema v1 `op_journal`, `tasks`, `task_events`, `transcripts`, `transcript_search` FTS5, `analyses`; `.lock` held with `File::try_lock`; `cache/`, `exports/`), document migrations (`migrate_document`, v1 base, `tests/fixtures/project-v1.json`), SQLite `user_version` migrations, journal `format_version`, duplication by `VACUUM INTO` without journal, OS trash (NSFileManager on macOS), `SqliteAppLibrary` (`library.db`: recent projects, settings).
- `se-core`: async router, one task per request, project handlers (`rpc/handlers/projects.rs`, `editing.rs`), `SystemClock`/`UuidV7Ids`/`EventSink` for the bus, `library.db` in `SE_DATA_DIR`, projects proposed in `<Documents>/SimpleEffect`, every open project saved before `coreShuttingDown`. Process tests: commit events, a commit surviving `kill`, the lock between two cores, the shutdown save.
- Renderer: `shared/project-store` (Zustand mirror, `applyPatch` with stale/gap/foreign outcomes, `connectProjectSync`: resync on gap, `resyncRequired` and reconnection, reopen after a core restart), `shared/shortcuts` (registry, provider, `useShortcut`), `shared/desktop` (bridge by context), `shared/routing`, `shared/errors` (every code translated, compile-time exhaustiveness); `features/projects` home screen (recent grid, accent-insensitive search, context and « ⋯ » menus: open, rename, duplicate, reveal, trash with confirmation; « Nouveau projet » dialog with name, location + native picker, `AspectRatioPicker`, resolutions per ratio or custom, frame rates from the core; « Ouvrir un projet »); editor loads the URL's project (or reopens it from the recent list), header with project and sequence names, undo/redo buttons and Ctrl/⌘+Z, Ctrl/⌘+Maj+Z; going home closes the project. Electron: `chooseFolder`, `showInFolder` with validated IPC arguments. The dev preview link is gone.
- Tests: se-protocol 23, se-app 31 (actor with paused clock, service), se-store 22 (+1 ignored real-trash test, run once by hand), se-core 30 unit + 15 process, desktop 83 (mirror, sync, shortcuts, errors, form services, dialog validation, editor loading, shell).
- Verified in the built app (macOS, isolated profile, CDP): create → editor; undo/redo by keyboard after a commit; snapshot written after 2 s with backups; home grid with badge and date; duplicate, rename (context menu), trash (folder in the macOS Trash); renderer reload restores the project and its undo state; closing the window saves and stops the core cleanly.
- Docs: ADR-0017, `docs/architecture.md` §3/§4.2/§4.4/§5.5/§5.6/§6/§11, glossary (14 terms), backlog (Projects section), CLAUDE.md of se-app, se-store, se-core, se-protocol, `@se/protocol`, desktop.

**Deviations from the prompt**
- The repository port is split: `ProjectRepository` (folders: locate, create, duplicate, open, trash) and `ProjectStore` (one open, locked project: load snapshot, read/append journal, save snapshot, close), so nothing writes a project it has not locked (ADR-0017).
- `library.db` lives in `SE_DATA_DIR` (Electron's `userData`), not a `dirs` location; `dirs` is not used.
- Extra command `getNewProjectDefaults` (proposed folder = last used, presets, `FrameRate::STANDARD`): the dialog takes the domain's lists from the core.
- The default sequence is named after the project; renaming changes the document name, not the folder; a copy is named by the renderer (« X (copie) ») and its folder numbered (`X 2.seproj`); trashing a project whose folder is gone only forgets it.
- Undo history is in memory (lost on close). Labels of core-made transactions are English identifiers translated by the renderer (`history-label.ts`).
- The `Asset::relative_hint` relinking and `missing` flag on open (prompt 05 follow-up) are left to prompt 08: there are no assets yet.
- Killing se-core inside the running app was not done by hand (the sandbox refused `kill`); the scenario is covered by the process test `a_commit_survives_killing_the_core` (kill right after a commit, restart on the same data folder, journal replayed).
- zustand 5.0.16 (5.1.0 was less than a day old). The editor header also got undo/redo buttons.

**Follow-ups**
- Prompt 08: relink missing assets on open (`missing`); media import as transactions through `ProjectService`/actor.
- Prompt 10: enrich `shared/shortcuts` (scopes, listing in the UI); the timeline reads the mirror with `useProjectSelector`.
- Prompt 11: a generic « apply transaction » use case/command for timeline edits, with UI-provided labels.
- Prompt 16: choose the active sequence (`selectActiveSequence` is the first one, `TODO(prompt-16)`).
- Prompts 09/17/21: refine `analyses`, `tasks`/`task_events`, `transcripts` through new `project.db` migrations.
- Backlog: persistent history, corrupt `library.db` recovery, non-UTF-8 paths, folder rename, single-instance app.

**Notes for the next agent**
- Commit flow and recovery invariants: journal before install; `se_app::recovery::replay` uses `commit` (deterministic); a snapshot truncates the journal. Use `InMemoryProjectRepository::folder(path)` to inspect what an actor wrote, and `fail_journal_appends`/`fail_snapshot_saves` to inject failures.
- clippy (toolchain 1.99) flags `assert!(x.is_empty())` (`assert_is_empty`): write `assert_eq!(x, [])`.
- `RunningCore::start_in(dir)` restarts a test core on the same data folder; `call()` returns a response plus the events pushed before it (events usually precede the response).
- The built-app recipe (CDP on port 9223, reload with `ignoreCache`) is in `apps/desktop/CLAUDE.md`. Prettier formats JSON fixtures under `crates/` too.
- The previous session had left the prompt 06 merge unpushed on local `dev` (6 commits); it was pushed with this merge.
- Windows CI caught two things macOS/Linux cannot (fixed on `dev` after the merge, `9bc27ae` and `1c612e6`): compare paths as `Path`s in tests (`Path::join` writes `\`), and code under `#[cfg(not(unix))]` is only linted on Windows (clippy pedantic `unnecessary_wraps` on a stub returning `Ok(())`).
- Pushing from an agent shell needs the GitHub token in the macOS keychain (`osxkeychain`), reachable only with the sandbox disabled; the user typed it once in their own terminal.

## 2026-11-07 — Prompt 06: Domaine : opérations, transactions, historique, changesets — Done

**Branch**: `feature/06-domain-operations`, merged into `dev` at `792f0b1`

**Done**
- `se-domain::operation`: `Operation` (internally tagged by `kind`, one payload struct per variant, `From` impls and dispatch generated by `operation_payloads!`) with 26 operations: `AddItem`, `RemoveItem`, `RestoreItem`, `MoveItem`, `TrimItem`, `SplitItem`, `UpdateItem` (`ItemPatch`: one typed patch per item kind), `SetKeyframes` (`PropertyAnimation`: position, scale, rotation, anchor, opacity, volume), `LinkItems`, `UnlinkItems`, `RippleDeleteRange`, `InsertGap`, `AddTrack`, `RemoveTrack`, `UpdateTrack`, `ReorderTracks`, `AddSequence`, `RemoveSequence`, `UpdateSequence`, `AddMarker`, `RemoveMarker`, `UpdateMarker`, `AddAsset`, `UpdateAsset`, `RemoveAsset`, `RenameProject`. `apply` is pure, validates the result, records the exact inverse and the `TouchedEntities`; `expectedRevision` on single-item operations.
- Shared plumbing: `Edit`/`Effect`/`run_steps` (composite edits as primitive steps whose inverses replay backwards), `lookup`, `checks`, `placement` (the only code moving items; keeps tracks sorted), `reshape` (keyframes stay at their instants, fades kept on trimmed edges and dropped on cut edges, media source continuity, source bounds).
- `transaction`: `Transaction`, `Actor`, `commit` (all or nothing; project revision +1, touched items +1), `TransactionId`.
- `history`: `History` (user transactions only, depth 200, labels, failing undo keeps its entry, no-op transactions skipped).
- `changeset`: `Changeset`/`ChangesetEntry` (`touchedRevisions`), `plan_revert` dry run with `Conflict::{ItemChanged, ForeignItems, CannotApply}`, `RevertPlan::into_transaction` (user transaction, undoable).
- `error`: `DomainError` (29 variants, 26 codes; messages with ids and `HH:MM:SS.mmm` times and the fix), `DomainErrorCode` via `code()`.
- Validation: new codes `ITEMS_NOT_SORTED` and `LINK_GROUP_ACROSS_SEQUENCES` (17 codes); `ItemKind::transform_mut`.
- Tests: 155 unit tests (edge cases of every operation, each checked for validity and exact inverse by `testing::apply_ok`), insta snapshots of a forward and an inverse transaction (journal format), `tests/operations/` proptest (1000 cases each: invariants + `apply ∘ inverse = identity`, undo/redo walks with growing revisions, undisturbed changeset revert = initial state) plus a non-vacuity test (every operation kind succeeds).
- Docs: ADR-0016 (+ index), `docs/architecture.md` §5.2/§5.3/§5.4/§15, glossary (12 terms added, 4 refined), backlog (3 items), `crates/se-domain/CLAUDE.md` (API, layout, rules, recipe "add an operation").

**Deviations from the prompt**
- `SetKeyframes { item, property, keyframes }` became `SetKeyframes { item, animation: PropertyAnimation }`: the property and its value travel together, typed (`Vec2` for position/scale/anchor, number otherwise), and the value may be static.
- Extra operation `RestoreItem` (inverse of every item-field edit, exact by snapshot); `RippleDeleteRange` and `InsertGap` take `splitIds` (an item spanning the deleted range is split, which needs an id); `Add*` operations take an `index` so removals restore positions exactly.
- Revisions are bumped by `commit`, not `apply` (`apply ∘ inverse` is exactly the identity); details and the revert semantics beyond the prompt (`ForeignItems`, `CannotApply`, dry run) in ADR-0016.
- `DomainError` has no transaction-level wrapper: `commit` returns the failing operation's error unchanged.
- Operation types derive `TS` but are not exported to TypeScript (the renderer receives entities, ADR-0005).

**Follow-ups**
- Prompt 07: `ProjectActor` calls `commit`, journals `Transaction` JSON (format fixed by `src/snapshots/*transaction*.snap`), records user transactions in `History` and AI ones in their `Changeset`, builds `ProjectPatched` from `TouchedEntities`.
- Prompt 11: link-aware edits (move/trim linked items together) as use cases emitting one operation per item.
- Prompts 24/25: operations on templates and the brand kit (`TODO(prompt-24)`, `TODO(prompt-25)` on `Operation`).
- Backlog: revisions on non-item entities for revert conflicts, "skip" for a stuck undo, ripple edits moving markers.

**Notes for the next agent**
- `operation::testing::apply_ok(project, op)` checks validity and the exact inverse: use it in every new operation test.
- Integration tests use `test_support` through a self dev-dependency (`se-domain = { workspace = true, features = ["test-support"] }`).
- `pnpm` is not on the PATH of a non-interactive shell on the macOS machine: Node 24 lives in `~/.local/share/node-v24.22.2-darwin-arm64/bin` (prepend it to `PATH`).
- zsh expands a leading `=` (`echo ====` fails with "not found"): quote such strings in shell commands.

## 2026-11-06 — Prompt 05: Domaine : temps et modèle de document — Done

**Branch**: `feature/05-domain-model`, merged into `dev` at `71d1014`

**Done**
- `se-domain::time`: `Flicks` (±`MAX_SAFE` = 2^53 − 1, checked `add/sub/mul/mul_f64`, `from/to_seconds_f64` nearest-flick, `from_frames`/`to_frames` with `RoundingMode` Floor/Ceil/Nearest, validating `Deserialize`), `FrameRate` (reduced fraction, 1–1000 fps, exact `frame_duration`, 9 standard constants, `is_ntsc`, `nominal_fps`, `Display` like `29.97`), `TimeRange` (half-open, duration ≥ 0 and end in range enforced at construction and deserialization, `contains`, `contains_range`, `intersects`, `intersection`, `shifted`), `parse_timecode` (`51s`, `51.5s`, `00:51`, `1:02.500`, `1:00:00`, `HH:MM:SS:FF`, `;FF` read as non-drop, leading `-`, bare numbers refused; errors `'1:75': seconds must be < 60`) and `format_timecode` (`HH:MM:SS:FF`, nominal-rate frame counting).
- Shared vectors `docs/test-vectors/flicks.json` and `timecode.json` (values cross-checked with an independent BigInt computation), run by `crates/se-domain/tests/test_vectors.rs`; proptest in `tests/time_properties.rs` (frames ↔ flicks at every standard rate with all roundings, floor containment, SMPTE round trip, `TimeRange` commutativity/containment/shift).
- `ids`: 10 UUIDv7 newtypes (`…, LinkGroupId, MarkerId`) over `uuid::Uuid` (`Copy`), strict hyphenated v7 parsing (`IdError`), string serde, `from_uuid`, `from_test_index` (feature `test-support`).
- `spatial` (`Vec2`, `DesignPixels`, `design_scale`), `color` (`Color`, `#rrggbb[aa]`).
- `document`: `Project`, `ProjectSettings`, `Asset`/`AssetKind`/`Decodability`, `MediaInfo`/`VideoStreamInfo`/`AudioStreamInfo`, `DerivedMedia`/`DerivedFile`/`DerivedStatus`, `Sequence`, `Format`/`FormatPreset`, `Track`/`TrackKind`, `Item`/`Origin`, `ItemKind` (`MediaClip` with `Reframe`, `TextItem`/`TextStyle`, `ShapeItem`/`ShapeGeometry`, `ImageItem`, `TemplateItem`/`TemplateParamValue`, `CaptionItem`), `Transform2D`, `Animatable`/`Keyframe`, `Easing`/`EasingPreset`, `Marker`/`MarkerColor`, `MotionTemplate`, `BrandKit`; `CURRENT_SCHEMA_VERSION = 1`.
- `validation`: `validate_project` → `Vec<Violation { code, path, message }>`, 15 `ViolationCode`s (table in `docs/architecture.md` §5.3), paths like `sequences[<id>].tracks[<id>].items[<id>].kind.speed`, at least one test per code.
- `test_support` (feature `test-support`): `example_project()` (valid, every item kind, keyframes, link group, AI origin, marker, template, brand kit), `example_ids`, `millis`, `range_ms`, `item`, `track`. insta snapshot of its JSON fixes the document format.
- `se-codegen` exports `Project`, `Violation`, `FormatPreset` roots (69 generated files); `Flicks` is `number & { readonly __brand: 'Flicks' }`, ids are `string`. `packages/protocol/src/document-types.test.ts` adds compile-time checks.
- Docs: ADR-0015 (spatial units) + index, `docs/architecture.md` §2/§3/§5.1–5.3, glossary (14 terms), backlog (drop-frame timecodes), CLAUDE.md of `se-domain`, `se-codegen`, `@se/protocol`.

**Deviations from the prompt**
- ADR number: the prompt asked for « ADR 0012 Spatial units », but 0012 was already taken (RPC conventions, prompt 02): it is **ADR-0015**.
- Design pixels are measured on the **shorter side** = 1080, not the height (user's choice, recorded in ADR-0015): 1 design px = 1 px in 16:9, 9:16, 1:1 and 4:5 at 1080, 2 px in 4K.
- `MAX_SAFE` is 2^53 − 1 (JavaScript's `Number.MAX_SAFE_INTEGER`) rather than 2^53, which JS cannot distinguish from 2^53 + 1.
- `MediaClip` has no separate `opacity`: it lives in `Transform2D` (the prompt listed both). Volume is `volume_db: Animatable<f64>` so automation is keyframes.
- Single-value invariants (flicks range, `TimeRange` duration ≥ 0, frame rates, ids, colours) are enforced by the types, including at deserialization; `validate_project` checks only cross-entity rules. Hence no violation code for « negative duration ».
- `FrameRate` only represents rates of 1–1000 fps with a whole number of flicks per frame; `VideoStreamInfo::frame_rate` is `None` for others (variable-frame-rate footage goes through a mezzanine).
- Extra validation codes beyond the prompt's list: `DUPLICATE_ID`, `ASSET_KIND_MISMATCH`, `INVALID_FORMAT`, `EMPTY_ITEM`, `EMPTY_KEYFRAMES`, `VALUE_OUT_OF_RANGE`.
- `Easing::Preset` holds an `EasingPreset` enum (ease-in/out/in-out) instead of a free string, so the engine never meets an unknown name. `Reframe` is a `kind`-tagged enum (`{"kind":"fit"}`) so prompt 26 can add a variant with data without changing the format.
- Timecode API is `parse_timecode` / `format_timecode` (functions), not a `Timecode` type; ADR-0004's mention of `Timecode::parse` predates this.
- `MediaInfo` has `video_stream: Option<…>` and `audio_streams: Vec<…>` (several audio streams in pro files) and `duration: Option<Flicks>` (images).
- Template parameter values cover text, number, boolean and colour; enum, font and asset parameters wait for prompt 24.

**Follow-ups**
- Prompt 06: operations must keep track items sorted by start (documented on `Track::items`, not validated), bump `Item::revision` and `Project::revision`, and run `validate_project` on the result.
- Prompt 07: `schema_version` migrations; `Asset::relative_hint` relinking; `missing` flag on open.
- Prompt 08: fill `MediaInfo`/`Decodability` (`TODO(prompt-08)` on per-stream decodability).
- Prompt 10: TypeScript flicks/timecode helpers must run `docs/test-vectors/*.json` (same error codes, camelCase variant names).
- Prompts 15, 23, 24, 25, 26: complete `TextStyle`/`ShapeItem`/easing presets, `CaptionItem`, `MotionTemplate`/params, `BrandKit`, `Reframe` (`TODO(prompt-NN)` in code).
- Link groups are not validated (e.g. members on different sequences): add a rule when operations create them (prompt 06/11).

**Notes for the next agent**
- ts-rs 12 (`serde-compat`) prints fake « failed to parse serde attribute » warnings for `#[serde(try_from)]`, `transparent`…: validated types implement `Deserialize` by hand (private `Raw…` struct + constructor) instead. Container-level `#[ts(type = "…")]` gives custom TS types (`Flicks`, ids, `Color`).
- Internally tagged enums need struct variants (`Static { value }`), not newtype variants around primitives; add `rename_all_fields = "camelCase"` for multi-word variant fields.
- Accept a changed snapshot with `INSTA_UPDATE=always cargo test -p se-domain` after reviewing the `.snap.new` (ignored by Git).
- clippy: `UUIDv7` is in `doc-valid-idents`; test files outside `#[test]` functions need `#![allow(clippy::unwrap_used/expect_used)]` (see `tests/*.rs`); `case_sensitive_file_extension_comparisons` fires on `ends_with(".pan")`.

## 2026-11-06 — Prompt 04: Design system : composants montage, shell, i18n — Done

**Branch**: `feature/04-design-system-editeur-shell`, merged into `dev` at `bf5f752`

**Done**
- `@se/ui` editor widgets (`packages/ui/src/editor/`, Storybook « Editor », 17 folders, stories + tests each): Panel (named region from its title or `aria-label`), Dock/DockPanel/DockHandle (`react-resizable-panels` 4.18, nestable, user resizes saved in an injected `DockLayoutStorage` under `dock-layout.v1.<layoutId>`, corrupt entries ignored), Toolbar (Radix toolbar: one tab stop, arrows inside, `pressed` toggles), PropertyGroup/PropertyRow (collapsible, keyframe slot, modified mark), ScrubInput (drag, Shift ×10, Alt ×0.1, arrows, double-click/Enter/digit typing with decimal comma, injected format/parse, **one `onCommit` per gesture**), TimecodeInput/TimecodeDisplay (injected `format`/`parse`, invalid → `aria-invalid`, Escape restores), ColorField (OKLCH sliders with live gradients, OKLCH and hex fields, EyeDropper when present, recent colours, `onChange`/`onCommit`), KeyframeButton (3 states, prev/next), EasingCurveEditor (draggable + arrow-key handles, presets linear/ease-in/ease-out/ease-in-out/back/hold, animated preview), TransportBar, Meter (dBFS zones, peak lines, pure `updatePeakHold`), AspectRatioPicker (16:9, 9:16, 1:1, 4:5, custom with thumbnails to scale), StatusDot, EmptyState, KeyboardShortcut (⌘⌥⇧ on Apple, localisable names elsewhere), AiTaskCard (5 statuses, log, error, cost, actions), PromptComposer (auto-growing, removable scope chips, Backspace removes the last, Ctrl/⌘+Enter sends, model/effort slots).
- Shared helpers: `usePointerDrag` (pointer capture, per-move deltas so modifiers can change mid-drag, one `onEnd` per press), `lib/platform.ts` (the only OS check). Colour maths gains `srgbToOklch` and `parseHex`. New token `animate-easing-preview`. Storybook wraps stories in a `TooltipProvider`.
- Renderer: i18next + react-i18next (`src/i18n/`, synchronous bundled `fr.json` grouped by `app`, `home`, `editor`, `statusBar`, `common`; keys typed through `CustomTypeOptions`), `eslint-plugin-i18next` `no-literal-string` (`jsx-only`, user-visible attributes listed, tests exempt; checked with deliberate violations). React Router 8 in library mode, hash router: `/` home (empty state, disabled « Nouveau projet », dev-only « Ouvrir l’éditeur » preview link), `/project/:projectId` editor (Médias | Viewer | Inspecteur+IA tabs over Timeline, every panel an explicit empty state), status bar (core connection as a polite `status`, « IA non configurée », version from `__APP_VERSION__`). `AppProviders` mounts `I18nextProvider`, `CoreClientProvider`, `TooltipProvider` and the `Toaster`. Panel sizes go to `localStorage` under `se.ui.` (`shared/storage/`). The temporary connection screen of prompts 01–02 is gone.
- Features created with their panel only: `projects`, `media-bin`, `viewer`, `inspector`, `ai`, `timeline` (each with a `TODO(prompt-NN)`).
- Tests: 461 in `ui` (axe on every story, all new stories included), 40 in `desktop` (shell routes and panels, saved layout restored, status bar, i18n completeness, storage adapter, connection indicator). ESLint forbids `react-resizable-panels` in the renderer.
- Verified in the built app (Electron, isolated profile, CDP): home and editor render, dragging handles and arrow keys on a focused handle resize panels, sizes are saved and identical after quitting and relaunching; core connects (« Cœur connecté »).
- Docs: ADR-0014, `docs/architecture.md` §4.1–4.2, glossary (editor widget, panel/dock, layout id, gesture commit, shell, translation key), `packages/ui/CLAUDE.md` (editor inventory, conventions, testing gotchas), `apps/desktop/CLAUDE.md` (routing, i18n, how to add a key, tests).

**Deviations from the prompt**
- Panel-size persistence uses our own `dock-layout-storage.ts` instead of the library's `useDefaultLayout`, so loading and saving are pure, unit-tested and save only user-made resizes (`requestedLayout`).
- `EasingCurveEditor` uses its own presentational `EasingCurve` type (`cubicBezier` | `hold`): the domain easing type arrives with prompt 15 and the app will map it.
- `ColorField` edits in OKLCH with three gradient sliders rather than a 2D area (keyboard-accessible out of the box); its value is a `#rrggbb` string (no alpha yet).
- Meter peak hold is computed by a pure helper the audio engine calls (`updatePeakHold`), not inside the component: values stay in props as the prompt asks, and the widget has no clock.
- A dev-only link on the home screen opens the editor (`/project/preview`) so the shell can be tried before projects exist (TODO(prompt-07)).
- Optional props of editor widgets are typed `?: T | undefined` for `exactOptionalPropertyTypes` consumers.

**Follow-ups**
- Prompt 07: enable « Nouveau projet », load the project of `/project/:projectId`, remove the preview link.
- Prompts 08/10/12/15/20: fill the panels (media bin, timeline, viewer + TransportBar, inspector property groups, AI composer and task cards) with the widgets of this prompt; send document commands from `onCommit` only (ADR-0014).
- `ColorField` has no alpha channel; `ScrubInput` has no expression evaluation (`=10*2`) nor pointer-lock infinite drag: add if a property needs them.
- Colour contrast of the new stories was not re-run in a real browser (axe in jsdom skips it); they use existing token pairs already covered by `contrast.test.ts`.

**Notes for the next agent**
- jsdom has no layout: `react-resizable-panels` sees 0 px, so resizing cannot be unit-tested, and clicks inside a `Dock` are taken for resize grabs (pointer-down default prevented): use the keyboard for tabs/buttons inside docks in tests. Do not mock `offsetWidth` to fix it: the library then fails to find panels.
- A shell test can use a real `CoreClient` whose endpoint resolver never resolves: it stays « connecting » and opens no socket (`app-shell.test.tsx`).
- To try the built app without disturbing a running `pnpm dev`: `electron-vite build --outDir <tmp>/out`, add a `package.json` with `"type": "module"` there, then `electron <tmp>/out --user-data-dir=<tmp>/profile --remote-debugging-port=9223` with `SE_CORE_PATH` pointing at `target/debug/se-core.exe`; Electron honours `--user-data-dir`.
- A running `pnpm dev` locks `target/debug/se-core.exe` on Windows, which makes `cargo test` in `pnpm check` fail with « Accès refusé »: close the app or use another `CARGO_TARGET_DIR`.

## 2026-11-06 — Prompt 03: Design system : fondations — Done

**Branch**: `feature/03-design-system-fondations`, merged into `dev` at `66efd7f`

**Done**
- `@se/ui` tokens (`src/tokens/*.css`, Tailwind v4 `@theme static`, OKLCH): surfaces (`bg`, `panel`, `panel-raised`, `overlay`, `input`, `control`, `control-hover`, `border`, `border-strong`, `hover`, `scrim`), text, accent (bright yellow, dark `accent-foreground`; `accents.css` keeps the `lemon`/`yellow`/`gold` shades under `[data-accent]`), semantic (+ muted, `danger-solid*`), item kinds and AI (+ muted/strong), 4 px spacing, control heights `h-control-xs|sm|md`, radii, `shadow-overlay`, type scale 11–20 px, weights, leading, easing, durations, keyframes (overlay/content in-out, progress sweep), z-index layers. Default Tailwind palette and scales removed. Structure ready for a `[data-theme='light']` override.
- `src/styles.css` (the only stylesheet apps import): Tailwind, bundled Inter Variable + JetBrains Mono (fontsource), tokens, base styles (dark colour scheme, focus-visible safety ring, thin scrollbars, `timecode` utility, reduced motion).
- Utilities: `cn()` (clsx + tailwind-merge taught our custom tokens), `focusRing`, `floatingSurfaceClasses`, `Slot`/`Slottable`; `@se/ui/icons` entry (lucide re-export) + `Icon` (12/14/16/20 px, decorative unless labelled).
- 28 primitives in `src/primitives/<name>/{name.tsx, .stories.tsx, .test.tsx, index.ts}`: Button, IconButton, Icon, Input (+ shared `textFieldClasses`), Textarea, Label, Field (Slot-based wiring of id/aria-describedby/aria-invalid), Select, Checkbox (indeterminate), Switch, Slider (`thumbLabels`, range), RadioGroup, Tooltip (+ `TooltipProvider`, shortcut key caps), Popover, Dialog (sizes, opt-in close button via `closeLabel`), AlertDialog, DropdownMenu, ContextMenu (shared `menu/menu-styles.ts`), Tabs (`segmented`/`underline`), ToggleGroup, Separator, ScrollArea, Kbd, Badge (semantic + item kinds), Toast (`Toaster` + `toast`, sonner `unstyled` with tokens), Progress (determinate/indeterminate, tones), Spinner, Skeleton.
- Storybook 10 (`pnpm storybook`, Vite builder, dark manager/docs, a11y + docs addons, telemetry and update checks off): "Foundations/Principles" (MDX), "Foundations/Tokens" (generated from the token stylesheets: swatches with hex and contrast, type scale, spacing, radii, layers, motion), "Foundations/Theme" (the 3 yellow shades side by side), every primitive with variants and states.
- Tests (Vitest jsdom + Testing Library + user-event, 247 in `ui`): behaviour of Select, Dialog, DropdownMenu, Tabs, ToggleGroup, Slider (keyboard, focus, open/close) and of every other primitive; `src/stories.a11y.test.tsx` runs axe (WCAG 2.2 A/AA) on all 109 stories; `tokens/contrast.test.ts` asserts that every colour is inside the sRGB gamut and that text is AA on every surface for every accent shade (OKLCH → sRGB in `color-math.ts`). axe colour-contrast also ran once in a real browser on all 109 stories: no violation.
- Raw-colour guard: `scripts/check-raw-colors.mjs` (+ `scripts/lib/raw-colors.mjs`) in `pnpm check`, scanning `packages/ui/src` and `apps/desktop/src` outside `tokens/`; tested with fixtures (new Vitest project `scripts`). ESLint forbids `radix-ui`, `@radix-ui/*`, `lucide-react`, `sonner`, `class-variance-authority` in the renderer; react-hooks rules now cover `packages/ui`.
- Desktop: `styles.css` imports `@se/ui/styles.css`; the temporary status screen uses tokens; CSP `style-src` allows inline `<style>` (Radix, sonner).
- Docs: ADR-0013, `packages/ui/CLAUDE.md` (anatomy, tokens cheat sheet, how to add a component, accent), `docs/architecture.md` §4.1/§4.4/§13, glossary (design token, accent, primitive), desktop and root `CLAUDE.md`.

**Deviations from the prompt**
- Accent: the prompt asked for an indigo-violet default with 3 variants. Indigo, violet and blue were presented in the Theme story; the user chose **yellow**, then the "Jaune vif" shade (OKLCH 0.87 0.175 96) over "Citron" and "Or". Consequences: `accent-foreground` is dark; `warning` moved from amber to orange (hue 65) and `item-caption` from yellow to violet (hue 290), `ai` to magenta (335), so nothing else reads as the accent.
- Extra tokens beyond the list: `control`, `control-hover`, `hover`, `scrim`, `danger-solid`, `danger-solid-hover`, `danger-foreground` (secondary buttons, ghost hovers, modal backdrop and filled destructive buttons need them to stay AA).
- Z-index order is `panel < modal < dropdown < toast < tooltip` (the prompt listed `dropdown` before `modal`): a select or menu opened inside a dialog must float above it.
- Production CSP now allows inline styles (`style-src 'self' 'unsafe-inline'`), see ADR-0013.
- Accessibility in CI is checked with axe on portable stories in jsdom (no Playwright browser download); colour contrast is covered by the token test and was checked once in a real browser with axe on all stories.
- `scripts/not-yet-available.mjs` removed (no remaining placeholder script).

**Follow-ups**
- Prompt 04: mount `TooltipProvider` and `<Toaster label=…>` at the app root, add i18n strings, build `editor/` widgets on `textFieldClasses`, `floatingSurfaceClasses` and the tokens.
- ContextMenu keyboard opening (Shift+F10 / context-menu key) is not unit-tested: jsdom does not synthesise the `contextmenu` event.
- The renderer bundles every fontsource subset (cyrillic, greek, vietnamese…, ~0.6 MB of woff2): fine for a desktop app, revisit in prompt 29 if startup matters.

**Notes for the next agent**
- When a story's `component` is a Radix re-export, type the meta as `const meta: Meta<typeof X> = {…}`: `satisfies` makes `tsc -b` fail with TS2883 (inferred type names Radix internals). Exported Radix values re-assigned in our modules need an explicit type (`typeof SlotPrimitive.Root`) for the same reason (TS4023).
- Vitest blanks CSS imports; `packages/ui/vitest.config.ts` enables CSS for `src/tokens/*.css` so `?raw` imports work in token tests.
- Radix single ToggleGroup renders `radiogroup`/`radio`; RadioGroup selects on arrow key *down* (tests use `{ArrowDown>}{/ArrowDown}`).
- sonner hard-codes a huge z-index: the Toaster uses `z-(--z-toast)!`.

## 2026-11-06 — Prompt 02: Protocole typé et couche RPC — Done

**Branch**: `feature/02-protocole-rpc`, merged into `dev` at `c8ca8dc`

**Done**
- `se-protocol`: `ClientMessage::Request(RequestEnvelope)`, `ServerMessage::{Response(ResponseEnvelope), Event(EventEnvelope)}`, `Command` (`ping`, `getCoreInfo`), `CommandResult` (same kinds; `CoreInfo { version, os, arch, appDataDir, logsDir }`), `CoreEvent` (`coreReady`, `coreShuttingDown`, `resyncRequired`), `ProtocolError { code, message, details? }`, `ErrorCode` (`INVALID_REQUEST`, `UNKNOWN_COMMAND`, `INTERNAL`, `TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED`). All derive `ts_rs::TS` (ts-rs 12); `CoreHandshake` too. Round-trip tests for every envelope.
- `crates/se-codegen` (new, dev tool) + `scripts/codegen.mjs`: `pnpm codegen` writes `packages/protocol/src/generated/` (one file per type + `index.ts`, `GENERATED — DO NOT EDIT` header, `u64`/`i64` → `number`); `pnpm codegen --check` (already in `pnpm check`) compares a fresh export, CRLF-normalised. Verified: adding a field to `CoreInfo` without regenerating fails the check.
- `se-core`: `rpc/` module — `RpcRouter` (parse → `dispatch` → encode; `INVALID_REQUEST` vs `UNKNOWN_COMMAND` with the supported kinds in the message), handlers per group (`rpc/handlers/system.rs`), `EventBus` (bounded `broadcast`, 256; lag → `resyncRequired`), per-connection loop (greets with `coreReady`, forwards `coreShuttingDown` before closing, tracing span per connection). `AppPaths` from `SE_DATA_DIR` (fallback `<OS data dir>/SimpleEffect`); logs: `tracing-appender` daily files `<data>/logs/se-core.YYYY-MM-DD.log`, 7 kept, filter `SE_LOG`, stderr in debug builds or when the folder is unusable. Integration tests moved to `tests/core_process/` (temporary `SE_DATA_DIR` per test; 2 clients receive `coreShuttingDown`; log file exists and never contains the token).
- `@se/protocol`: `CoreClient.connect(source)` (fixed endpoint or resolver called before every attempt), `send` typed by `ResultFor<K>` (derived, no table; compile-time check that every command has a result), per-request timeout (30 s default), `on(kind, handler)`, observable `state` (`connecting | open | reconnecting | closed`), backoff 250 ms → 5 s, queued requests survive reconnection, in-flight ones fail with `CONNECTION_LOST`, `close()` → `CLIENT_CLOSED`. Tested with `FakeWebSocket` + fake timers.
- Electron: supervisor passes `SE_DATA_DIR` = `app.getPath('userData')` (`core-environment.ts`); the handshake parser uses the generated `CoreHandshake` type.
- Renderer: `src/shared/core/` (`createCoreClient`, `CoreClientProvider`, `useCoreClient`, `useConnectionStatus`, `useCoreEvent`, `useCoreInfo` + pure `watchCoreInfo`); the hand-written ping (`shared/core-connection/`) is gone. The screen shows the status, version, platform, data and log folders from `getCoreInfo`.
- ADR-0012 (protocol conventions), `docs/architecture.md` §2/§3/§4.4/§6, glossary, CLAUDE.md of `se-protocol` (add-a-command recipe), `se-core`, `se-codegen`, `@se/protocol`, `@se/desktop`, root.
- Verified on Windows with `electron-vite dev`: screen shows « Cœur connecté — se-core 0.1.0 » plus the folders; `taskkill /F` on se-core → « Reconnexion… » → reconnected to the restarted core (new port) in < 1 s; `%APPDATA%\SimpleEffect\logs\se-core.2026-11-06.log` written; closing the window leaves no process.

**Deviations from the prompt**
- The event enum is `CoreEvent`, not `Event` (the generated TS type would shadow the DOM `Event` in the renderer).
- Results use the same `kind` as their command (`ping` → `{"kind":"ping"}`, no more `pong`): that is what lets `ResultFor` be derived without a table (ADR-0012).
- `CoreClient.connect` is a static factory taking `{ url, token }` **or** a resolver function, because a restarted core has a new port and token.
- `TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED` live in the Rust `ErrorCode` although only clients raise them (one list to match on).
- Log filter variable is `SE_LOG` (was `RUST_LOG` in prompt 01), as the prompt asks.

**Follow-ups**
- React hooks of `src/shared/core/` have no component tests: there is no DOM test environment yet (Testing Library arrives with prompt 03); their logic lives in tested plain functions.
- Handlers are synchronous; the first async handler must make `RpcRouter::handle_text` async and spawn a task per request (noted in `crates/se-core/CLAUDE.md`).
- Nothing reacts to `resyncRequired` or to reconnections yet: prompt 07+ (project snapshot) must refetch state then.
- `release.yml` still unverified on macOS/Linux (unchanged by this prompt).

**Notes for the next agent**
- `pnpm codegen` builds and runs `se-codegen`; it refuses a non-empty output folder because ts-rs merges into existing files.
- `serde(flatten)` on `ResponseOutcome` accepts a frame with both `result` and `error` (first wins): harmless since only se-core produces responses.
- tracing-appender prunes old logs before creating the folder, so `logging.rs` creates `logs/` first (otherwise a spurious error is printed on a fresh install).
- `TaskStop` on a backgrounded `electron-vite dev` leaves Electron running on Windows; close it with `taskkill /PID <main electron pid>` (graceful: se-core stops through its stdin).

## 2026-11-06 — Prompt 01: Bootstrap du monorepo — Done

**Branch**: `feature/01-bootstrap-monorepo`, merged into `dev` at `4d8888a`

**Done**
- Rust workspace: 11 crates (edition 2024, toolchain pinned to 1.99.1), inter-crate dependencies declared per `docs/architecture.md` §3, workspace lints (`unsafe_code` forbidden, `missing_docs`, clippy `pedantic` + `unwrap_used`/`expect_used`/`panic`/`todo`/`dbg_macro` denied; tests allowed through `clippy.toml`).
- `se-core`: axum 0.8 on `127.0.0.1:0`, 256-bit session token, single-line stdout handshake, `/rpc` WebSocket ping/pong with hand-written `se-protocol` DTOs (plus `INVALID_REQUEST`/`INTERNAL` error responses), access-guard middleware (foreign `Host` → 403, missing/wrong token → 401, constant-time compare), graceful shutdown on stdin EOF or Ctrl+C with a 2 s drain bound. `tests/rpc_endpoint.rs` spawns the real binary (handshake, ping, 401, 403, stdin close with and without an open socket).
- pnpm workspace (pnpm 12.18), `tsconfig.base.json` + project references (`tsc -b`), ESLint 10 flat config (`strictTypeChecked`, react-hooks 7 with React Compiler rules, `eslint-plugin-boundaries` 7 policies — checked with deliberate violations), Prettier, Vitest 5 projects per package.
- `apps/desktop`: Electron 45 + electron-vite 6 (Vite 8) + React 19.3 + React Compiler (Babel preset) + Tailwind 4.3. `CoreSupervisor` (handshake parsing, 15 s handshake timeout, backoff 250 ms → 10 s reset after 10 s of stable run, stop = close stdin then kill after 3 s), sandboxed CommonJS preload bridge, hardened window, CSP meta tag generated per mode, renderer status screen with automatic reconnection.
- Scripts (`scripts/*.mjs`, cross-platform): `check`, `dev`, `build`, `package`, `set-version`, `codegen` and `storybook` placeholders.
- Packaging (`electron-builder.yml`: NSIS / dmg / AppImage + deb, `se-core` in `resources/bin`), `ci.yml` (ubuntu + windows, `pnpm check`) and `release.yml` (tags `v*.*.*`, pre-release when the tag contains `-`, notes generated, `contents: write` on the publish job only). `actionlint` 1.7.12: no findings.
- Verified on Windows: `pnpm dev` shows « Cœur connecté — se-core 0.1.0 »; killing `se-core` shows « Reconnexion… » then reconnects to a new core within ~0.5 s; closing the window and force-killing Electron both leave no `se-core`; the NSIS installer installs to `%LOCALAPPDATA%\Programs\simpleeffect`, the app starts the core from `resources\bin` and connects; silent uninstall is clean.

**Deviations from the prompt**
- TypeScript 6.0.3, not the latest 7.0.2: TypeScript 7 (native) has no JS API and `typescript-eslint` supports `<6.1` (gotcha added to the root `CLAUDE.md`).
- Tailwind 4.3.4, not 4.3.5: 4.3.5 was less than a day old and pnpm's `minimumReleaseAge` rejects it.
- `@babel/core` 7.29, not 8: `babel-plugin-react-compiler` 1.0 depends on `@babel/types` 7. The native (oxc) compiler of `@vitejs/plugin-react` 6 is experimental and not used.
- Electron ≥ 45 no longer downloads its binary at install time: the desktop `dev` script runs `install-electron` (idempotent) first.
- `productName: SimpleEffect` in `apps/desktop/package.json` and `extraMetadata.name: simpleeffect` in `electron-builder.yml`, otherwise the install folder is `@sedesktop` and the user-data folder `%APPDATA%\@se`.
- `pnpm check` runs every step and reports all failures instead of stopping at the first one.

**Follow-ups**
- TODO(prompt-02): replace the hand-written ping (`se-protocol` types, `apps/desktop/src/shared/core-connection/`) with generated types and `CoreClient`; implement `pnpm codegen`.
- TODO(prompt-04): move the temporary status strings to i18n and replace `App.tsx` with the shell.
- `release.yml` has not run yet (needs a tag, or `gh workflow run release.yml` once the workflow is on `main`): the macOS and Linux packaging steps are unverified.
- `actionlint` ran without shellcheck (not installed locally).
- The renderer bundle is not minified (electron-vite default); reconsider in prompt 29 if startup time matters.

**Notes for the next agent**
- Remote-debug the renderer: `pnpm --filter @se/desktop exec electron-vite dev --remoteDebuggingPort 9223`, then use the Chrome DevTools Protocol on `http://127.0.0.1:9223/json`.
- Run the core alone: `cargo run -p se-core` prints the handshake; Ctrl+C stops it.
- pnpm 12 may leave paths longer than 260 characters after a failed install on Windows: delete with `rd /s /q "\\?\<absolute path>"`.
