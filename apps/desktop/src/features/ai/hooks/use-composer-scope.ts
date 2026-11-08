import type { Flicks, ItemEntity, SequenceEntity, TrackEntity } from '@se/protocol';

import { useProjectSelector } from '../../../shared/project-store';
import { useActiveSequence, useInOutPoints, useSelectedItemIds } from '../../timeline';
import { sequenceContentEnd } from '../services/sequence-content';
import { deriveScope, type DerivedScope } from '../services/task-scope';
import { findTimePhrase, type TimePhrase } from '../services/time-phrases';
import { useComposer } from '../store/composer-store';

/** What the composer shows about its scope. */
export interface ComposerScope {
  /** The sequence the prompt works on (chosen, else the timeline's active one). */
  readonly sequence: SequenceEntity;
  /** Its tracks, in project order. */
  readonly tracks: readonly TrackEntity[];
  /** End of its last item. */
  readonly sequenceEnd: Flicks;
  readonly derived: DerivedScope;
  /** The range read in the text, when it is the scope's. */
  readonly phrase: TimePhrase | null;
  /** The item « Refaire avec l'IA » names, when it still exists. */
  readonly redoItem: ItemEntity | null;
}

/**
 * The scope of the prompt being written, derived from the composer's state and what the editor
 * shows (selection and in/out points of the sequence, the text); `null` without a sequence.
 */
export function useComposerScope(): ComposerScope | null {
  const active = useActiveSequence();
  const chosenId = useComposer((state) => state.sequenceId);
  const sequences = useProjectSelector((state) => state.mirror?.sequences);
  const allTracks = useProjectSelector((state) => state.mirror?.tracks);
  const items = useProjectSelector((state) => state.mirror?.items);
  const selectedIds = useSelectedItemIds();
  const text = useComposer((state) => state.text);
  const dismissed = useComposer((state) => state.dismissedPhrase);
  const wholeSequence = useComposer((state) => state.wholeSequence);
  const chosenTracks = useComposer((state) => state.chosenTracks);
  const redo = useComposer((state) => state.redo);
  const chosen = chosenId === null ? undefined : sequences?.[chosenId];
  const sequence = chosen ?? active;
  const points = useInOutPoints(sequence?.id ?? '');
  if (sequence === null || allTracks === undefined || items === undefined) return null;
  const tracks = sequence.trackIds.flatMap((id) => allTracks[id] ?? []);
  const end = sequenceContentEnd(tracks, items);
  const inOut = points;
  const span =
    inOut.inPoint === null && inOut.outPoint === null
      ? null
      : { start: inOut.inPoint ?? (0 as Flicks), end: inOut.outPoint ?? end };
  const found = findTimePhrase(text, { frameRate: sequence.format.frameRate, sequenceEnd: end });
  const phrase = found !== null && found.text !== dismissed ? found : null;
  const onSequence = (item: ItemEntity) => sequence.trackIds.includes(item.trackId);
  const selection = selectedIds.flatMap((id) => {
    const item = items[id];
    return item !== undefined && onSequence(item) ? [item] : [];
  });
  const redoItem = redo === null ? null : (items[redo.itemId] ?? null);
  const derived = deriveScope({
    sequenceId: sequence.id,
    inOut: span !== null && span.end > span.start ? span : null,
    selection,
    phrase,
    redo: redo !== null && redo.sequenceId === sequence.id ? redo : null,
    wholeSequence,
    chosenTracks,
  });
  return { sequence, tracks, sequenceEnd: end, derived, phrase, redoItem };
}
