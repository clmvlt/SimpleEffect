import type { KeyboardPlatform } from '@se/ui';

/**
 * Where a shortcut applies: `global` everywhere, the others only while that region is active
 * (last clicked or focused, see `ShortcutRegion`).
 */
export type ShortcutContext = 'global' | 'timeline' | 'viewer' | 'transcript';

/** Every shortcut context, in the order help screens list them. */
export const SHORTCUT_CONTEXTS: readonly ShortcutContext[] = [
  'global',
  'timeline',
  'viewer',
  'transcript',
];

/** A named action and its default keys. */
export interface ShortcutActionDefinition {
  /** Regions where the action applies. */
  readonly contexts: readonly ShortcutContext[];
  /** Default combinations (`mod+shift+z`, see `parseShortcut`), first one shown in tooltips. */
  readonly keys: readonly string[];
  /** Replaces `keys` on Apple systems when the usual combination differs there. */
  readonly appleKeys?: readonly string[];
  /** Whether holding the keys repeats the action (stepping frames, zooming). */
  readonly repeats?: boolean;
}

/** Playhead navigation works wherever time is shown (the transcript too, prompt 22). */
const TIME_CONTEXTS: readonly ShortcutContext[] = ['timeline', 'viewer', 'transcript'];

/**
 * The app's named actions and their default shortcuts (the single source of truth for key
 * bindings, help screens and tooltips). Labels are `shortcuts.actions.<id>` in `fr.json`.
 * `findShortcutConflicts` must stay empty for this table (tested).
 */
export const SHORTCUT_ACTIONS = {
  undo: { contexts: ['global'], keys: ['mod+z'], repeats: true },
  redo: {
    contexts: ['global'],
    keys: ['mod+shift+z', 'mod+y'],
    appleKeys: ['mod+shift+z'],
    repeats: true,
  },
  showShortcuts: { contexts: ['global'], keys: ['?'] },
  askAi: { contexts: ['global'], keys: ['mod+i'] },
  togglePlay: { contexts: TIME_CONTEXTS, keys: ['space'] },
  // JKL: J plays in reverse, K pauses, L plays forward (repeated presses speed up); holding K
  // while pressing J or L steps one frame.
  shuttleReverse: { contexts: TIME_CONTEXTS, keys: ['j'] },
  shuttlePause: { contexts: TIME_CONTEXTS, keys: ['k'] },
  shuttleForward: { contexts: TIME_CONTEXTS, keys: ['l'] },
  toggleLoop: { contexts: TIME_CONTEXTS, keys: ['mod+l'] },
  goToStart: { contexts: TIME_CONTEXTS, keys: ['home'] },
  goToEnd: { contexts: TIME_CONTEXTS, keys: ['end'] },
  previousFrame: { contexts: TIME_CONTEXTS, keys: ['arrowleft'], repeats: true },
  nextFrame: { contexts: TIME_CONTEXTS, keys: ['arrowright'], repeats: true },
  backTenFrames: { contexts: TIME_CONTEXTS, keys: ['shift+arrowleft'], repeats: true },
  forwardTenFrames: { contexts: TIME_CONTEXTS, keys: ['shift+arrowright'], repeats: true },
  previousEditPoint: { contexts: TIME_CONTEXTS, keys: ['arrowup'], repeats: true },
  nextEditPoint: { contexts: TIME_CONTEXTS, keys: ['arrowdown'], repeats: true },
  markIn: { contexts: TIME_CONTEXTS, keys: ['i'] },
  markOut: { contexts: TIME_CONTEXTS, keys: ['o'] },
  clearInOut: { contexts: TIME_CONTEXTS, keys: ['alt+x'] },
  addMarker: { contexts: ['timeline'], keys: ['m'] },
  zoomIn: { contexts: ['timeline'], keys: ['=', 'plus'], repeats: true },
  zoomOut: { contexts: ['timeline'], keys: ['-'], repeats: true },
  zoomToFit: { contexts: ['timeline'], keys: ['shift+z'] },
  toggleSnapping: { contexts: ['timeline'], keys: ['n'] },
  selectAll: { contexts: ['timeline'], keys: ['mod+a'] },
  deselectAll: { contexts: ['timeline'], keys: ['mod+shift+a'] },
  splitAtPlayhead: { contexts: ['timeline'], keys: ['s', 'mod+k'] },
  addTransition: { contexts: ['timeline'], keys: ['mod+t'] },
  addText: { contexts: TIME_CONTEXTS, keys: ['t'] },
  deleteSelection: { contexts: ['timeline'], keys: ['delete', 'backspace'] },
  rippleDelete: { contexts: ['timeline'], keys: ['shift+delete', 'shift+backspace'] },
  deleteWords: { contexts: ['transcript'], keys: ['delete', 'backspace'] },
  copy: { contexts: ['timeline'], keys: ['mod+c'] },
  cut: { contexts: ['timeline'], keys: ['mod+x'] },
  paste: { contexts: ['timeline'], keys: ['mod+v'] },
  pasteInsert: { contexts: ['timeline'], keys: ['mod+shift+v'] },
  nudgeLeft: { contexts: ['timeline'], keys: [','], repeats: true },
  nudgeRight: { contexts: ['timeline'], keys: ['.'], repeats: true },
  // `<` and `>` are Maj+, and Maj+. on QWERTY keyboards and have their own key on AZERTY ones.
  nudgeLeftTen: { contexts: ['timeline'], keys: ['<'], repeats: true },
  nudgeRightTen: { contexts: ['timeline'], keys: ['>'], repeats: true },
} as const satisfies Record<string, ShortcutActionDefinition>;

/** Id of a named action. */
export type ShortcutActionId = keyof typeof SHORTCUT_ACTIONS;

/** The action table, typed for lookups. */
export type ShortcutActionTable = Readonly<Record<ShortcutActionId, ShortcutActionDefinition>>;

/** Every action id, in table order. */
export const SHORTCUT_ACTION_IDS = Object.keys(SHORTCUT_ACTIONS) as ShortcutActionId[];

/** The combinations of `action` on `platform`. */
export function actionKeys(
  action: ShortcutActionDefinition,
  platform: KeyboardPlatform,
): readonly string[] {
  return platform === 'apple' ? (action.appleKeys ?? action.keys) : action.keys;
}
