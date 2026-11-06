import { describe, expect, it } from 'vitest';

import { parseHandshakeLine } from './core-handshake';

const TOKEN = 'a1'.repeat(32);

function line(fields: Record<string, unknown>): string {
  return JSON.stringify(fields);
}

describe('parseHandshakeLine', () => {
  it('parses the line printed by se-core', () => {
    expect(parseHandshakeLine(line({ port: 51234, token: TOKEN, version: '0.1.0' }))).toEqual({
      ok: true,
      handshake: { port: 51234, token: TOKEN, version: '0.1.0' },
    });
  });

  it('tolerates a trailing carriage return', () => {
    const result = parseHandshakeLine(`${line({ port: 1, token: TOKEN, version: '0.1.0' })}\r`);
    expect(result.ok).toBe(true);
  });

  it('rejects text that is not JSON', () => {
    const result = parseHandshakeLine('Compiling se-core...');
    expect(result).toEqual({ ok: false, error: expect.stringContaining('not JSON') as unknown });
  });

  it.each([
    ['a JSON array', '[1,2,3]'],
    ['a missing port', line({ token: TOKEN, version: '0.1.0' })],
    ['a fractional port', line({ port: 1.5, token: TOKEN, version: '0.1.0' })],
    ['an out-of-range port', line({ port: 70_000, token: TOKEN, version: '0.1.0' })],
    ['a short token', line({ port: 1, token: 'abc', version: '0.1.0' })],
    ['an uppercase token', line({ port: 1, token: TOKEN.toUpperCase(), version: '0.1.0' })],
    ['an empty version', line({ port: 1, token: TOKEN, version: '' })],
  ])('rejects %s', (_case, input) => {
    expect(parseHandshakeLine(input).ok).toBe(false);
  });

  it('never echoes the token in error messages', () => {
    const result = parseHandshakeLine(line({ port: 0, token: TOKEN, version: '0.1.0' }));
    expect(result.ok).toBe(false);
    expect(JSON.stringify(result)).not.toContain(TOKEN);
  });
});
