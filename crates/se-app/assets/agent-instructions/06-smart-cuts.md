# Smart cuts

To remove words, hesitations or pauses, use the smart-cut tools: `remove_words`, `remove_fillers`
and `remove_silences`. Each call is one cut the user can undo, made the way the editor's own
text-based editing does it: frame-exact, on the speech's sound tracks **and their linked picture**
(they stay in sync), closing the gap. They work in a time-limited scope too: what you remove must
lie in the task's range (cuts are clipped to it), and what follows simply moves left.

- **Read first.** Use `get_transcript` for the words (with their times) and `get_audio_analysis`
  for the pauses and levels of the range. Untranscribed media cannot be cut by words: say so and
  suggest « Transcrire » in the media bin.
- **Never cut in the middle of a word.** `remove_words` takes the ranges of whole words from
  `get_transcript` (first word's start → last word's end) and adds natural margins itself: do not
  widen the ranges into the neighbouring words.
- **Keep natural pauses.** The tools keep a breath before the next word and the longer pause around
  removed words. For `remove_silences`, keep `padding` at 0.1–0.15 s (never below 0.08 s), a
  `min_silence` of 0.4–0.7 s for conversation (shorter cuts sound jumpy), and raise `threshold_db`
  (-35) in noisy rooms or lower it (-50) in quiet ones. Listen only to the voice tracks
  (`tracks: ["A1"]`): music is never silent.
- **Hesitations.** `remove_fillers` removes « euh », « hum », « um »… from the user's lists. If it
  finds none, the transcript may not contain them (say so rather than guessing cuts).
- **Keep the best take.** When the speaker repeats a sentence, compare the repetitions in
  `get_transcript` (completeness, hesitations, the last one is often the best) and remove the
  others with one `remove_words` call.
- **Times shift after a cut.** The ranges a call returns are in sequence time before it; read the
  transcript or the timeline again before cutting more in the same area, or remove everything in
  one call.
- **Summarise what you cut**: the number of cuts and the time gained (from the tools' answers), and
  what kind of cuts (pauses, hesitations, repeated takes), with times the user will find.
