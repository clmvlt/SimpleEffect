import type { CaptionPresetId } from '@se/engine/caption-styles';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';
import type { StyleScope } from '../services/caption-rows';

/** The style new captions get: a preset, or the project's brand kit look (prompt 25). */
export type GenerationStyle = CaptionPresetId | 'brandKit';

/** The caption panel's preferences (saved under `se.ui.captions.v1`). */
export interface CaptionsPreferences {
  /** Who style changes apply to. */
  readonly scope: StyleScope;
  /** The style new captions get (the brand kit's falls back to `karaoke` without a kit). */
  readonly preset: GenerationStyle;
}

const STORAGE_KEY = 'captions.v1';
const STYLES: readonly GenerationStyle[] = ['brandKit', 'tiktokBold', 'minimal', 'karaoke', 'box'];
const DEFAULTS: CaptionsPreferences = { scope: 'track', preset: 'brandKit' };

/** The preset used when the brand kit is chosen but the project has none. */
export const FALLBACK_PRESET: CaptionPresetId = 'karaoke';

/** Saved preferences, defaults for anything missing or invalid. */
export function readCaptionsPreferences(text: string | null): CaptionsPreferences {
  try {
    const value: unknown = text === null ? null : JSON.parse(text);
    if (typeof value !== 'object' || value === null) return DEFAULTS;
    const { scope, preset } = value as Record<string, unknown>;
    return {
      scope: scope === 'selection' || scope === 'track' ? scope : DEFAULTS.scope,
      preset: STYLES.find((candidate) => candidate === preset) ?? DEFAULTS.preset,
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
export function setGenerationPreset(preset: GenerationStyle): void {
  captionsStore.setState({ preset });
}

/** The panel's preferences. */
export function useCaptionsPreferences(): CaptionsPreferences {
  return useStore(captionsStore);
}
