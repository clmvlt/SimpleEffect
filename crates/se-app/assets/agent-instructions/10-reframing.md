# Automatic reframing

When a landscape edit must become portrait (9:16 for Shorts, Reels, TikTok), square or 4:5, or
when clips that fill the frame should keep a speaker in view, use **`auto_reframe`** rather than
setting `reframeFocus` keyframes by hand:

- `auto_reframe {"target_format": "9:16"}` adds a **new sequence** after your task's sequence: a
  version in that format whose clips fill the frame and follow their subject (faces). Texts wider
  than the new frame are narrowed. It is the one sequence you may create; reverting your task
  removes it. Check it with `get_frames` on that sequence (`version_sequence_id`); you cannot edit
  it further in this task (your scope stays on your sequence), so tell the user what to adjust.
- `auto_reframe {}` (or `item_ids`) makes the media clips of your scope follow their subject in
  your sequence's own format (useful when the sequence is already portrait).
- Faces are analysed locally first (once per media, a few seconds per minute of video). If the
  answer says `finished: false`, call it again with the same arguments: nothing changed yet.
- `smoothing` (0–1, default 0.5): lower follows lively moves closely, higher keeps the frame
  steady (interviews, talking heads). Leave the default unless the user asks.
- The window jumps to the next shot's subject exactly on each shot change; media without faces
  stay centred. Afterwards the focus is ordinary `reframeFocus` keyframes: adjust one with
  `set_keyframes` if a moment is badly framed (look at it with `get_frames` first).
- `VISION_UNAVAILABLE`: faces cannot be detected on this machine; say so and offer to frame the
  clips by hand (`update_item` with `reframe: "fill"`, then `set_keyframes` on `reframeFocus`).
