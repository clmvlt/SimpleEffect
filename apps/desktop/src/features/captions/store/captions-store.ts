import type { CaptionPresetId } from '@se/engine/caption-styles';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';
import type { StyleScope } from '../services/caption-rows';

/** The caption panel's preferences (saved under `se.ui.captions.v1`). */
export interface CaptionsPreferences {
  /** Who style changes apply to. */
  readonly scope: StyleScope;
  /** The style new captions get. */
  readonly preset: CaptionPresetId;
}

const STORAGE_KEY = 'captions.v1';
const PRESETS: readonly CaptionPresetId[] = ['tiktokBold', 'minimal', 'karaoke', 'box'];
const DEFAULTS: CaptionsPreferences = { scope: 'track', preset: 'karaoke' };

/** Saved preferences, defaults for anything missing or invalid. */
export function readCaptionsPreferences(text: string | null): CaptionsPreferences {
  try {
    const value: unknown = text === null ? null : JSON.parse(text);
    if (typeof value !== 'object' || value === null) return DEFAULTS;
    const { scope, preset } = value as Record<string, unknown>;
    return {
      scope: scope === 'selection' || scope === 'track' ? scope : DEFAULTS.scope,
      preset: PRESETS.find((candidate) => candidate === preset) ?? DEFAULTS.preset,
    };
  } catch {
    return DEFAULTS;
  }
}

/** The captions panel store. */
export const captionsStore = createStore<CaptionsPreferences>()(() => DEFAULTS);

/** Restores the saved preferences and saves every change; returns the unsubscribe function. */
export function connectCaptionsPersistence(storage: KeyValueStorage): () => void {
  captionsStore.setState(readCaptionsPreferences(storage.getItem(STORAGE_KEY)));
  return captionsStore.subscribe((state) => {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  });
}

/** Sets who style changes apply to. */
export function setStyleScope(scope: StyleScope): void {
  captionsStore.setState({ scope });
}

/** Sets the style of new captions. */
export function setGenerationPreset(preset: CaptionPresetId): void {
  captionsStore.setState({ preset });
}

/** The panel's preferences. */
export function useCaptionsPreferences(): CaptionsPreferences {
  return useStore(captionsStore);
}
