# 0026 — Reframing, format changes, sequence versions and safe areas

- Status: Accepted
- Date: 2026-11-07

## Context
Prompt 16 lets an edit move between formats (16:9, 9:16, 1:1, 4:5…): change a sequence's format, keep several sequences per project (a 16:9 edit and its 9:16 version), choose per clip how its picture fits the frame and which part of it stays visible over time, and show where platforms (TikTok, Instagram Reels, YouTube Shorts) cover the picture with their interface. ADR-0015 already made positions normalised and sizes design pixels, so texts and graphics keep their relative places across formats; what remained open: what a clip's reframe mode stores (and how a "kept area" survives a change of aspect), what "custom" means, where format changes and copies are computed, how the frame rate behaves, how the user learns that a change pushed something out of the frame, and where safe-area data and choices live.

## Decision
- **Reframe modes.** `Reframe = fit | fill { focus } | custom | tracked { focus }` on media clips (schema version 6; documents and journals without a focus read it as the centre).
  - `fit`: the whole picture inside the frame (letterbox).
  - `fill` / `tracked`: the picture covers the frame and is **cropped** to the frame's aspect around a **focus**: a point of the upright picture, normalised in it, kept at the frame's centre as far as the picture allows (the crop never leaves it). The layer's box is the frame; the transform then applies to the cropped picture, so a zoom scales around the visible centre.
  - `custom`: the picture at its natural size, one picture pixel per design pixel (like images' `natural`), framed only by the transform; it does not adapt to the frame.
  - `tracked` is evaluated like `fill`; prompt 26 writes its keyframes (subject tracking).
- **The focus is an animatable property** (`AnimatableProperty::ReframeFocus`, each component in `[0, 1]`): keyframes, auto-keyframing, keyframe lanes, trims and splits all work through the existing machinery (ADR-0025). Being a point *of the picture*, independent of the frame's aspect, a focus (and prompt 26's trajectories) survives format changes and versions.
- **One evaluation.** `@se/engine/evaluate` (`reframedPicture`) computes each video layer's box and crop from the domain's data; the Pixi compositor frames its texture on the crop, mapped back to the decoded pixels through display rotation and mirroring (`codedCrop`). The renderer's size-keeping mode switches use the engine's `reframeScale`; nothing else computes boxes.
- **Format changes and versions are domain functions.** `se_domain::reformat`: `FormatAdaptation { clipReframe: keep | fit | fill, fitTextWidths }`; `format_change` returns an `UpdateSequence` plus one `UpdateItem` per item that changes (items on locked tracks stay) — their inverses undo the whole change exactly; `sequence_version` copies a sequence with fresh ids (`copy_sequence` over a `SequenceIds` the caller implements: link groups and transitions remapped, revisions reset) into another format and adapts it (locked tracks included: the copy is new content). `fill` keeps the focus of clips already filling (or tracked). `fitTextWidths` gives texts wider than 90 % of the new frame (wrapping width × horizontal scale; no wrapping counts as too wide) that wrapping width, so long lines wrap.
- **The frame rate is fixed once a sequence holds items** (items sit on frames of its rate): `format_change` and `sequence_version` refuse (`FRAME_RATE_LOCKED`); the dialog disables the field and says why. The rule is in these planners, not in `UpdateSequence`, so undoing a change always works.
- **Use cases and protocol.** `renameSequence`, `setSequenceFormat`, `duplicateSequence` (same format, or a version in `format` — labelled « Create version ») and `removeSequence` (refused for the project's only sequence: `LAST_SEQUENCE`); one undo step each. `updateItems` takes optional `animations` set in the same step, so a mode switch and its compensating scale are one undo step.
- **Out-of-frame warning, after the fact.** After a format change or a version, the renderer compares each item before and after (same id, or same position in the copy) at its first, middle and last instants with `evaluate` and a canvas text measurer, and reports items **newly** leaving the frame (already-outside items, entrances, filling clips are not reported) in a toast whose « Sélectionner » selects them.
- **Safe areas are engine data, guides are viewer preferences.** `packages/engine/src/safe-areas/*.json` (one file per platform with source, URL and measurement date; normalised at load) feeds `@se/engine/safe-areas`. The viewer draws them (covered zones shaded, title/action safe outlined, plus a thirds grid and a centre cross) as a DOM overlay: never part of a frame or an export. The choice is remembered **per sequence** in the viewer's machine preferences (`se.ui.viewer.v1`), not in the project.
- **Reframing in the viewer.** Double-clicking a filling clip (or the inspector's « Recadrer dans le viewer ») replaces the transform gizmo by a reframing overlay: the whole picture's outline and its thumbnail (nearest tile of the asset's sheet, dimmed) around the frame; dragging moves the picture under the frame (preview, then one `setItemAnimations`); Enter, Escape or « Terminer » leave.

## Consequences
- A 16:9 → 9:16 conversion keeps texts at the same relative places, narrows wide texts, fills the frame with every clip and lets the user move each clip's kept area by keyframes — all editable data (golden rule 6), the same through the UI and (prompt 19) through MCP tools.
- Prompt 26 only has to write `tracked` focus keyframes; nothing in evaluation, editing or the UI changes.
- Platform interfaces change: the safe-area files carry their date and source and must be re-checked from time to time; the figures are third-party measurements (no platform publishes official ones for organic posts).
- `custom` clips switched from another mode get a compensating scale (renderer), so the picture does not jump.
- Trims and splits now retime every animatable property (they missed blur and text styles before).

## Alternatives considered
- **Fill as an overflowing box clipped by the frame** (the prompt 12 behaviour): a focus would need an anchor shift that the gizmo would then write back as the user's anchor, and a zoom would pivot around the picture's centre rather than the visible one. Rejected for the crop.
- **A crop window position (0 = left edge, 1 = right edge)**: depends on the frame's aspect, so keyframes would be wrong after a format change. Rejected for a focus point of the picture.
- **`custom` = the fit box with a free transform**: indistinguishable from `fit` once rendered. Rejected for the natural size (the user's choice).
- **Intent-level operations (`SetSequenceFormat`) in the domain**: the existing operations already invert exactly; a planner keeps the journal format unchanged.
- **Safe-area choice stored in the project**: it is a view setting (not rendered, not exported), like the timeline's view state.
- **Warning before applying (preview in the dialog)**: would need the adaptation rules duplicated in the renderer; comparing the real states after the change needs none.
