import type { CaptionStyle, Flicks, SequenceEntity } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useProjectSelector } from '../../../shared/project-store';
import { playheadStore, playheadTimeOf } from '../../timeline';
import { activeRowIndex, captionRows, contentEnd, type CaptionRow } from '../services/caption-rows';

const NO_STYLES: readonly CaptionStyle[] = [];

/** The captions of `sequence` and the project's caption styles, from the mirror. */
export function useCaptionRows(sequence: SequenceEntity | null): {
  readonly rows: readonly CaptionRow[];
  readonly styles: readonly CaptionStyle[];
  /** Where the sequence's content ends (for the in/out span). */
  readonly end: Flicks;
} {
  const tracks = useProjectSelector((state) => state.mirror?.tracks);
  const items = useProjectSelector((state) => state.mirror?.items);
  const styles = useProjectSelector((state) => state.mirror?.project.captionStyles) ?? NO_STYLES;
  if (sequence === null || tracks === undefined || items === undefined) {
    return { rows: [], styles, end: 0 as Flicks };
  }
  return {
    rows: captionRows(sequence, tracks, items),
    styles,
    end: contentEnd(sequence, tracks, items),
  };
}

/**
 * Index of the caption under `sequence`'s playhead, updated only when it changes (the playhead
 * moves at every frame while playing).
 */
export function useActiveRow(sequenceId: string | null, rows: readonly CaptionRow[]): number {
  const [active, setActive] = useState(-1);
  useEffect(() => {
    if (sequenceId === null) return undefined;
    const update = () => {
      const time = playheadTimeOf(playheadStore.getState(), sequenceId);
      setActive(activeRowIndex(rows, time));
    };
    update();
    return playheadStore.subscribe(update);
  }, [sequenceId, rows]);
  return active;
}
