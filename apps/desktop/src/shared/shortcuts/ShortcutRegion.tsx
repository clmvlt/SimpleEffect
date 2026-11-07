import { useEffect, type ComponentProps } from 'react';

import type { ShortcutContext } from './shortcut-actions';
import { useShortcutRegistry } from './shortcut-context';
import { SHORTCUT_CONTEXT_ATTRIBUTE } from './ShortcutProvider';

/** Props of {@link ShortcutRegion}. */
export interface ShortcutRegionProps extends ComponentProps<'div'> {
  /** The context made active by clicking or focusing inside the region. */
  readonly context: Exclude<ShortcutContext, 'global'>;
  /** Makes the region active when it mounts if no region is (the editor's default target). */
  readonly claimOnMount?: boolean | undefined;
}

/** A `div` whose clicks and focus make `context` the active shortcut context. */
export function ShortcutRegion({ context, claimOnMount = false, ...props }: ShortcutRegionProps) {
  const registry = useShortcutRegistry();
  useEffect(() => {
    if (claimOnMount && registry.activeContext === null) registry.setActiveContext(context);
  }, [registry, context, claimOnMount]);
  return <div {...{ [SHORTCUT_CONTEXT_ATTRIBUTE]: context }} {...props} />;
}
