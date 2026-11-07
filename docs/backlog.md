# Backlog — ideas not yet scheduled

Ideas worth keeping that are not part of the current roadmap (`prompts/`). When one is scheduled, write a prompt from `prompts/_TEMPLATE.md` and remove it from here.

## Editing
- Drop-frame timecodes (29.97/59.94 DF): `HH:MM:SS;FF` is currently read as non-drop and formatting is always non-drop (`se_domain::time::timecode`, `docs/test-vectors/timecode.json`).
- Beat detection on music tracks (local onset detection) to snap cuts and animations to the beat.
- Project snapshots / named versions with visual diff between versions.
- Draft/review mode for AI changes (evaluate "document + pending changeset", accept/reject) — the storage model already allows it (ADR-0005).
- Multicam editing (sync by audio waveform).
- Speed ramps with keyframed speed curves.
- Color correction basics (exposure, contrast, saturation, LUT import).
- Transitions library beyond cross dissolve (wipes, zooms, glitch) as motion templates.
- Revisions on tracks, sequences, markers and assets so that a changeset revert can detect a later change of the same field by another actor (today the revert restores the fields the task changed; ADR-0016).
- "Skip" for an undo or redo that no longer applies (the entry currently stays at the top of its stack until the document allows it again; ADR-0016).
- Option for ripple delete / insert gap to move sequence markers along (they stay in place today).
- Timeline: decode waveform peaks in a worker (a zoom crossing a peak level decodes on the main thread, one ~40 ms frame seen with long files; ADR-0020).
- Timeline: keyboard navigation between clips (Tab/arrows inside lanes, Space to select) for screen-reader users; clips are `option`s without focus today.
- Timeline: user-defined shortcuts (remapping `SHORTCUT_ACTIONS` keys, saved as a machine preference; conflicts already detected).
- Timeline: per-track waveform display in video tracks (clips with sound) and a « show waveforms / thumbnails » view option.
- Timeline: sync the view state (zoom, in/out, track heights) between machines, e.g. an optional `view.json` next to the project (today a per-machine preference; ADR-0020).
- Timeline: per-track sync lock (choose which tracks ripple edits and inserts shift; today the concerned tracks are the edited ones plus linked partners, ADR-0021).
- Timeline: roll, slip and slide edits, and trimming two adjacent edges at once.
- Timeline: auto-scroll while dragging near the lanes' edges, and dragging items to a new track (drops already add one).
- Timeline: snap points of other sequences' items are ignored; snapping to shot changes (scene detection) and to the waveform's transients.
- Media bin: multiple selection to drag several assets to the timeline at once (`placeMedia` already places a list).
- Timeline: copy/paste between projects (the internal clipboard keeps copies for the project they come from).
- Transitions: shrink rather than remove a transition when a plain trim keeps its cut but steals handles (only composite edits and edit plans shorten them today, ADR-0024); freeze-frame handles when the media runs out; per-transition easing; one command to add a picture dissolve and a sound cross-fade on linked cuts from a single cut handle.
- Timeline: volume and pan of audio tracks in headers shorter than 56 px (a popover), keyframe selection with marquee and `Suppr` on the volume line, eased volume segments edited on the line (today linear keyframes from the line; easing through the inspector, prompt 15).
- Timeline: fades of linked picture and sound adjusted together (each clip has its own handles today).

## AI
- Additional AI providers: Gemini CLI, Codex CLI, opencode (one adapter each, see architecture §15).
- API-key mode (needed only if the app is ever distributed to other users).
- B-roll suggestions from the user's media library (local embeddings of thumbnails and transcripts).
- "Style memory": learn the user's editing preferences from accepted/reverted AI changes.
- Agent-to-agent planning: one planner task splitting a long edit into scoped sub-tasks.

## Audio
- Local noise reduction / voice enhancement (e.g. RNNoise or a small ONNX model).
- Loudness normalisation to platform targets (-14 LUFS YouTube, etc.) on export.
- Local text-to-speech for voice-overs.
- Sound at 2×/4× and in reverse with the volume curves applied (scrub snippets already use the clip gain at their instant).

