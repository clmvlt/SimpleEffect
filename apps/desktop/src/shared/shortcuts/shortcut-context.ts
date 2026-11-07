import { createContext } from 'react';

import type { ShortcutRegistry } from './shortcut-registry';

/** The app's shortcut registry; `null` outside `ShortcutProvider`. */
export const ShortcutContext = createContext<ShortcutRegistry | null>(null);
