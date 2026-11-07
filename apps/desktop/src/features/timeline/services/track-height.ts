/** Track heights in pixels: bounds, presets and wheel resizing. */

/** Smallest track: the label bar and a thin picture. */
export const MIN_TRACK_HEIGHT = 24;

/** Tallest track. */
export const MAX_TRACK_HEIGHT = 200;

/** Height presets offered for every track at once. */
export const TRACK_HEIGHT_PRESETS = { compact: 32, normal: 52, large: 96 } as const;

/** A preset name. */
export type TrackHeightPreset = keyof typeof TRACK_HEIGHT_PRESETS;

/** Height of a track the user never resized. */
export const DEFAULT_TRACK_HEIGHT = TRACK_HEIGHT_PRESETS.normal;

/** `height` rounded and kept within the bounds. */
export function clampTrackHeight(height: number): number {
  if (!Number.isFinite(height)) return DEFAULT_TRACK_HEIGHT;
  return Math.round(Math.min(MAX_TRACK_HEIGHT, Math.max(MIN_TRACK_HEIGHT, height)));
}

/** The height after an Alt+wheel step over the track (wheel up = taller). */
export function wheelResizedHeight(height: number, deltaY: number): number {
  const step = Math.sign(-deltaY) * Math.max(4, Math.round(height * 0.12));
  return clampTrackHeight(height + step);
}
