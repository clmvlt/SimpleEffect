import { useSyncExternalStore } from 'react';

import type { ShortcutContext } from './shortcut-actions';
import { useShortcutRegistry } from './shortcut-context';

/** The active shortcut context (region last clicked or focused), re-rendering when it changes. */
export function useActiveShortcutContext(): ShortcutContext | null {
  const registry = useShortcutRegistry();
  return useSyncExternalStore(
    (listener) => registry.subscribe(listener),
    () => registry.activeContext,
  );
}
