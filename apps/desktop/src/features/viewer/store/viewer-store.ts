import { SAFE_AREA_PLATFORMS, type SafeAreaPlatformId } from '@se/engine/safe-areas';
import { createStore } from 'zustand/vanilla';

import type { KeyValueStorage } from '../../../shared/storage';

/** Zoom of the viewer: fit the panel, or a fixed scale (1 = one sequence pixel per CSS pixel). */
export type ViewerZoom = 'fit' | 0.5 | 1 | 2;

/** Zoom levels offered, in menu order. */
export const VIEWER_ZOOMS: readonly ViewerZoom[] = ['fit', 0.5, 1, 2];

/** The guides drawn over the picture of one sequence (never rendered into frames or exports). */
export interface ViewerGuides {
  /** The platform whose safe areas show, or `null`. */
  readonly safeArea: SafeAreaPlatformId | null;
  /** The rule-of-thirds grid. */
  readonly thirds: boolean;
  /** A cross at the frame's centre. */
  readonly centre: boolean;
}

/** No guide. */
export const NO_GUIDES: ViewerGuides = { safeArea: null, thirds: false, centre: false };

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
  /** Guides chosen per sequence id (a 9:16 version keeps its TikTok zones, its 16:9 source not). */
  readonly guides: Readonly<Record<string, ViewerGuides>>;
}

/** Defaults on a new machine. */
export const DEFAULT_VIEWER_PREFERENCES: ViewerPreferences = {
  zoom: 'fit',
  proxy: false,
  checkerboard: false,
  loop: false,
  scrubAudio: true,
  hud: false,
  guides: {},
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

/** The guides of `sequenceId` (none until chosen). */
export function guidesOf(preferences: ViewerPreferences, sequenceId: string): ViewerGuides {
  return preferences.guides[sequenceId] ?? NO_GUIDES;
}

/** Changes the guides of one sequence. */
export function setSequenceGuides(sequenceId: string, change: Partial<ViewerGuides>): void {
  viewerStore.setState((state) => ({
    guides: { ...state.guides, [sequenceId]: { ...guidesOf(state, sequenceId), ...change } },
  }));
}

function isZoom(value: unknown): value is ViewerZoom {
  return VIEWER_ZOOMS.includes(value as ViewerZoom);
}

function isPlatform(value: unknown): value is SafeAreaPlatformId {
  return SAFE_AREA_PLATFORMS.some((platform) => platform.id === value);
}

/** Saved guides read back, entries that are not guides dropped. */
function parseGuides(value: unknown): Record<string, ViewerGuides> {
  if (typeof value !== 'object' || value === null) return {};
  const guides: Record<string, ViewerGuides> = {};
  for (const [sequenceId, entry] of Object.entries(value as Record<string, unknown>)) {
    if (typeof entry !== 'object' || entry === null) continue;
    const saved = entry as Record<string, unknown>;
    guides[sequenceId] = {
      safeArea: isPlatform(saved['safeArea']) ? saved['safeArea'] : null,
      thirds: saved['thirds'] === true,
      centre: saved['centre'] === true,
    };
  }
  return guides;
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
    guides: parseGuides(saved['guides']),
  };
}

/** Restores the saved preferences and saves every change; returns the unsubscribe function. */
export function connectViewerPersistence(storage: KeyValueStorage): () => void {
  viewerStore.setState(parseViewerPreferences(storage.getItem(STORAGE_KEY)));
  return viewerStore.subscribe((state) => {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  });
}
