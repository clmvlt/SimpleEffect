/**
 * Design token values for canvas drawing (waveforms, ruler, overview), read once from the root
 * styles: calling `getComputedStyle` on every draw forces a style recalculation and was the
 * main cost of zooming the timeline. The theme never changes while the app runs.
 */
const cache = new Map<string, string>();

/** The value of the CSS custom property `name` (e.g. `--color-item-audio-strong`). */
export function tokenValue(name: `--${string}`): string {
  let value = cache.get(name);
  if (value === undefined) {
    value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (value !== '') cache.set(name, value);
  }
  return value;
}
