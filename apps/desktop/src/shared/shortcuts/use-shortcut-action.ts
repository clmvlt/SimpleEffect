import { useEffect, useEffectEvent } from 'react';

import type { ShortcutActionId } from './shortcut-actions';
import { useShortcutRegistry } from './shortcut-context';

/**
 * Runs `run` when the keys of `action` are pressed (in its contexts), while the component is
 * mounted and `enabled` is true. The latest `run` is always called, without re-binding.
 */
export function useShortcutAction(action: ShortcutActionId, run: () => void, enabled = true): void {
  const registry = useShortcutRegistry();
  const onPress = useEffectEvent(run);
  useEffect(() => {
    if (!enabled) return undefined;
    return registry.bind(action, () => {
      onPress();
    });
  }, [registry, action, enabled]);
}
