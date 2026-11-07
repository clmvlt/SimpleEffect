import type { Flicks, TransitionAlignment, TransitionId } from '@se/protocol';

import { useShortcutAction } from '../../../shared/shortcuts';
import { FLICKS_PER_SECOND } from '../../../shared/time';
import { transitionsAtPlayhead } from '../services/audio-edits';
import { editPreferencesStore } from '../store/edit-preferences-store';
import { clearSelection, selectionStore } from '../store/selection-store';
import type { TimelineCommands } from './use-timeline-commands';
import type { TimelineModel } from './use-timeline-model';

/** The transition edits shared by the shortcut, the context menu and the inspector. */
export interface TransitionActions {
  /** Adds a cross-fade on the cut nearest the playhead of each track (`Ctrl/⌘+T`). */
  readonly addAtPlayhead: () => void;
  readonly realign: (transitionId: TransitionId, alignment: TransitionAlignment) => void;
  readonly remove: (transitionId: TransitionId) => void;
}

/** The default length of new transitions, in flicks. */
export function defaultTransitionDuration(): Flicks {
  return Math.round(
    editPreferencesStore.getState().transitionSeconds * FLICKS_PER_SECOND,
  ) as Flicks;
}

/** The transition actions on the active sequence of `model`; binds `Ctrl/⌘+T`. */
export function useTransitionActions(
  model: TimelineModel,
  commands: TimelineCommands,
  playheadTime: () => Flicks,
): TransitionActions {
  const addAtPlayhead = () => {
    const selected = new Set(selectionStore.getState().itemIds);
    const selectedTracks = new Set(
      model.items.filter((item) => selected.has(item.id)).map((item) => item.trackId),
    );
    const wanted = transitionsAtPlayhead(
      model.tracks,
      model.itemsByTrack,
      playheadTime(),
      defaultTransitionDuration(),
      selectedTracks,
    );
    if (wanted.length > 0) void commands.addTransitions(wanted);
  };
  const realign = (transitionId: TransitionId, alignment: TransitionAlignment) => {
    void commands.updateTransition(transitionId, { alignment });
  };
  const remove = (transitionId: TransitionId) => {
    void commands.removeTransitions([transitionId]).then((done) => {
      if (done !== null && selectionStore.getState().transitionId === transitionId) {
        clearSelection();
      }
    });
  };
  useShortcutAction('addTransition', addAtPlayhead);
  return { addAtPlayhead, realign, remove };
}
