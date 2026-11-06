/** Where a se-core process listens, as announced by its handshake. */
export interface CoreEndpoint {
  /** HTTP origin of se-core, e.g. `http://127.0.0.1:51234`. */
  readonly url: string;
  /** Session token, sent as the `t` query parameter. Never log it. */
  readonly token: string;
}

/**
 * Either a fixed endpoint or a function resolving the current one. Use a function when the core
 * can restart with a new port and token (the Electron app): it is called before every attempt.
 */
export type CoreEndpointSource = CoreEndpoint | (() => Promise<CoreEndpoint>);

/** WebSocket URL of the core's `/rpc` endpoint, carrying the session token. */
export function rpcSocketUrl(endpoint: CoreEndpoint): string {
  const url = new URL('/rpc', endpoint.url);
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  url.searchParams.set('t', endpoint.token);
  return url.toString();
}

/** Resolves a {@link CoreEndpointSource} to the endpoint to use for the next attempt. */
export function resolveEndpoint(source: CoreEndpointSource): Promise<CoreEndpoint> {
  return typeof source === 'function' ? source() : Promise.resolve(source);
}
