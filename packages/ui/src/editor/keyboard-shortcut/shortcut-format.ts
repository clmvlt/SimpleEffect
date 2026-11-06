import type { KeyboardPlatform } from '../../lib/platform';

/**
 * Platform-neutral key names understood by {@link formatShortcutKeys}. `Mod` is the primary
 * modifier: ⌘ on Apple systems, Ctrl elsewhere. Any other string (a letter, `F5`…) is shown as is.
 */
export type ShortcutKey =
  | 'Mod'
  | 'Ctrl'
  | 'Alt'
  | 'Shift'
  | 'Enter'
  | 'Backspace'
  | 'Delete'
  | 'Escape'
  | 'Space'
  | 'Tab'
  | 'ArrowUp'
  | 'ArrowDown'
  | 'ArrowLeft'
  | 'ArrowRight'
  | (string & {});

/** Spelled-out names of keys on non-Apple keyboards; the app passes localised ones. */
export type ShortcutKeyLabels = Partial<
  Record<
    'Ctrl' | 'Alt' | 'Shift' | 'Enter' | 'Backspace' | 'Delete' | 'Escape' | 'Space' | 'Tab',
    string
  >
>;

/** Default (English) spelled-out names. */
export const DEFAULT_SHORTCUT_KEY_LABELS: Required<ShortcutKeyLabels> = {
  Ctrl: 'Ctrl',
  Alt: 'Alt',
  Shift: 'Shift',
  Enter: 'Enter',
  Backspace: 'Backspace',
  Delete: 'Del',
  Escape: 'Esc',
  Space: 'Space',
  Tab: 'Tab',
};

/** Glyphs printed on Apple keyboards and menus. */
const APPLE_GLYPHS: Readonly<Record<string, string>> = {
  Mod: '⌘',
  Ctrl: '⌃',
  Alt: '⌥',
  Shift: '⇧',
  Enter: '↩',
  Backspace: '⌫',
  Delete: '⌦',
  Escape: '⎋',
  Tab: '⇥',
};

/** Arrow glyphs, shared by every platform. */
const ARROW_GLYPHS: Readonly<Record<string, string>> = {
  ArrowUp: '↑',
  ArrowDown: '↓',
  ArrowLeft: '←',
  ArrowRight: '→',
};

function isLabelledKey(key: string): key is keyof ShortcutKeyLabels {
  return key in DEFAULT_SHORTCUT_KEY_LABELS;
}

/**
 * The key caps to print for a shortcut on `platform`: `['Mod', 'Shift', 'Z']` gives
 * `['⌘', '⇧', 'Z']` on Apple systems and `['Ctrl', 'Shift', 'Z']` elsewhere. List modifiers first,
 * in the order Mod, Ctrl, Alt, Shift.
 */
export function formatShortcutKeys(
  keys: readonly ShortcutKey[],
  platform: KeyboardPlatform,
  labels: ShortcutKeyLabels = {},
): string[] {
  const names = { ...DEFAULT_SHORTCUT_KEY_LABELS, ...labels };
  return keys.map((key) => {
    const arrow = ARROW_GLYPHS[key];
    if (arrow !== undefined) {
      return arrow;
    }
    if (platform === 'apple') {
      return APPLE_GLYPHS[key] ?? (key === 'Space' ? names.Space : key);
    }
    if (key === 'Mod') {
      return names.Ctrl;
    }
    return isLabelledKey(key) ? names[key] : key;
  });
}
