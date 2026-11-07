import type { KeyboardPlatform } from '@se/ui';

import { parseShortcut } from './shortcut';
import {
  actionKeys,
  type ShortcutActionDefinition,
  type ShortcutContext,
} from './shortcut-actions';

/** Two or more actions answering the same combination in overlapping contexts. */
export interface ShortcutConflict {
  /** The combination, normalised (`mod+shift+z`). */
  readonly keys: string;
  /** The actions sharing it. */
  readonly actions: readonly string[];
}

/** A combination in one canonical spelling, so `Shift+Mod+Z` and `mod+shift+z` compare equal. */
function canonical(combination: string): string {
  const { key, mod, shift, alt } = parseShortcut(combination);
  return [mod && 'mod', alt && 'alt', shift && 'shift', key].filter(Boolean).join('+');
}

/** Whether two actions can be active at the same time (`global` overlaps every context). */
function overlap(a: readonly ShortcutContext[], b: readonly ShortcutContext[]): boolean {
  return a.includes('global') || b.includes('global') || a.some((context) => b.includes(context));
}

/**
 * The combinations bound to more than one action whose contexts overlap on `platform`: such a
 * key press would be ambiguous. The default table must have none.
 */
export function findShortcutConflicts(
  actions: Readonly<Record<string, ShortcutActionDefinition>>,
  platform: KeyboardPlatform,
): ShortcutConflict[] {
  const byKeys = new Map<string, [string, ShortcutActionDefinition][]>();
  for (const [id, action] of Object.entries(actions)) {
    for (const combination of new Set(actionKeys(action, platform).map(canonical))) {
      byKeys.set(combination, [...(byKeys.get(combination) ?? []), [id, action]]);
    }
  }
  const conflicts: ShortcutConflict[] = [];
  for (const [keys, entries] of byKeys) {
    const clashing = entries.filter(([id, action]) =>
      entries.some(
        ([otherId, other]) => otherId !== id && overlap(action.contexts, other.contexts),
      ),
    );
    if (clashing.length > 0) conflicts.push({ keys, actions: clashing.map(([id]) => id) });
  }
  return conflicts;
}
