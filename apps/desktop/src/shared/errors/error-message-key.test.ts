import { CoreRequestError } from '@se/protocol';
import { describe, expect, expectTypeOf, it } from 'vitest';

import fr from '../../i18n/locales/fr.json';
import {
  EDIT_DETAIL_CODES,
  ERROR_CODES,
  errorMessageKey,
  type ErrorCodesCovered,
} from './error-message-key';

describe('errorMessageKey', () => {
  it('lists every protocol error code', () => {
    expectTypeOf<ErrorCodesCovered>().toEqualTypeOf<true>();
  });

  it('has a French message for every code and for unknown failures', () => {
    const messages: Record<string, unknown> = fr.errors;
    for (const code of [...ERROR_CODES, 'unknown']) {
      expect(messages[code], code).toBeTruthy();
    }
  });

  it('has a French message for every refused edit with its own message', () => {
    const messages: Record<string, string> = fr.errors.edit;
    for (const code of EDIT_DETAIL_CODES) expect(messages[code], code).toBeTruthy();
  });

  it('gives common refused edits their own message', () => {
    const locked = new CoreRequestError('moveItems', {
      code: 'EDIT_REJECTED',
      message: '…',
      details: { domainCode: 'TRACK_LOCKED' },
    });
    expect(errorMessageKey(locked)).toBe('errors.edit.TRACK_LOCKED');
    const other = new CoreRequestError('moveItems', {
      code: 'EDIT_REJECTED',
      message: '…',
      details: { domainCode: 'INVALID_DOCUMENT' },
    });
    expect(errorMessageKey(other)).toBe('errors.EDIT_REJECTED');
  });

  it('maps core errors by code and anything else to the generic message', () => {
    const locked = new CoreRequestError('openProject', { code: 'PROJECT_LOCKED', message: '…' });
    expect(errorMessageKey(locked)).toBe('errors.PROJECT_LOCKED');
    expect(errorMessageKey(new Error('boom'))).toBe('errors.unknown');
  });
});
