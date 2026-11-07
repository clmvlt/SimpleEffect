import { useEffect } from 'react';

import { uiPreferencesStorage, type KeyValueStorage } from '../../../shared/storage';
import {
  connectGraphicsPersistence,
  useGraphicsPreferences,
  type GraphicsPreferences,
} from '../store/graphics-preferences-store';

/** The graphics preferences, restored from this machine's UI preferences and saved on change. */
export function usePersistedGraphicsPreferences(
  storage: () => KeyValueStorage = uiPreferencesStorage,
): GraphicsPreferences {
  useEffect(() => connectGraphicsPersistence(storage()), [storage]);
  return useGraphicsPreferences();
}
