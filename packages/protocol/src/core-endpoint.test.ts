import { describe, expect, it } from 'vitest';

import { resolveEndpoint, rpcSocketUrl } from './core-endpoint';

describe('rpcSocketUrl', () => {
  it('targets /rpc over ws with the token as the t parameter', () => {
    expect(rpcSocketUrl({ url: 'http://127.0.0.1:51234', token: 'abc123' })).toBe(
      'ws://127.0.0.1:51234/rpc?t=abc123',
    );
  });

  it('uses wss for an https origin and encodes the token', () => {
    expect(rpcSocketUrl({ url: 'https://localhost:9', token: 'a b' })).toBe(
      'wss://localhost:9/rpc?t=a+b',
    );
  });
});

describe('resolveEndpoint', () => {
  it('accepts a fixed endpoint or a resolver function', async () => {
    const endpoint = { url: 'http://127.0.0.1:1', token: 't' };
    await expect(resolveEndpoint(endpoint)).resolves.toBe(endpoint);
    await expect(resolveEndpoint(() => Promise.resolve(endpoint))).resolves.toBe(endpoint);
  });
});
