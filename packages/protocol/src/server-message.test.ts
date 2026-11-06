import { describe, expect, it } from 'vitest';

import { parseServerMessage } from './server-message';

describe('parseServerMessage', () => {
  it('accepts responses with a result or an error, including a null id', () => {
    expect(parseServerMessage('{"type":"response","id":1,"result":{"kind":"ping"}}')).toEqual({
      type: 'response',
      id: 1,
      result: { kind: 'ping' },
    });
    expect(
      parseServerMessage(
        '{"type":"response","id":null,"error":{"code":"INVALID_REQUEST","message":"bad"}}',
      ),
    ).toMatchObject({ id: null, error: { code: 'INVALID_REQUEST' } });
  });

  it('accepts events', () => {
    expect(parseServerMessage('{"type":"event","event":{"kind":"coreShuttingDown"}}')).toEqual({
      type: 'event',
      event: { kind: 'coreShuttingDown' },
    });
  });

  it('rejects anything else', () => {
    for (const frame of [
      'not json',
      '[]',
      '{"type":"response","id":"1","result":{"kind":"ping"}}',
      '{"type":"response","id":1}',
      '{"type":"event","event":{}}',
      '{"type":"request","id":1,"command":{"kind":"ping"}}',
      new ArrayBuffer(2),
    ]) {
      expect(parseServerMessage(frame)).toBeNull();
    }
  });
});
