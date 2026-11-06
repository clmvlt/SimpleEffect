/**
 * `@se/protocol`: TypeScript side of the se-core protocol.
 *
 * - Every type generated from the Rust crates by `pnpm codegen` (`src/generated/`, never
 *   hand-edited): `Command`, `CommandResult`, `CoreEvent`, `ErrorCode`, `CoreInfo`…
 * - Types derived from them (`ResultFor`, `CoreEventOf`…).
 * - `CoreClient`, the typed WebSocket client of se-core.
 */
export type * from './generated';
export { CoreClient } from './core-client';
export type { ConnectionState, CoreClientOptions, SendOptions } from './core-client';
export { rpcSocketUrl } from './core-endpoint';
export type { CoreEndpoint, CoreEndpointSource } from './core-endpoint';
export { CoreRequestError } from './core-request-error';
export type {
  CommandKind,
  CommandOf,
  CommandsWithoutResult,
  CoreEventKind,
  CoreEventOf,
  ResultFor,
} from './protocol-types';
export type { ReconnectDelayOptions } from './reconnect-delay';
export type { WebSocketFactory, WebSocketLike } from './web-socket-like';
