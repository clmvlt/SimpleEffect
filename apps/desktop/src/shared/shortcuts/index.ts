/**
 * Global keyboard shortcuts: a registry dispatching `keydown` events (ignored in text fields),
 * its provider and `useShortcut(keys, run)`. Minimal for prompt 07 (undo/redo); prompt 10 adds
 * scopes and listings.
 */
export { matchesShortcut, parseShortcut } from './shortcut';
export type { Shortcut, ShortcutEvent } from './shortcut';
export { ShortcutRegistry } from './shortcut-registry';
export type { ShortcutBinding } from './shortcut-registry';
export { ShortcutProvider } from './ShortcutProvider';
export type { ShortcutProviderProps } from './ShortcutProvider';
export { useShortcut } from './use-shortcut';
