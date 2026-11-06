# 0008 — Premiere Pro export strategy

- Status: Accepted
- Date: 2026-11-06

## Context
Projects must be exportable to Adobe Premiere Pro for finishing. Text and motion design created in SimpleEffect have no native equivalent in Premiere's interchange formats (MOGRT/Essential Graphics is proprietary and undocumented). The user accepted pre-rendered graphics with editable subtitles.

## Decision
Export a **Premiere package folder**:
- `sequence.xml` — **FCP7 XML (xmeml)**, the primary format: sequences, video/audio tracks, clips with original media paths and source in/out, audio levels and fades, opacity/scale/position (Basic Motion), cross dissolves, markers.
- `sequence.otio` — **OpenTimelineIO**, secondary: supported by Premiere 26.0+ (January 2026) and DaVinci Resolve; community reports mention some losses (e.g. clip names).
- `overlays/*.mov` — text, shapes, images and motion templates **pre-rendered as ProRes 4444 with alpha**, placed on upper tracks at the right times.
- `captions.srt` — subtitles, imported by Premiere as **editable captions**.
- `README.txt` — import instructions.

Writers are pure functions from the domain model, snapshot-tested; a manual import checklist is part of the export prompt.

## Consequences
- Reliable round trip of the edit structure; graphics are not editable in Premiere (only captions are).
- Overlay rendering reuses the engine's export path plus ffmpeg ProRes encoding.

## Alternatives considered
- **MOGRT generation** for editable text: undocumented proprietary format, uncertain feasibility. Kept in the backlog as research.
- **AAF**: complex binary format, weaker for graphics. Rejected.
- **EDL**: single track, no audio levels or transforms. Rejected.
