# Captions

Captions (subtitles) are made from the transcript with `create_captions`: each caption holds its
words with their times, so animated styles highlight the word being spoken. They go on the caption
track (« Sous-titres », created on top when missing) and stay editable data: never draw captions
with `add_text`.

- **Create, then check.** Call `create_captions` on the range (default: your scope) with a style
  suited to the video: `tiktokBold` or `karaoke` for short vertical social videos, `minimal` for
  interviews and long horizontal videos, `box` on busy pictures. Look at one or two frames with
  `get_frames` afterwards. Media not transcribed yet get no caption: transcribe them first with
  `transcribe_media`.
- **Readability first.** Keep lines short: about 32–42 characters for two-line captions on 16:9,
  15–20 characters on a single line for vertical videos; at most two lines. A caption should stay
  at least 0.7 s and rarely more than 6 s. Change `max_chars_per_line` / `max_lines` on
  `create_captions` (or `update_caption_style`) rather than editing timings by hand.
- **Keep clear of the platforms' interfaces.** The position is the bottom centre of the caption
  block (0–1). Vertical 9:16 videos (TikTok, Reels, Shorts): the bottom third and the right edge
  are covered by buttons and descriptions — keep the bottom at 0.60–0.65 and lines short. 16:9
  videos: 0.85–0.9 stays inside the title-safe area. The user's viewer can show the safe areas.
- **Fix words, not timings.** Correct misheard words, names and punctuation with
  `edit_caption_text`: unchanged and corrected words keep their timing, so the highlight still
  follows the speech. Merge or split captions with the timeline tools only when needed
  (`split_at` between two words; words follow the part holding their middle).
- **Emphasis sparingly.** `emphasize_caption_words` puts a key word forward (one or two per caption
  at most, nouns and numbers that matter), in the style's emphasis colour.
- **Styles are shared.** `update_caption_style` with `item_ids` changes only those captions (a
  shared style is copied for them); with `style_id` it changes every caption of that style, which
  must all lie in your scope. Prefer one consistent style per video.
- **After cuts.** Captions follow smart cuts automatically (the caption track is cut with the
  speech). If you rewrote or moved speech otherwise, run `create_captions` again on the range: it
  replaces the captions it crosses.
- **Summarise**: how many captions, the style, the range, and the words you corrected.
