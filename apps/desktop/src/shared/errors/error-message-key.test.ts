import { CoreRequestError } from '@se/protocol';
import { describe, expect, expectTypeOf, it } from 'vitest';

import fr from '../../i18n/locales/fr.json';
import { ERROR_CODES, errorMessageKey, type ErrorCodesCovered } from './error-message-key';

describe('errorMessageKey', () => {
  it('lists every protocol error code', () => {
    expectTypeOf<ErrorCodesCovered>().toEqualTypeOf<true>();
  });

  it('has a French message for every code and for unknown failures', () => {
    const messages: Record<string, string> = fr.errors;
    for (const code of [...ERROR_CODES, 'unknown']) {
      expect(messages[code], code).toBeTruthy();
    }
  });

  it('maps core errors by code and anything else to the generic message', () => {
    const locked = new CoreRequestError('openProject', { code: 'PROJECT_LOCKED', message: '…' });
    expect(errorMessageKey(locked)).toBe('errors.PROJECT_LOCKED');
    expect(errorMessageKey(new Error('boom'))).toBe('errors.unknown');
  });
});
