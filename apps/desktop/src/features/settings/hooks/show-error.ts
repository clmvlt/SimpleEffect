import { toast } from '@se/ui';
import type { TFunction } from 'i18next';

import { errorMessageKey } from '../../../shared/errors';

/** Logs `error` and shows its translated message in a toast. */
export function showError(t: TFunction, error: unknown): void {
  console.warn('[settings]', error);
  toast.error(t(errorMessageKey(error)));
}
