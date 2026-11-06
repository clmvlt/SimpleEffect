import type { CoreClient } from '@se/protocol';
import { createContext } from 'react';

/** The app's {@link CoreClient}, provided once at the root by `CoreClientProvider`. */
export const CoreClientContext = createContext<CoreClient | null>(null);
