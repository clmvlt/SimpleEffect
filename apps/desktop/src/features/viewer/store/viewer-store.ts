import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';

/** Zoom of the viewer: fit the panel, or a fixed scale (1 = one sequence pixel per CSS pixel). */
export type ViewerZoom = 'fit' | 0.5 | 1 | 2;

/** Zoom levels offered, in menu order. */
export const VIEWER_ZOOMS: readonly ViewerZoom[] = ['fit', 0.5, 1, 2];

/** Viewer options: machine preferences, never project data. */
export interface ViewerPreferences {
  readonly zoom: ViewerZoom;
  /** Decode ready proxies instead of originals. */
  readonly proxy: boolean;
  /** Show transparent areas over a checkerboard (instead of black). */
  readonly checkerboard: boolean;
  /** Loop playback over the in/out span (the whole content without one). */
  readonly loop: boolean;
  /** Play short snippets of sound when the paused playhead moves. */
  readonly scrubAudio: boolean;
  /** Show the performance HUD over the picture. */
  readonly hud: boolean;
}

/** Defaults on a new machine. */
export const DEFAULT_VIEWER_PREFERENCES: ViewerPreferences = {
  zoom: 'fit',
  proxy: false,
  checkerboard: false,
  loop: false,
  scrubAudio: true,
  hud: false,
};

/** The on/off preferences, read back from storage the same way. */
type ViewerFlag = 'proxy' | 'checkerboard' | 'loop' | 'scrubAudio' | 'hud';

const STORAGE_KEY = 'viewer.v1';

/** The viewer preferences store. */
export const viewerStore = createStore<ViewerPreferences>()(() => DEFAULT_VIEWER_PREFERENCES);

/** Changes some viewer preferences. */
export function setViewerPreferences(change: Partial<ViewerPreferences>): void {
  viewerStore.setState(change);
}

function isZoom(value: unknown): value is ViewerZoom {
  return VIEWER_ZOOMS.includes(value as ViewerZoom);
}

/** Saved preferences read back; anything malformed falls back to the defaults. */
export function parseViewerPreferences(text: string | null): ViewerPreferences {
  if (text === null) return DEFAULT_VIEWER_PREFERENCES;
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return DEFAULT_VIEWER_PREFERENCES;
  }
  if (typeof value !== 'object' || value === null) return DEFAULT_VIEWER_PREFERENCES;
  const saved = value as Record<string, unknown>;
  const flag = (name: ViewerFlag) => {
    const stored = saved[name];
    return typeof stored === 'boolean' ? stored : DEFAULT_VIEWER_PREFERENCES[name];
  };
  return {
    zoom: isZoom(saved['zoom']) ? saved['zoom'] : DEFAULT_VIEWER_PREFERENCES.zoom,
    proxy: flag('proxy'),
    checkerboard: flag('checkerboard'),
    loop: flag('loop'),
    scrubAudio: flag('scrubAudio'),
    hud: flag('hud'),
  };
}

/** Restores the saved preferences and saves every change; returns the unsubscribe function. */
export function connectViewerPersistence(storage: KeyValueStorage): () => void {
  viewerStore.setState(parseViewerPreferences(storage.getItem(STORAGE_KEY)));
  return viewerStore.subscribe((state) => {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  });
}
