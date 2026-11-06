import { CoreRequestError } from './core-request-error';
import type { CommandResult, ProtocolError, ResponseEnvelope } from './generated';
import type { CommandKind } from './protocol-types';

interface PendingRequest {
  readonly command: CommandKind;
  readonly frame: string;
  /** Whether the frame was written to a socket (so the core may have executed it). */
  sent: boolean;
  readonly resolve: (result: CommandResult) => void;
  readonly reject: (error: CoreRequestError) => void;
  readonly timer: ReturnType<typeof setTimeout>;
}

/**
 * Requests awaiting their response, keyed by id. Each one either resolves with its result or
 * rejects with a {@link CoreRequestError}, exactly once; its timeout timer always gets cleared.
 */
export class PendingRequests {
  readonly #requests = new Map<number, PendingRequest>();

  /** Registers a request (not sent yet) that fails with `TIMEOUT` after `timeoutMs`. */
  add(id: number, command: CommandKind, frame: string, timeoutMs: number): Promise<CommandResult> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.#fail(id, {
          code: 'TIMEOUT',
          message: `se-core did not answer within ${String(timeoutMs)} ms. It may be busy or stuck: retry, and check the se-core logs if it persists.`,
        });
      }, timeoutMs);
      this.#requests.set(id, { command, frame, sent: false, resolve, reject, timer });
    });
  }

  /** Frames not written to a socket yet, oldest first; they are marked as sent. */
  takeUnsent(): string[] {
    const frames: string[] = [];
    for (const request of this.#requests.values()) {
      if (!request.sent) {
        request.sent = true;
        frames.push(request.frame);
      }
    }
    return frames;
  }

  /** Settles the request answered by `response`; unknown ids (e.g. timed out) are ignored. */
  settle(id: number, response: ResponseEnvelope): void {
    const request = this.#requests.get(id);
    if (request === undefined) {
      return;
    }
    if ('error' in response) {
      this.#fail(id, response.error);
    } else if (response.result.kind !== request.command) {
      this.#fail(id, {
        code: 'INTERNAL',
        message: `se-core answered \`${request.command}\` with a \`${response.result.kind}\` result. The renderer and se-core are out of sync: rebuild both from the same revision.`,
      });
    } else {
      this.#remove(id);
      request.resolve(response.result);
    }
  }

  /** Fails every request already written to a socket (the connection dropped). */
  failSent(error: ProtocolError): void {
    for (const [id, request] of this.#requests) {
      if (request.sent) {
        this.#fail(id, error);
      }
    }
  }

  /** Fails every request, sent or not. */
  failAll(error: ProtocolError): void {
    for (const id of [...this.#requests.keys()]) {
      this.#fail(id, error);
    }
  }

  #fail(id: number, error: ProtocolError): void {
    const request = this.#remove(id);
    request?.reject(new CoreRequestError(request.command, error));
  }

  #remove(id: number): PendingRequest | undefined {
    const request = this.#requests.get(id);
    if (request !== undefined) {
      clearTimeout(request.timer);
      this.#requests.delete(id);
    }
    return request;
  }
}
