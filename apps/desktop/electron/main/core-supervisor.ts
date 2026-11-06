import { spawn, type ChildProcessByStdio } from 'node:child_process';
import { createInterface } from 'node:readline';
import type { Readable, Writable } from 'node:stream';

import type { CoreHandshake } from '@se/protocol';

import type { CoreConnection } from '../shared/bridge-api';
import { coreEnvironment } from './core-environment';
import { parseHandshakeLine } from './core-handshake';
import { restartDelayMs } from './restart-backoff';

type CoreProcess = ChildProcessByStdio<Writable, Readable, Readable>;

/** Where supervisor messages go (the console by default). */
export type SupervisorLogger = Pick<Console, 'info' | 'warn' | 'error'>;

/** Options of {@link CoreSupervisor}. */
export interface CoreSupervisorOptions {
  readonly binaryPath: string;
  /** App data folder passed to se-core as `SE_DATA_DIR` (its logs go to `<dataDir>/logs`). */
  readonly dataDir: string;
  readonly logger?: SupervisorLogger;
}

/** A core that has not printed its handshake by then is considered stuck and killed. */
const HANDSHAKE_TIMEOUT_MS = 15_000;
/** Time given to se-core to exit after its stdin closes, before it is killed. */
const STOP_GRACE_PERIOD_MS = 3_000;
/** A core that ran this long before exiting resets the restart backoff. */
const STABLE_RUN_MS = 10_000;

/**
 * Spawns se-core, reads its handshake, restarts it with exponential backoff when it exits
 * unexpectedly and stops it when the app quits.
 *
 * The core's stdin stays open for its whole life and is never written to: closing it (or this
 * process dying) is what tells se-core to shut down, so no orphan core survives the app.
 */
export class CoreSupervisor {
  readonly #binaryPath: string;
  readonly #dataDir: string;
  readonly #logger: SupervisorLogger;
  #process: CoreProcess | null = null;
  #connection: CoreConnection | null = null;
  #waiting: ((connection: CoreConnection) => void)[] = [];
  #consecutiveFailures = 0;
  #readySince: number | null = null;
  #restartTimer: NodeJS.Timeout | undefined;
  #stopping = false;

  constructor({ binaryPath, dataDir, logger = console }: CoreSupervisorOptions) {
    this.#binaryPath = binaryPath;
    this.#dataDir = dataDir;
    this.#logger = logger;
  }

  /** Launches the core. Calling it again while a core is running does nothing. */
  start(): void {
    if (this.#process === null && !this.#stopping) {
      this.#launch();
    }
  }

  /** Resolves with the current connection, or with the next one once the core is ready. */
  waitForConnection(): Promise<CoreConnection> {
    const connection = this.#connection;
    if (connection !== null) {
      return Promise.resolve(connection);
    }
    return new Promise((resolve) => {
      this.#waiting.push(resolve);
    });
  }

  /** Stops the core for good: closes its stdin, then kills it if it does not exit in time. */
  async stop(): Promise<void> {
    this.#stopping = true;
    clearTimeout(this.#restartTimer);
    const running = this.#process;
    if (running !== null) {
      await stopGracefully(running, STOP_GRACE_PERIOD_MS);
    }
  }

  #launch(): void {
    const child = spawn(this.#binaryPath, [], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: coreEnvironment(this.#dataDir, process.env),
      windowsHide: true,
    });
    this.#process = child;
    // Nothing is ever written to stdin; ignore EPIPE if the core dies before the pipe closes.
    child.stdin.on('error', () => undefined);

    const handshakeTimer = setTimeout(() => {
      this.#logger.error('[se-core] no handshake received in time; killing the process');
      child.kill();
    }, HANDSHAKE_TIMEOUT_MS);
    this.#readHandshake(child, handshakeTimer);
    forwardLines(child.stderr, (line) => {
      this.#logger.info(`[se-core] ${line}`);
    });

    child.once('error', (error) => {
      clearTimeout(handshakeTimer);
      this.#logger.error(`[se-core] cannot run "${this.#binaryPath}": ${error.message}`);
      this.#handleExit(child);
    });
    child.once('exit', (code, signal) => {
      clearTimeout(handshakeTimer);
      this.#logger.warn(`[se-core] exited (code: ${String(code)}, signal: ${String(signal)})`);
      this.#handleExit(child);
    });
  }

  #readHandshake(child: CoreProcess, handshakeTimer: NodeJS.Timeout): void {
    let handshakeSeen = false;
    forwardLines(child.stdout, (line) => {
      if (handshakeSeen) {
        this.#logger.warn('[se-core] ignored unexpected stdout output after the handshake');
        return;
      }
      handshakeSeen = true;
      clearTimeout(handshakeTimer);
      this.#acceptHandshake(child, line);
    });
  }

  #acceptHandshake(child: CoreProcess, line: string): void {
    const result = parseHandshakeLine(line);
    if (!result.ok) {
      this.#logger.error(`[se-core] ${result.error}`);
      child.kill();
      return;
    }
    const connection = toConnection(result.handshake);
    this.#connection = connection;
    this.#readySince = Date.now();
    this.#logger.info(`[se-core] version ${connection.version} ready on ${connection.url}`);
    for (const resolve of this.#waiting.splice(0)) {
      resolve(connection);
    }
  }

  #handleExit(child: CoreProcess): void {
    // 'error' and 'exit' can both fire for the same process: handle it once.
    if (this.#process !== child) {
      return;
    }
    this.#process = null;
    this.#connection = null;
    if (!this.#stopping) {
      this.#scheduleRestart();
    }
  }

  #scheduleRestart(): void {
    const ranStably = this.#readySince !== null && Date.now() - this.#readySince >= STABLE_RUN_MS;
    if (ranStably) {
      this.#consecutiveFailures = 0;
    }
    this.#readySince = null;
    const delay = restartDelayMs(this.#consecutiveFailures);
    this.#consecutiveFailures += 1;
    this.#logger.warn(`[se-core] restarting in ${String(delay)} ms`);
    this.#restartTimer = setTimeout(() => {
      this.#launch();
    }, delay);
  }
}

function toConnection({ port, token, version }: CoreHandshake): CoreConnection {
  return { url: `http://127.0.0.1:${String(port)}`, token, version };
}

function forwardLines(stream: Readable, onLine: (line: string) => void): void {
  createInterface({ input: stream, crlfDelay: Infinity }).on('line', onLine);
}

function stopGracefully(child: CoreProcess, gracePeriodMs: number): Promise<void> {
  return new Promise((resolve) => {
    if (child.exitCode !== null || child.signalCode !== null) {
      resolve();
      return;
    }
    const killTimer = setTimeout(() => child.kill(), gracePeriodMs);
    child.once('exit', () => {
      clearTimeout(killTimer);
      resolve();
    });
    // se-core shuts itself down when its stdin reaches end-of-file.
    child.stdin.end();
  });
}
