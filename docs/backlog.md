# Backlog — ideas not yet scheduled

Ideas worth keeping that are not part of the current roadmap (`prompts/`). When one is scheduled, write a prompt from `prompts/_TEMPLATE.md` and remove it from here.

## Editing
- Beat detection on music tracks (local onset detection) to snap cuts and animations to the beat.
- Project snapshots / named versions with visual diff between versions.
- Draft/review mode for AI changes (evaluate "document + pending changeset", accept/reject) — the storage model already allows it (ADR-0005).
- Multicam editing (sync by audio waveform).
- Speed ramps with keyframed speed curves.
- Color correction basics (exposure, contrast, saturation, LUT import).
- Transitions library beyond cross dissolve (wipes, zooms, glitch) as motion templates.

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

## Platform
- Light theme.
- Additional UI languages (English first).
- Plugin system for community motion templates.
