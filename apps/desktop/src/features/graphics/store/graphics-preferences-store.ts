import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';

/** Preferences of the graphics tools: machine preferences, never project data. */
export interface GraphicsPreferences {
  /** Length of new texts, shapes and images, in seconds. */
  readonly graphicSeconds: number;
  /** Length of applied animations in seconds; `null` uses each preset's own. */
  readonly animationSeconds: number | null;
}

/** Shortest and longest graphic, in seconds. */
export const GRAPHIC_SECONDS_RANGE = { min: 0.5, max: 600 } as const;

/** Shortest and longest animation, in seconds. */
export const ANIMATION_SECONDS_RANGE = { min: 0.1, max: 10 } as const;

/** Defaults on a new machine. */
export const DEFAULT_GRAPHICS_PREFERENCES: GraphicsPreferences = {
  graphicSeconds: 5,
  animationSeconds: null,
};

const STORAGE_KEY = 'graphics.v1';

/** The graphics preferences store. */
export const graphicsPreferencesStore = createStore<GraphicsPreferences>()(
  () => DEFAULT_GRAPHICS_PREFERENCES,
);

function clamp(value: number, range: { readonly min: number; readonly max: number }): number {
  return Math.min(range.max, Math.max(range.min, value));
}

/** Changes some graphics preferences (lengths kept in range). */
export function setGraphicsPreferences(change: Partial<GraphicsPreferences>): void {
  const next = { ...graphicsPreferencesStore.getState(), ...change };
  graphicsPreferencesStore.setState({
    graphicSeconds: clamp(next.graphicSeconds, GRAPHIC_SECONDS_RANGE),
    animationSeconds:
      next.animationSeconds === null ? null : clamp(next.animationSeconds, ANIMATION_SECONDS_RANGE),
  });
}

/** The graphics preferences. */
export function useGraphicsPreferences(): GraphicsPreferences {
  return useStore(graphicsPreferencesStore);
}

/** Saved preferences read back; anything malformed falls back to the defaults. */
export function parseGraphicsPreferences(text: string | null): GraphicsPreferences {
  let saved: Record<string, unknown> = {};
  try {
    const value: unknown = text === null ? null : JSON.parse(text);
    if (typeof value === 'object' && value !== null) saved = value as Record<string, unknown>;
  } catch {
    return DEFAULT_GRAPHICS_PREFERENCES;
  }
  const graphic = saved['graphicSeconds'];
  const animation = saved['animationSeconds'];
  return {
    graphicSeconds:
      typeof graphic === 'number' && Number.isFinite(graphic)
        ? clamp(graphic, GRAPHIC_SECONDS_RANGE)
        : DEFAULT_GRAPHICS_PREFERENCES.graphicSeconds,
    animationSeconds:
      typeof animation === 'number' && Number.isFinite(animation)
        ? clamp(animation, ANIMATION_SECONDS_RANGE)
        : null,
  };
}

/** Restores the saved preferences and saves every change; returns the unsubscribe function. */
export function connectGraphicsPersistence(storage: KeyValueStorage): () => void {
  graphicsPreferencesStore.setState(parseGraphicsPreferences(storage.getItem(STORAGE_KEY)));
  return graphicsPreferencesStore.subscribe((state) => {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  });
}
