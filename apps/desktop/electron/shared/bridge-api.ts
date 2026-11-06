/**
 * Contract between the Electron main process, the preload script and the renderer.
 * Types and constants only: this file is bundled into all three.
 */

/** How the renderer reaches the running se-core process. */
export interface CoreConnection {
  /** HTTP origin of se-core, e.g. `http://127.0.0.1:51234`. The RPC socket is `/rpc` on it. */
  readonly url: string;
  /** Session token to pass as the `t` query parameter. Never log it. */
  readonly token: string;
  /** Version announced by se-core in its handshake. */
  readonly version: string;
}

/** API exposed by the preload script as `window.simpleEffect`. */
export interface SimpleEffectBridge {
  /**
   * Resolves with the connection of the running core, waiting while it (re)starts.
   * Call it again after every disconnect: a restarted core has a new port and token.
   */
  getCoreConnection(): Promise<CoreConnection>;
}

/** IPC channels between preload and main. */
export const IPC_CHANNELS = {
  getCoreConnection: 'core:get-connection',
} as const;
