import { use, useEffect, useEffectEvent } from 'react';

import { ShortcutContext } from './shortcut-context';

/**
 * Binds `keys` (e.g. `mod+z`) to `run` while the component is mounted and `enabled` is true.
 * The latest `run` is always called, without re-registering when it changes.
 */
export function useShortcut(keys: string, run: () => void, enabled = true): void {
  const registry = use(ShortcutContext);
  if (registry === null) {
    throw new Error('useShortcut must be used inside <ShortcutProvider>.');
  }
  const onPress = useEffectEvent(run);
  useEffect(() => {
    if (!enabled) return undefined;
    return registry.register({
      keys,
      run: () => {
        onPress();
      },
    });
  }, [registry, keys, enabled]);
}
