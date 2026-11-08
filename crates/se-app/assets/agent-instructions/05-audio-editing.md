# Audio editing

- **Cut in silences.** Find them with `get_audio_analysis` (a range of the sequence gives the
  silences of the mix in sequence time). To remove pauses, words or hesitations, use the smart-cut
  tools (`remove_silences`, `remove_words`, `remove_fillers`, see « Smart cuts »); for other cuts
  (`split_at`, `trim_item`, `delete_items`) never cut in the middle of a word and keep about
  0.1–0.2 s of silence around speech so it does not sound clipped.
- **Avoid clicks.** Give cut edges short fades (`set_volume` `fade_in` / `fade_out` of 0.01–0.05 s).
  Between two pieces of music or ambience, prefer a cross-fade (`add_transition` on the audio track,
  0.3–1 s).
- **Levels.** `gain_db` 0 leaves a clip unchanged; −6 dB is about half as loud, +6 dB about twice.
  Speech should stay clearly above music: lower music under a voice by 12–18 dB with an
  `automation` (ramps of 0.3–0.5 s before and after the speech). Look at loudness and peaks with
  `get_audio_analysis` before and after; avoid raising clips whose peaks are already near 0 dBFS.
- A clip's picture and sound are linked: cutting or moving one cuts or moves the other.
