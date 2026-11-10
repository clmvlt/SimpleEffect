import { useEffect } from 'react';

import type { KeyValueStorage } from '../../../shared/storage';
import { isFirstRun } from '../services/onboarding-steps';
import { openOnboarding } from '../store/onboarding-store';

/** Opens the wizard once at startup when it was never finished or skipped on this machine. */
export function useFirstRun(preferences: KeyValueStorage): void {
  useEffect(() => {
    if (isFirstRun(preferences)) openOnboarding();
  }, [preferences]);
}
