import type { PreviewStats } from '@se/engine/preview';
import type { Flicks, SequenceId } from '@se/protocol';

/**
 * What the viewer lets the performance measure (prompt 29, `docs/performance.md`) drive: the
 * real scrub path (playhead store → engine → drawn frame) and the engine's counters. The viewer
 * registers one while it shows a sequence.
 */
export interface ViewerProbe {
  readonly sequenceId: SequenceId;
  /** End of the sequence's content: measured times stay before it. */
  readonly contentEnd: Flicks;
  /** The playhead of the shown sequence. */
  time(): Flicks;
  /** Moves the playhead to `time`; resolves with the milliseconds until its frame was drawn. */
  seek(time: Flicks): Promise<number>;
  /** Decoder and playback counters (frames drawn and dropped, open `VideoFrame`s). */
  stats(): PreviewStats | null;
}

let current: ViewerProbe | null = null;

/** Makes `probe` the measured viewer; returns the function removing it. */
export function registerViewerProbe(probe: ViewerProbe): () => void {
  current = probe;
  return () => {
    if (current === probe) current = null;
  };
}

/** The viewer showing a sequence now, or `null` (no project open, no WebCodecs). */
export function currentViewerProbe(): ViewerProbe | null {
  return current;
}
