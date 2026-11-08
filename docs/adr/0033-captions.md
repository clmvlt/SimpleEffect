# 0033 — Captions: timed words, shared styles with copy on write, rendered by the engine

- Status: Accepted
- Date: 2026-11-08

## Context
Prompt 23 turns the transcript into animated subtitles in the style of short social videos (the
word being spoken highlighted, words popping) that stay editable (text, timing, style), are
exported as SRT (and later to Premiere) and can be made and styled by agents. Until then a
caption was a fixed-look text (`CaptionItem { text }`, drawn by the engine with a hard-coded
style). Questions:
- how word timing survives edits of the text, trims, splits and the smart cuts of prompt 22;
- where styles live, how one caption is restyled without restyling the others, and how agents are
  kept inside their scope when a style is shared;
- where the predefined styles live, since the renderer (panel, previews), the core (generation,
  agents) and the engine all need them;
- how the engine draws per-word animation with the same code in preview and export.

## Decision
- **A caption is timed words and a style reference**: `CaptionItem { words: [CaptionWord { text,
  start, end, emphasis }], styleId }`, word times relative to the item like keyframes (validation
  `INVALID_CAPTION_WORDS`: non-empty, apart, in order, inside the item). Document schema 8; a
  version-7 caption's text becomes words spread equally with a `minimal` style.
- **Words follow every edit of their item by their middle**: trims, splits, ripple deletes and
  gaps reshape an item; a word belongs to the part holding its middle and is clamped into it
  (`reshape::retime_words`). No caption-specific operation is needed for timing: `MoveItem`,
  `TrimItem`, `SplitItem` and the multi-range ripple delete work on captions as on any item.
- **Text edits keep the timing of unchanged words** (`se_domain::captions::retimed_words`): the new
  text is compared word by word with the old (longest common subsequence); unchanged words keep
  their times and emphasis, a changed run with as many new words keeps the old times one for one
  (a typo fix never moves the highlight), other runs share their time (or the pause around an
  insertion) in proportion to characters plus one; a run without time takes in its neighbours.
  This refines the prompt's « proportional redistribution » so a correction does not move words
  it did not touch.
- **Styles are project entities shared by captions** (`Project.captionStyles`, operations
  `AddCaptionStyle`, `UpdateCaptionStyle`, `RemoveCaptionStyle` — refused while used). Changing the
  look of some captions is **copy on write** (`se_app::captions::styles::patch_styles`): a style
  whose users are all targeted changes in place; a style also used by other captions is copied for
  the targeted ones; styles left without users by an edit are removed in the same transaction.
  The interface picks the targets (« Appliquer à : sélection / toute la piste »); agents give
  `item_ids` (copy) or `style_id` (in place).
- **Scope rule for styles**: an AI transaction may touch a caption style only if every caption
  using it, before and after, lies in the task's scope (`scope.rs`), since a style change changes
  their look.
- **Predefined styles are data files of the engine** (`packages/engine/src/caption-styles/*.json`:
  `tiktokBold`, `minimal`, `karaoke`, `box`), embedded by se-domain (`caption_preset`) so the core
  creates the same styles for generation and agents; both sides check them in tests. A style keeps
  the id of its preset to be named in the interface (no user-facing string in the document). In a
  portrait sequence a style made from a preset is raised above the bottom third (platform
  interfaces).
- **Generation is pure segmentation on the caption track** (`se_domain::captions::segment_words`):
  words whose middle lies in the range are grouped (pause ≥ 0.6 s, lines full — cut after a clause
  end when possible —, longer than the maximum duration, sentence ends, clause ends once half the
  room is used; punctuation never starts a caption), captions last 0.7–6 s, short gaps are closed,
  edges snap to frames. The **caption track** is the highest unlocked video track holding captions
  only, created on top (« Sous-titres ») when missing; captions of that track crossing the range
  are replaced in the same transaction (the range grows over them).
- **Captions follow smart cuts**: caption tracks with captions at or after the first cut are added
  to the cut tracks (only when the task may edit them, for agents); a caption crossing a cut is cut
  like any item, its words by their middle.
- **The engine draws a `caption` layer word by word** (`evaluate/caption-layer.ts` +
  `composite/pixi/caption-node.ts`): lines broken greedily by characters (shared vectors with the
  core), the page of lines holding the active word (the last word started, active until the next
  starts), each word a Pixi text placed from a layout measured on the plain words (colours, pops,
  reveals and outlines never move a word), plates behind the block or each line. Animations:
  `none`, `wordHighlight` (+ `highlightSpoken` for karaoke), `popPerWord`, `boxPerLine`,
  `revealPerWord`; emphasised words use the style's emphasis colour. Preview and export share it.
- **SRT** is a pure writer and a tolerant reader in se-export behind se-app's `SubtitleFiles` port
  (UTF-8 with BOM, CRLF; BOM/LF/CR, dot milliseconds, markup and Latin-1 accepted on import).
  Imported cues become captions with words spread equally.

## Consequences
- Captions are ordinary items: undo, timeline editing, AI reverts and the project format work
  without special cases; the cost is that a caption crossing a cut becomes two captions (the user
  merges them back if wanted).
- A style change from the interface may add a style (copy) or remove unused ones; the number of
  styles stays proportional to the looks actually used.
- The preset files sit in the engine package but are part of the core binary: changing one changes
  new styles only (documents keep their copies).
- Line breaking by characters ignores glyph widths: a very wide font can overflow the frame; the
  style's character limit is the user's lever (backlog: measured breaking).
