import type { Flicks, FrameRate } from '@se/protocol';

import {
  clampFlicks,
  flicksToPixels,
  flicksToSeconds,
  frameDuration,
  pixelsToFlicks,
  ZERO_FLICKS,
} from '../../../shared/time';

/**
 * The visible window of the timeline: zoom (pixels per second), the time at the left edge of the
 * lanes, and the lanes' width in pixels. View state only (never document data), so floats are
 * fine for the zoom; times stay flicks.
 */
export interface Viewport {
  readonly pixelsPerSecond: number;
  readonly scrollTime: Flicks;
  readonly width: number;
}

/** Zoom bounds for a sequence. */
export interface ZoomLimits {
  readonly min: number;
  readonly max: number;
}

/** Zoom used before the first fit (50 px per second). */
export const DEFAULT_PIXELS_PER_SECOND = 50;

/** Furthest zoom out: 1 px for 100 s (a 2-hour film in 72 px). */
const MIN_PIXELS_PER_SECOND = 0.01;

/** Closest zoom in: one frame spans 120 px. */
const MAX_PIXELS_PER_FRAME = 120;

/** Free space after the last item, as a fraction of the visible width. */
const TRAILING_SPACE = 0.5;

/** Zoom bounds at `rate`. */
export function zoomLimits(rate: FrameRate): ZoomLimits {
  return {
    min: MIN_PIXELS_PER_SECOND,
    max: (MAX_PIXELS_PER_FRAME * rate.numerator) / rate.denominator,
  };
}

/** `pixelsPerSecond` kept inside `limits`. */
export function clampZoom(pixelsPerSecond: number, limits: ZoomLimits): number {
  if (!Number.isFinite(pixelsPerSecond)) return DEFAULT_PIXELS_PER_SECOND;
  return Math.min(limits.max, Math.max(limits.min, pixelsPerSecond));
}

/** X of `time` in the lanes (0 = left edge of the visible window). */
export function timeToX(time: Flicks, viewport: Viewport): number {
  return flicksToPixels((time - viewport.scrollTime) as Flicks, viewport.pixelsPerSecond);
}

/** Time under `x` (pixels from the left edge of the lanes), never before zero. */
export function xToTime(x: number, viewport: Viewport): Flicks {
  const time = viewport.scrollTime + pixelsToFlicks(x, viewport.pixelsPerSecond);
  return clampFlicks(Math.max(0, time));
}

/** Flicks spanned by `pixels` at the viewport's zoom. */
export function pixelsToDuration(pixels: number, viewport: Viewport): Flicks {
  return pixelsToFlicks(pixels, viewport.pixelsPerSecond);
}

/** The times shown, widened by `overscan` pixels on each side. */
export function visibleRange(
  viewport: Viewport,
  overscan = 0,
): { readonly start: Flicks; readonly end: Flicks } {
  return {
    start: xToTime(-overscan, viewport),
    end: clampFlicks(
      viewport.scrollTime + pixelsToFlicks(viewport.width + overscan, viewport.pixelsPerSecond),
    ),
  };
}

/** Latest left-edge time: the content end plus some free space stays reachable. */
export function maxScrollTime(viewport: Viewport, contentEnd: Flicks): Flicks {
  const visible = pixelsToFlicks(viewport.width, viewport.pixelsPerSecond);
  const trailing = pixelsToFlicks(viewport.width * TRAILING_SPACE, viewport.pixelsPerSecond);
  return clampFlicks(Math.max(0, contentEnd + trailing - visible));
}

/** `viewport` with its scroll kept between zero and {@link maxScrollTime}. */
export function clampScroll(viewport: Viewport, contentEnd: Flicks): Viewport {
  const scrollTime = clampFlicks(
    Math.min(maxScrollTime(viewport, contentEnd), Math.max(0, viewport.scrollTime)),
  );
  return scrollTime === viewport.scrollTime ? viewport : { ...viewport, scrollTime };
}

/** `viewport` scrolled horizontally by `pixels` (positive = later). */
export function scrollByPixels(viewport: Viewport, pixels: number, contentEnd: Flicks): Viewport {
  const scrollTime = clampFlicks(
    viewport.scrollTime + pixelsToFlicks(pixels, viewport.pixelsPerSecond),
  );
  return clampScroll({ ...viewport, scrollTime }, contentEnd);
}

/**
 * `viewport` zoomed to `pixelsPerSecond` (clamped) with the time under `anchorX` staying under it
 * (the cursor for Ctrl+wheel, the playhead for the keyboard).
 */
export function zoomAround(
  viewport: Viewport,
  pixelsPerSecond: number,
  anchorX: number,
  limits: ZoomLimits,
): Viewport {
  const zoom = clampZoom(pixelsPerSecond, limits);
  const anchorTime = viewport.scrollTime + pixelsToFlicks(anchorX, viewport.pixelsPerSecond);
  const scrollTime = clampFlicks(Math.max(0, anchorTime - pixelsToFlicks(anchorX, zoom)));
  return { ...viewport, pixelsPerSecond: zoom, scrollTime };
}

/** The viewport showing `[0, contentEnd]` across 95 % of the width (the default zoom if empty). */
export function fitViewport(viewport: Viewport, contentEnd: Flicks, limits: ZoomLimits): Viewport {
  const seconds = flicksToSeconds(contentEnd);
  const zoom =
    seconds > 0 && viewport.width > 0
      ? clampZoom((viewport.width * 0.95) / seconds, limits)
      : DEFAULT_PIXELS_PER_SECOND;
  return { ...viewport, pixelsPerSecond: zoom, scrollTime: ZERO_FLICKS };
}

/** `viewport` scrolled so `time` is visible, leaving it where it is when it already shows. */
export function revealTime(viewport: Viewport, time: Flicks, contentEnd: Flicks): Viewport {
  const x = timeToX(time, viewport);
  if (x >= 0 && x <= viewport.width) return viewport;
  const margin = pixelsToFlicks(viewport.width * 0.1, viewport.pixelsPerSecond);
  const scrollTime =
    x < 0
      ? time - margin
      : time - (pixelsToFlicks(viewport.width, viewport.pixelsPerSecond) - margin);
  return clampScroll({ ...viewport, scrollTime: clampFlicks(Math.max(0, scrollTime)) }, contentEnd);
}

/** Slider positions of the zoom control (logarithmic: each step is the same zoom ratio). */
export const ZOOM_SLIDER_MAX = 1000;

/** Slider position of `pixelsPerSecond`. */
export function zoomToSlider(pixelsPerSecond: number, limits: ZoomLimits): number {
  const ratio = Math.log(clampZoom(pixelsPerSecond, limits) / limits.min);
  return Math.round((ratio / Math.log(limits.max / limits.min)) * ZOOM_SLIDER_MAX);
}

/** Zoom of slider position `value`. */
export function sliderToZoom(value: number, limits: ZoomLimits): number {
  return limits.min * Math.exp((value / ZOOM_SLIDER_MAX) * Math.log(limits.max / limits.min));
}

/** Pixels of one frame at the viewport's zoom. */
export function framePixels(viewport: Viewport, rate: FrameRate): number {
  return flicksToPixels(frameDuration(rate) as Flicks, viewport.pixelsPerSecond);
}
