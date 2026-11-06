import { describe, expect, it } from 'vitest';

import { encodePingRequest, parsePongVersion, rpcSocketUrl } from './rpc-messages';

describe('rpcSocketUrl', () => {
  it('targets /rpc over ws with the token as the t parameter', () => {
    const url = rpcSocketUrl({ url: 'http://127.0.0.1:51234', token: 'abc123', version: '0.1.0' });
    expect(url).toBe('ws://127.0.0.1:51234/rpc?t=abc123');
  });
});

describe('encodePingRequest', () => {
  it('matches the documented request shape', () => {
    expect(JSON.parse(encodePingRequest(1))).toEqual({
      type: 'request',
      id: 1,
      command: { kind: 'ping' },
    });
  });
});

describe('parsePongVersion', () => {
  const pong = '{"type":"response","id":1,"result":{"kind":"pong","version":"0.1.0"}}';

  it('extracts the version from the matching pong', () => {
    expect(parsePongVersion(pong, 1)).toBe('0.1.0');
  });

  it('ignores responses to other requests, errors and non-JSON frames', () => {
    expect(parsePongVersion(pong, 2)).toBeNull();
    expect(
      parsePongVersion('{"type":"response","id":1,"error":{"code":"INVALID_REQUEST"}}', 1),
    ).toBeNull();
    expect(parsePongVersion('not json', 1)).toBeNull();
    expect(parsePongVersion(new ArrayBuffer(4), 1)).toBeNull();
  });
});
