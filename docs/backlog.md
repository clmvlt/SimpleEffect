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

## Platform
- Light theme.
- Additional UI languages (English first).
- Plugin system for community motion templates.
