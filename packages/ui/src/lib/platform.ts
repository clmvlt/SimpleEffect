/**
 * Keyboard conventions of the user's OS. This is the only place in `@se/ui` that looks at the
 * platform (golden rule 10): Apple systems show ⌘ ⌥ ⇧ glyphs, the others spell `Ctrl`, `Alt`…
 */
export type KeyboardPlatform = 'apple' | 'other';

/** The navigator fields used to recognise an Apple platform. */
export type PlatformHints = Pick<Navigator, 'platform' | 'userAgent'>;

/**
 * Detects the keyboard platform from the browser. `navigator.platform` is deprecated but still the
 * most reliable signal in Electron; the user agent is the fallback.
 */
export function detectKeyboardPlatform(hints: PlatformHints = navigator): KeyboardPlatform {
  const source = hints.platform === '' ? hints.userAgent : hints.platform;
  return /mac|iphone|ipad|ipod/i.test(source) ? 'apple' : 'other';
}