## Interop
- MOGRT generation for editable text in Premiere (research: proprietary, undocumented format).
- FCPXML export (Final Cut Pro, DaVinci Resolve).
- Proprietary RAW support (Blackmagic RAW SDK, RED SDK).
- Master render path through ffmpeg from originals for 10-bit deliverables.

## Projects
- Keep the undo history of a project across closing and reopening (stored in `project.db`; ADR-0017).
- Recover from a corrupt `library.db` (move it aside and start a new one) instead of refusing to start.
- Non-UTF-8 folder paths (Linux): paths travel as UTF-8 strings in the protocol and `library.db`.
- Rename the project folder along with the project (today only the document's name changes).
- Single-instance app (`app.requestSingleInstanceLock`), or several windows each with their project.

## Media
- Bump the Linux ffmpeg build (BtbN monthly build, kept two years) before 2028-10, and the macOS Intel one when Martin Riedl stops publishing them (January 2027); `scripts/binaries.manifest.json`.
- Notarize or re-sign the bundled ffmpeg/ffprobe with the app's identity once macOS builds are signed; publish the GPL source offer with installers that bundle them.
- Show more than the first sound stream in waveforms (pro files with several mono tracks), and keep more than 2 channels when useful.
- Thumbnails of long-GOP footage follow the keyframe interval: add a slower exact mode if scrubbing short screen recordings needs it.
- Linux/macOS: a sidecar can outlive a core killed with `SIGKILL` (its own process group survives); use `PR_SET_PDEATHSIG` on Linux or a watchdog. Windows ties every child to the core with a kill-on-close job (prompt 09).
- A configurable cache location (one folder for every project's derived files, e.g. on a faster drive) and a size limit evicting regenerable files; the settings show the project cache's location and size only (prompt 09).
- Conform variable-frame-rate footage (`MediaInfo` frame rate `None`) to a constant rate through a mezzanine; mezzanines keep the source timing today.
- Keep every sound stream in mezzanines (pro files with several mono tracks); only the first is kept today, like playback.
- AMF (AMD) and Quick Sync (Intel) hardware encoders on Windows; only NVENC is tried there today.
- Raise the priority of a mezzanine when a clip starts using its asset (prompt 11): mezzanines of unused media wait behind analyses.
- A mezzanine decided on a machine that decodes HEVC Main 10 is HEVC: opening the project on a machine that does not decode it needs a « regenerate for this machine » action.
- A project's cache is cleaned by the settings screen only: offer a cleanup of derived files no asset uses any more (removed assets, relinked content).

## Rendering engine
- Run decoding and compositing in a Worker (`OffscreenCanvas` is supported; demuxing runs on the renderer's main thread today) once playback (prompt 13) shows the main thread is busy.
- Recover from a lost WebGL context (recreate the compositor and redraw); the viewer only shows « Aperçu indisponible » when the engine fails to start.
- Pixel aspect ratio: the prober reports none, so anamorphic footage is drawn with square pixels.
- Fall back to the mezzanine at runtime when the original was reported decodable but `canDecode()` refuses it on this machine (today: a « decodeFailed » issue).
- Text in Workers needs the fonts loaded there (FontFace); the preview draws text on the main thread (prompt 15).
- Prefetch the frames around the playhead while scrubbing; prompt 13 decodes sequentially for playback only.
- Playback timers (display refresh, 25 ms audio ticks) are throttled when the window is hidden or minimised: sound stutters there. Options: `backgroundThrottling: false` while playing, or audio scheduling in a Worker (prompt 13).
- Sound at other speeds: JKL 2×/4× and reverse are silent, and clip speed changes pitch (`playbackRate`); add pitch-preserving or granular playback when editing needs it.
- Reverse playback of long-GOP footage decodes a window per chunk and skips frames when the GOP is long; an intra-frame proxy for shuttling would make it smooth.
- Several sound streams per file, and more than stereo output (the master is stereo).
- The first chunk decoded after a seek in a Matroska/WebM file is placed from a millisecond-rounded timestamp (±0.5 ms for the whole run until the next seek): inaudible, but exact only when the previous chunk was decoded first.

## Platform
- Light theme.
- Additional UI languages (English first).
- Plugin system for community motion templates.
