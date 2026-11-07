import { createContext, use } from 'react';

import type { ShortcutRegistry } from './shortcut-registry';

/** The app's shortcut registry; `null` outside `ShortcutProvider`. */
export const ShortcutRegistryContext = createContext<ShortcutRegistry | null>(null);

/** The registry of the enclosing `ShortcutProvider`; throws outside one. */
export function useShortcutRegistry(): ShortcutRegistry {
  const registry = use(ShortcutRegistryContext);
  if (registry === null) {
    throw new Error('Shortcut hooks must be used inside <ShortcutProvider>.');
  }
  return registry;
}
