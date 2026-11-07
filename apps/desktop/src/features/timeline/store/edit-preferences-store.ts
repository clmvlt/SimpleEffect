import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';

/** Editing preferences of the timeline: machine preferences, never project data. */
export interface EditPreferences {
  /** Length of new transitions (`Ctrl/⌘+T`), in seconds. */
  readonly transitionSeconds: number;
  /** Show the silences found by the audio analysis on audio clips (hatched). */
  readonly showSilences: boolean;
}

/** Shortest and longest default transition, in seconds. */
export const TRANSITION_SECONDS_RANGE = { min: 0.1, max: 10 } as const;

/** Defaults on a new machine. */
export const DEFAULT_EDIT_PREFERENCES: EditPreferences = {
  transitionSeconds: 1,
  showSilences: true,
};

const STORAGE_KEY = 'timeline-edit.v1';

/** The editing preferences store. */
export const editPreferencesStore = createStore<EditPreferences>()(() => DEFAULT_EDIT_PREFERENCES);

/** Changes some editing preferences (the transition length is kept in range). */
export function setEditPreferences(change: Partial<EditPreferences>): void {
  const seconds = change.transitionSeconds;
  editPreferencesStore.setState(
    seconds === undefined
      ? change
      : {
          ...change,
          transitionSeconds: Math.min(
            TRANSITION_SECONDS_RANGE.max,
            Math.max(TRANSITION_SECONDS_RANGE.min, seconds),
          ),
        },
  );
}

/** The editing preferences. */
export function useEditPreferences(): EditPreferences {
  return useStore(editPreferencesStore);
}

/** Saved preferences read back; anything malformed falls back to the defaults. */
export function parseEditPreferences(text: string | null): EditPreferences {
  let saved: Record<string, unknown> = {};
  try {
    const value: unknown = text === null ? null : JSON.parse(text);
    if (typeof value === 'object' && value !== null) saved = value as Record<string, unknown>;
  } catch {
    return DEFAULT_EDIT_PREFERENCES;
  }
  const seconds = saved['transitionSeconds'];
  const silences = saved['showSilences'];
  return {
    transitionSeconds:
      typeof seconds === 'number' &&
      seconds >= TRANSITION_SECONDS_RANGE.min &&
      seconds <= TRANSITION_SECONDS_RANGE.max
        ? seconds
        : DEFAULT_EDIT_PREFERENCES.transitionSeconds,
    showSilences: typeof silences === 'boolean' ? silences : DEFAULT_EDIT_PREFERENCES.showSilences,
  };
}

/** Restores the saved preferences and saves every change; returns the unsubscribe function. */
export function connectEditPreferences(storage: KeyValueStorage): () => void {
  editPreferencesStore.setState(parseEditPreferences(storage.getItem(STORAGE_KEY)));
  return editPreferencesStore.subscribe((state) => {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  });
}
