import type { EffortLevel, SequenceId, TaskId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';
import {
  browseHistory,
  MAX_HISTORY,
  NOT_BROWSING,
  rememberPrompt,
  type HistoryBrowsing,
} from '../services/prompt-history';
import type { RedoTarget, TrackChoice } from '../services/task-scope';

/**
 * The AI composer's state, kept outside React so it survives switching the side panel's tabs: the
 * draft, how the user adjusted the scope, « Refaire avec l'IA », whether to continue the latest
 * task, the model and effort chosen, and the prompt history (model, effort and history are machine
 * preferences saved under `ai.v1`).
 */
export interface ComposerState {
  readonly text: string;
  /** Words of a time range the user removed: not proposed again while they stay in the text. */
  readonly dismissedPhrase: string | null;
  /** The user removed the automatic range: the whole sequence. */
  readonly wholeSequence: boolean;
  /** Tracks the user chose (`'all'`: every track); `null` lets the scope decide. */
  readonly chosenTracks: TrackChoice;
  /** The sequence chosen; `null` follows the timeline's active sequence. */
  readonly sequenceId: SequenceId | null;
  /** « Refaire avec l'IA »: the item to redo. */
  readonly redo: RedoTarget | null;
  /** Send as a follow-up of the redo item's task (its agent keeps its context). */
  readonly resume: boolean;
  /** The recent task the user chose not to continue (the next prompt starts a new task). */
  readonly declinedTaskId: TaskId | null;
  /** `null`: the default of « Réglages → IA ». */
  readonly model: string | null;
  readonly effort: EffortLevel | null;
  readonly history: readonly string[];
  readonly browsing: HistoryBrowsing;
  /** Grows when the composer must take the focus… */
  readonly focusRequest: number;
  /** …and catches up once it did (a composer mounted later still takes it). */
  readonly focusHandled: number;
}

const EMPTY_DRAFT = {
  text: '',
  dismissedPhrase: null,
  wholeSequence: false,
  chosenTracks: null,
  redo: null,
  resume: false,
  declinedTaskId: null,
  browsing: NOT_BROWSING,
} satisfies Partial<ComposerState>;

const INITIAL: ComposerState = {
  ...EMPTY_DRAFT,
  sequenceId: null,
  model: null,
  effort: null,
  history: [],
  focusRequest: 0,
  focusHandled: 0,
};

const STORAGE_KEY = 'ai.v1';
const EFFORTS: readonly EffortLevel[] = ['low', 'medium', 'high', 'xhigh', 'max'];

/** The composer store. */
export const composerStore = createStore<ComposerState>()(() => INITIAL);

/** Changes some fields of the composer. */
export function updateComposer(change: Partial<ComposerState>): void {
  composerStore.setState(change);
}

/** Types in the field (browsing ends by itself once the text differs from the entry shown). */
export function setComposerText(text: string): void {
  composerStore.setState({ text });
}

/** Starts a prompt redoing `target`, resuming its task's session when it has one. */
export function startRedo(target: RedoTarget): void {
  composerStore.setState((state) => ({
    ...EMPTY_DRAFT,
    text: state.text,
    redo: target,
    resume: target.taskId !== null,
    sequenceId: target.sequenceId,
    focusRequest: state.focusRequest + 1,
  }));
}

/** Asks the composer to take the focus. */
export function focusComposer(): void {
  composerStore.setState((state) => ({ focusRequest: state.focusRequest + 1 }));
}

/** Whether a focus request waits; marks it handled. */
export function takeFocusRequest(): boolean {
  const { focusRequest, focusHandled } = composerStore.getState();
  if (focusRequest <= focusHandled) return false;
  composerStore.setState({ focusHandled: focusRequest });
  return true;
}

/** ↑/↓ in the field; whether the key was used. */
export function navigateHistory(direction: 'previous' | 'next'): boolean {
  const state = composerStore.getState();
  const next = browseHistory(state.history, state.browsing, state.text, direction);
  if (next === null) return false;
  composerStore.setState({ text: next.value, browsing: next.browsing });
  return true;
}

/** After a prompt was sent: remembered, and the draft cleared (model, effort, sequence kept). */
export function clearAfterSubmit(prompt: string): void {
  composerStore.setState((state) => ({
    ...EMPTY_DRAFT,
    history: rememberPrompt(state.history, prompt),
  }));
}

/** Back to a fresh composer (tests). */
export function resetComposer(): void {
  composerStore.setState(INITIAL);
}

/** Reads the composer with `selector`. */
export function useComposer<T>(selector: (state: ComposerState) => T): T {
  return useStore(composerStore, selector);
}

/** Saved preferences read back; anything malformed is ignored. */
export function parseComposerPreferences(
  text: string | null,
): Pick<ComposerState, 'model' | 'effort' | 'history'> {
  let saved: Record<string, unknown> = {};
  try {
    const value: unknown = text === null ? null : JSON.parse(text);
    if (typeof value === 'object' && value !== null) saved = value as Record<string, unknown>;
  } catch {
    // Malformed preferences fall back to the defaults.
  }
  const { model, effort, history } = saved;
  return {
    model: typeof model === 'string' ? model : null,
    effort: EFFORTS.find((level) => level === effort) ?? null,
    history: Array.isArray(history)
      ? history.filter((entry): entry is string => typeof entry === 'string').slice(-MAX_HISTORY)
      : [],
  };
}

/** Restores the saved preferences and saves their changes; returns the unsubscribe function. */
export function connectComposerPersistence(storage: KeyValueStorage): () => void {
  composerStore.setState(parseComposerPreferences(storage.getItem(STORAGE_KEY)));
  return composerStore.subscribe((state, previous) => {
    if (
      state.model === previous.model &&
      state.effort === previous.effort &&
      state.history === previous.history
    ) {
      return;
    }
    storage.setItem(
      STORAGE_KEY,
      JSON.stringify({ model: state.model, effort: state.effort, history: state.history }),
    );
  });
}
