import { useEffect } from 'react';
import { useStore } from 'zustand';

import { uiPreferencesStorage, type KeyValueStorage } from '../../../shared/storage';
import {
  connectViewerPersistence,
  viewerStore,
  type ViewerPreferences,
} from '../store/viewer-store';

/** The viewer preferences, restored from this machine's UI preferences and saved on change. */
export function useViewerPreferences(
  storage: () => KeyValueStorage = uiPreferencesStorage,
): ViewerPreferences {
  useEffect(() => connectViewerPersistence(storage()), [storage]);
  return useStore(viewerStore);
}
