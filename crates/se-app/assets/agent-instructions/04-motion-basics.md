# Motion basics

**Units.** Sizes are design pixels: 1080 is the frame's shorter side, whatever the format (in 9:16
the width is 1080, in 16:9 the height). Positions are normalised: `x` 0 (left) to 1 (right), `y` 0
(top) to 1 (bottom); `(0.5, 0.5)` is the centre and an item's position is where its centre sits.

**Presets, keyframes or a template.** For usual entrances and exits use presets (`list_animation_presets`):
`animation_in` / `animation_out` of `add_text`, `add_shape`, `add_image`, or
`apply_animation_preset`. Use `set_keyframes` for a custom move (a title sliding to a corner, a
slow zoom). Easing: `ease_out` for entrances, `ease_in` for exits, `ease_in_out` for moves; avoid
`linear` for motion.

**Durations.**
- Entrances and exits: 0.2–0.6 s (a pop 0.3–0.4 s); never longer than a third of the item.
- Text must stay readable: at least 1 s plus about 0.3 s per word on screen; a short title 1.5–4 s.
- Keep a rhythm: similar items get the same animation and length.

**Typography for short videos.**
- Big and bold for key words: `font_size` 100–160 (9:16) or 80–120 (16:9), `font_weight` 700–900;
  captions and secondary text 48–72.
- Few words per line (at most about six); set `max_width` (about 900 in 9:16) for longer text.
- Legibility over video: an outline (`stroke` 4–8 px, dark), a `shadow`, or a `background` box;
  high contrast colours; avoid thin weights.

**Placement and safe areas.** Prefer the named positions (`center`, `top`, `bottom`,
`lower_third`). In vertical videos keep key text out of the top 12 %, the bottom 20 % and the
right edge (the apps' buttons and captions cover them); in 16:9 keep a 5 % margin. Check with
`get_frames`.

**Templates.** For procedural motion or several elements moving together (an animated lower
third, words rising one by one, a progress bar, a call-to-action button), write a motion template
(next section): the user keeps editing its texts, colours and options in the inspector.
