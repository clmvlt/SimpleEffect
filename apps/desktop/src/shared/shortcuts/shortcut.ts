import type { KeyboardPlatform } from '@se/ui';

/**
 * A keyboard shortcut written `mod+shift+z`: `mod` is ⌘ on Apple systems and Ctrl elsewhere,
 * then `shift`, `alt`, and the key (as `KeyboardEvent.key`, case-insensitive).
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
  'key' | 'metaKey' | 'ctrlKey' | 'shiftKey' | 'altKey'
>;

const MODIFIERS = new Set(['mod', 'shift', 'alt']);

/** Parses `mod+shift+z`. Throws on a combination without a key or with an unknown modifier. */
export function parseShortcut(combination: string): Shortcut {
  const parts = combination
    .toLowerCase()
    .split('+')
    .map((part) => part.trim());
  const key = parts.at(-1);
  const modifiers = parts.slice(0, -1);
  const unknown = modifiers.find((modifier) => !MODIFIERS.has(modifier));
  if (key === undefined || key === '' || MODIFIERS.has(key) || unknown !== undefined) {
    throw new Error(`invalid shortcut "${combination}": write it like "mod+shift+z"`);
  }
  return {
    key,
    mod: modifiers.includes('mod'),
    shift: modifiers.includes('shift'),
    alt: modifiers.includes('alt'),
  };
}

/** Whether `event` is exactly `shortcut` on `platform` (no extra modifier pressed). */
export function matchesShortcut(
  event: ShortcutEvent,
  shortcut: Shortcut,
  platform: KeyboardPlatform,
): boolean {
  const [modPressed, otherPressed] =
    platform === 'apple' ? [event.metaKey, event.ctrlKey] : [event.ctrlKey, event.metaKey];
  return (
    event.key.toLowerCase() === shortcut.key &&
    modPressed === shortcut.mod &&
    !otherPressed &&
    event.shiftKey === shortcut.shift &&
    event.altKey === shortcut.alt
  );
}
