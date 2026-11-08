import type { SequenceId } from '@se/protocol';
import { useEffect, type RefObject } from 'react';

import { wordAt, type SequenceWord } from '../../../shared/transcript';
import { playheadStore, playheadTimeOf } from '../../timeline';

/** Attribute marking the word under the playhead (styled by the text component). */
export const CURRENT_WORD_ATTRIBUTE = 'data-current';

/**
 * Marks the word playing at the playhead of `sequenceId` and keeps it in view. It follows the
 * playhead store outside React (a frame's update toggles one attribute, never re-renders the
 * transcript), so it stays cheap while playing long transcripts. Off when `sequenceId` is `null`
 * (words in media time).
 */
export function useWordHighlight(
  container: RefObject<HTMLElement | null>,
  words: readonly SequenceWord[],
  sequenceId: SequenceId | null,
): void {
  useEffect(() => {
    if (sequenceId === null) return undefined;
    let current: Element | null = null;
    let currentIndex = -1;
    const update = () => {
      const index = wordAt(words, playheadTimeOf(playheadStore.getState(), sequenceId));
      if (index === currentIndex && current?.isConnected === true) return;
      current?.removeAttribute(CURRENT_WORD_ATTRIBUTE);
      currentIndex = index;
      current =
        index < 0
          ? null
          : (container.current?.querySelector(`[data-word-index="${String(index)}"]`) ?? null);
      if (current !== null) {
        current.setAttribute(CURRENT_WORD_ATTRIBUTE, '');
        current.scrollIntoView({ block: 'nearest' });
      }
    };
    update();
    const unsubscribe = playheadStore.subscribe(update);
    return () => {
      unsubscribe();
      current?.removeAttribute(CURRENT_WORD_ATTRIBUTE);
    };
  }, [container, words, sequenceId]);
}
