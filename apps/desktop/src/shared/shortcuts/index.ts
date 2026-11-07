/**
 * Keyboard shortcuts: the table of named actions with OS-adapted default keys
 * (`SHORTCUT_ACTIONS`), the registry dispatching `keydown` to the handlers bound with
 * `useShortcutAction` (global actions everywhere, others in the active `ShortcutRegion`), conflict
 * detection, and the key caps shown in tooltips and help screens.
 */
export { matchesShortcut, parseShortcut, shortcutDisplayKeys } from './shortcut';
export type { Shortcut, ShortcutEvent } from './shortcut';
export {
  actionKeys,
  SHORTCUT_ACTION_IDS,
  SHORTCUT_ACTIONS,
  SHORTCUT_CONTEXTS,
} from './shortcut-actions';
export type {
  ShortcutActionDefinition,
  ShortcutActionId,
  ShortcutActionTable,
  ShortcutContext,
} from './shortcut-actions';
export { findShortcutConflicts } from './shortcut-conflicts';
export type { ShortcutConflict } from './shortcut-conflicts';
export { useShortcutRegistry } from './shortcut-context';
export { ShortcutRegistry } from './shortcut-registry';
export { SHORTCUT_CONTEXT_ATTRIBUTE, ShortcutProvider } from './ShortcutProvider';
export type { ShortcutProviderProps } from './ShortcutProvider';
export { ShortcutRegion } from './ShortcutRegion';
export type { ShortcutRegionProps } from './ShortcutRegion';
export { useActiveShortcutContext } from './use-active-shortcut-context';
export { useShortcutAction } from './use-shortcut-action';
export { useShortcutDisplayKeys, useShortcutHint, useShortcutKeyLabels } from './use-shortcut-hint';
