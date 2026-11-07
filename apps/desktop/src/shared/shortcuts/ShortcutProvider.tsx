import { useEffect, type ReactNode } from 'react';

import { ShortcutContext } from './shortcut-context';
import type { ShortcutRegistry } from './shortcut-registry';

/** Props of {@link ShortcutProvider}. */
export interface ShortcutProviderProps {
  readonly registry: ShortcutRegistry;
  readonly children: ReactNode;
}

/** Listens to key presses on the window and dispatches them to `registry`. */
export function ShortcutProvider({ registry, children }: ShortcutProviderProps) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      registry.handleKeyDown(event);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [registry]);
  return <ShortcutContext value={registry}>{children}</ShortcutContext>;
}
