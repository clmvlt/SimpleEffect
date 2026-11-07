import type { KeyboardPlatform, ShortcutKey } from '@se/ui';

/**
 * A keyboard shortcut written `mod+shift+z`: `mod` is ⌘ on Apple systems and Ctrl elsewhere,
 * then `shift`, `alt`, and the key as `KeyboardEvent.key` (case-insensitive: `arrowleft`,
 * `home`, `?`). `plus` stands for `+` (the separator) and `space` for the space bar.
 */
export interface Shortcut {
  readonly key: string;
  readonly mod: boolean;
  readonly shift: boolean;
  readonly alt: boolean;
}

/** The parts of a keyboard event a shortcut looks at. */
export type ShortcutEvent = Pick<
  KeyboardEvent,
  'key' | 'code' | 'metaKey' | 'ctrlKey' | 'shiftKey' | 'altKey'
>;

const MODIFIERS = new Set(['mod', 'shift', 'alt']);

/** Names usable in a combination for keys that cannot be written there. */
const KEY_ALIASES: Readonly<Record<string, string>> = { plus: '+', space: ' ' };

/** Display names (`@se/ui` `ShortcutKey`) of named keys, by `KeyboardEvent.key` in lower case. */
const DISPLAY_NAMES: Readonly<Record<string, ShortcutKey>> = {
  ' ': 'Space',
  arrowup: 'ArrowUp',
  arrowdown: 'ArrowDown',
  arrowleft: 'ArrowLeft',
  arrowright: 'ArrowRight',
  escape: 'Escape',
  enter: 'Enter',
  backspace: 'Backspace',
  delete: 'Delete',
  tab: 'Tab',
  home: 'Home',
  end: 'End',
};

/** Parses `mod+shift+z`. Throws on a combination without a key or with an unknown modifier. */
export function parseShortcut(combination: string): Shortcut {
  const parts = combination
    .toLowerCase()
    .split('+')
    .map((part) => part.trim());
  const written = parts.at(-1);
  const modifiers = parts.slice(0, -1);
  const unknown = modifiers.find((modifier) => !MODIFIERS.has(modifier));
  if (written === undefined || written === '' || MODIFIERS.has(written) || unknown !== undefined) {
    throw new Error(`invalid shortcut "${combination}": write it like "mod+shift+z"`);
  }
  return {
    key: KEY_ALIASES[written] ?? written,
    mod: modifiers.includes('mod'),
    shift: modifiers.includes('shift'),
    alt: modifiers.includes('alt'),
  };
}

/** A printed symbol (`?`, `+`, `-`): typing it may need Shift depending on the layout. */
function isSymbol(key: string): boolean {
  return key.length === 1 && !/[\p{L}\p{N}\s]/u.test(key);
}

/**
 * The key of `event` in lower case. With ⌥ on Apple systems `key` is the composed character
 * (⌥X gives `≈`), so letters and digits are read from the physical `code` instead.
 */
function pressedKey(event: ShortcutEvent, platform: KeyboardPlatform): string {
  if (platform === 'apple' && event.altKey) {
    const physical = /^(?:Key([A-Z])|Digit([0-9]))$/u.exec(event.code);
    const character = physical?.[1] ?? physical?.[2];
    if (character !== undefined) return character.toLowerCase();
  }
  return event.key.toLowerCase();
}

/**
 * Whether `event` is exactly `shortcut` on `platform` (no extra modifier pressed). Shift is not
 * compared for symbols, whose Shift depends on the keyboard layout (`?` is Shift+, in French).
 */
export function matchesShortcut(
  event: ShortcutEvent,
  shortcut: Shortcut,
  platform: KeyboardPlatform,
): boolean {
  const [modPressed, otherPressed] =
    platform === 'apple' ? [event.metaKey, event.ctrlKey] : [event.ctrlKey, event.metaKey];
  return (
    pressedKey(event, platform) === shortcut.key &&
    modPressed === shortcut.mod &&
    !otherPressed &&
    (isSymbol(shortcut.key) || event.shiftKey === shortcut.shift) &&
    event.altKey === shortcut.alt
  );
}

/** Platform-neutral key caps of a shortcut for `KeyboardShortcut`/`formatShortcutKeys`. */
export function shortcutDisplayKeys(shortcut: Shortcut): ShortcutKey[] {
  const modifiers: ShortcutKey[] = [];
  if (shortcut.mod) modifiers.push('Mod');
  if (shortcut.alt) modifiers.push('Alt');
  if (shortcut.shift) modifiers.push('Shift');
  return [...modifiers, DISPLAY_NAMES[shortcut.key] ?? shortcut.key.toUpperCase()];
}
