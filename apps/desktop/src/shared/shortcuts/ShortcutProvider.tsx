import { useEffect, type ReactNode } from 'react';

import type { ShortcutContext } from './shortcut-actions';
import { ShortcutRegistryContext } from './shortcut-context';
import type { ShortcutRegistry } from './shortcut-registry';

/** Attribute naming the shortcut context of a region (set by `ShortcutRegion`). */
export const SHORTCUT_CONTEXT_ATTRIBUTE = 'data-shortcut-context';

/** The context of the region holding `target`, or `null` outside every region. */
function contextOf(target: EventTarget | null): ShortcutContext | null {
  if (typeof Element === 'undefined' || !(target instanceof Element)) return null;
  const region = target.closest(`[${SHORTCUT_CONTEXT_ATTRIBUTE}]`);
  return (region?.getAttribute(SHORTCUT_CONTEXT_ATTRIBUTE) as ShortcutContext | null) ?? null;
}

/** Props of {@link ShortcutProvider}. */
export interface ShortcutProviderProps {
  readonly registry: ShortcutRegistry;
  readonly children: ReactNode;
}

/**
 * Dispatches key presses on the window to `registry`, and makes the region last clicked or
 * focused (or none) the active shortcut context.
 */
export function ShortcutProvider({ registry, children }: ShortcutProviderProps) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      registry.handleKeyDown(event);
    };
    const onActivate = (event: Event) => {
      registry.setActiveContext(contextOf(event.target));
    };
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('pointerdown', onActivate, true);
    window.addEventListener('focusin', onActivate, true);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('pointerdown', onActivate, true);
      window.removeEventListener('focusin', onActivate, true);
    };
  }, [registry]);
  return <ShortcutRegistryContext value={registry}>{children}</ShortcutRegistryContext>;
}
