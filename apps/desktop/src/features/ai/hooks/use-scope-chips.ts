import type { PromptScope } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { useActiveSequence } from '../../timeline';
import { itemLabel } from '../services/item-label';
import { shortRange } from '../services/short-time';
import { updateComposer } from '../store/composer-store';
import type { ComposerScope } from './use-composer-scope';

/** The scope chips of the composer and what removing one does. */
export interface ScopeChips {
  readonly chips: readonly PromptScope[];
  readonly remove: (id: string) => void;
}

/**
 * Chips saying the prompt's scope: its range and where it comes from (dashed when read in the
 * text), its tracks, and its sequence when it is not the one shown. Removing the range chip goes
 * back to the automatic range (from the text: the text's range is dismissed; from the item: the
 * redo is dropped; automatic: the whole sequence); removing chosen tracks goes back to automatic.
 */
export function useScopeChips(scope: ComposerScope | null): ScopeChips {
  const { t } = useTranslation();
  const active = useActiveSequence();
  const assets = useProjectSelector((state) => state.mirror?.assets);
  if (scope === null) return { chips: [], remove: () => undefined };
  const { derived, sequence, tracks, phrase, redoItem } = scope;
  const range = derived.scope.range === null ? '' : shortRange(derived.scope.range);
  const chips: PromptScope[] = [];
  switch (derived.rangeSource) {
    case 'text':
      chips.push({
        id: 'range',
        label: range,
        detected: true,
        description: t('ai.composer.scope.textHint', { words: phrase?.text ?? '' }),
      });
      break;
    case 'item': {
      const label = redoItem === null ? null : itemLabel(redoItem, assets ?? {});
      const name = label?.label ?? t(`editor.timeline.items.${label?.fallback ?? 'media'}`);
      chips.push({ id: 'range', label: t('ai.composer.scope.item', { name, range }) });
      break;
    }
    case 'selection':
      chips.push({ id: 'range', label: t('ai.composer.scope.selection', { range }) });
      break;
    case 'inOut':
      chips.push({ id: 'range', label: t('ai.composer.scope.inOut', { range }) });
      break;
    case 'sequence':
      chips.push({ id: 'range', label: t('ai.composer.scope.wholeSequence'), removable: false });
      break;
  }
  const scopeTracks = derived.scope.tracks;
  if (scopeTracks !== null) {
    const names = tracks
      .filter((track) => scopeTracks.includes(track.id))
      .map((track) => track.name);
    chips.push({
      id: 'tracks',
      label: t('ai.composer.scope.tracks', { names: names.join(', ') }),
      removable: derived.tracksSource === 'chosen',
    });
  }
  if (active !== null && sequence.id !== active.id) {
    chips.push({ id: 'sequence', label: t('ai.composer.scope.sequence', { name: sequence.name }) });
  }
  const remove = (id: string) => {
    if (id === 'tracks') updateComposer({ chosenTracks: null });
    if (id === 'sequence') updateComposer({ sequenceId: null, chosenTracks: null });
    if (id !== 'range') return;
    if (derived.rangeSource === 'text') updateComposer({ dismissedPhrase: phrase?.text ?? null });
    else if (derived.rangeSource === 'item') updateComposer({ redo: null, resume: false });
    else updateComposer({ wholeSequence: true });
  };
  return { chips, remove };
}
