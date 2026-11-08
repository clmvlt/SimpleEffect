# 0032 — Smart cuts: one planning for text-based editing, fillers and silences, users and agents

- Status: Accepted
- Date: 2026-11-08

## Context
Prompt 22 cuts video by editing its transcript (removing words removes the passage), removes
filler words and silences in one action, and gives agents the same abilities. Every cut must stay
ordinary, adjustable editing (no baked result), one user action or tool call must be one
transaction, and no cut may break the sync of linked picture and sound. Several things the
existing code did not provide:
- removing hundreds of ranges (every pause of an hour-long interview) as single
  `RippleDeleteRange`s records, for each range, a snapshot of every item that moves after it:
  tens of megabytes of inverse in the journal and in the undo history;
- the right parts of items split by a ripple delete had no link group, so a cut in the middle of
  a linked clip left picture and sound unlinked;
- silences were stored only for one setting (−40 dBFS, 0.5 s), while the user picks the threshold;
- the strict scope rule of agents (ADR-0029) refuses any ripple edit that moves content outside a
  time-limited scope, and items crossing its edges: « remove the pauses between 0 and 2 min » of a
  10-minute interview would always be refused.

## Decision
- **One planning for everyone** (`se_app::smart_cuts`): a `CutRequest` (words of ranges, fillers
  of a range, silences of a range; all tracks or not; a limit) is **resolved on the current
  document** — words through `se_domain::sequence_transcript`, fillers among them, silences from
  the assets' level envelopes through each clip's window — into candidate ranges; `plan_cuts`
  pads them (silences), keeps them inside the clips and the limit, merges those closer than 80 ms,
  extends a cut to a clip edge closer than 80 ms, snaps to frames (silences inwards, words to the
  nearest frame) and drops cuts under a frame. The user's commands (`removeWords`,
  `removeSilences`, previews) and the agents' tools (`remove_words`, `remove_fillers`,
  `remove_silences`) run the same code; only the commit differs (user transaction vs
  `apply_ai_on` with a changeset).
- **Word margins**: removing a run of words keeps the longer of the two pauses around it and cuts
  the shorter one down to a 60 ms breath before the next kept word; a removed word is cut 30 ms
  before its start (DTW word starts are within ±30 ms, ends are late, ADR-0031); margins never take
  more than half a pause; clip edges bound everything.
- **A new domain operation `RippleDeleteRanges`** removes several sorted, apart ranges in one pass:
  each item is reshaped once (first part keeps its id, one new part per range strictly inside it),
  later items move left by the time removed before them, transitions are restored as in
  `RippleDeleteRange`. Its inverse holds one snapshot per changed item. It equals the single
  ripple deletes applied from the last range (property test). `remaining_parts` (public) tells
  callers how many ids an item needs.
- **Linked parts are relinked**: the planner links the parts of the items of one link group lying
  between the same two cuts (the earliest ones keep the group). Groups without a split part are left
  alone.
- **Cut tracks**: the speech's tracks (the words' clips, or the listened sound tracks) plus, until
  stable, every unlocked track holding an item linked to an item crossing or following the cut
  (picture follows sound); `all_tracks` cuts every unlocked track (graphics and music included).
  Graphics and unlinked music are not cut by default.
- **Silences for any threshold**: the audio analysis stores a `LevelEnvelope` (RMS of 20 ms
  windows, whole dB below full scale, ~180 kB per hour as JSON numbers) next to its silences.
  Silences are measured over the whole sequence (every listened track quiet; time where no
  listened track has a clip is never a silence; unanalysed media count as sound), then clipped to
  the requested range, so a silence crossing the range keeps its full length for the minimum.
  Analyses made before this are `is_outdated` and run again when their project opens.
- **The « cut » scope rule for agents**: for the three smart-cut tools only, a cut is accepted when
  the **time it removes** lies in the task's range (resolution clips cuts to it) and **every track
  it edits** is one of the task's tracks. What follows may move; a clip crossing the scope's edge is
  cut inside it only. `ripple_delete_range` and `insert_gap` keep the strict rule. The smart-cut
  tools are not batchable (each resolves on the current document).
- **Transcript side** (renderer): selected words are cut with Suppr (new shortcut context
  `transcript`); cut passages are found between touching clips of one asset with a forward source
  jump and shown struck through (a silence as « ⋯ 1,2 s »); « Restaurer » ripple-trims the end of
  the clip before the cut (and its linked partners) by the passage. Silence removal previews its
  cuts as temporary bands on the timeline (`showCutPreview`, never project data).
- **Hesitations**: whisper tends to clean « euh » away; a preference (« Conserver les hésitations »,
  on by default) opens its prompt with a few hesitations in the transcript's language.

## Consequences
- Every cut is one undo step of ordinary items: a pause left can be lengthened with a ripple trim;
  agents' cuts revert with their task.
- The journal stays small for many cuts; previews cost one resolution on the core (no IO beyond the
  stored analyses and transcripts).
- Old projects re-run their audio analyses once (seconds per hour of media).
- Restoring a passage needs its two clips still touching; a cut at a clip's very start or end leaves
  no neighbour, so it is not shown (undo still restores it).
- Filler detection depends on the transcript containing the fillers.

## Alternatives considered
- **Sequential `RippleDeleteRange`s**: no new operation, but inverses grow with ranges × items
  (≈ 50 MB for 300 cuts on an hour-long interview).
- **Re-running ffmpeg silence detection for each preview**: nothing more stored, but seconds per
  preview on long media; the slider would not follow.
- **Keeping the strict scope rule for cuts**: agents would need a whole-sequence scope to cut
  anything in a long recording, defeating time-limited tasks.
- **Detecting fillers and silences in the renderer**: two implementations to keep in step with the
  agents' tools; the core already holds analyses and transcripts.
