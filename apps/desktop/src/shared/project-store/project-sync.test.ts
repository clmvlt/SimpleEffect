import {
  CoreRequestError,
  type Command,
  type ConnectionState,
  type CoreEvent,
  type CoreEventKind,
  type ErrorCode,
} from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import { createProjectStore } from './project-store';
import { connectProjectSync, type ProjectSyncClient } from './project-sync';
import { IDS, emptyPatch, session } from './testing/session-fixture';

type Responder = (command: Command) => Promise<unknown>;

/** A core client whose events and answers the test controls. */
class FakeCoreClient {
  state: ConnectionState = 'open';
  readonly sent: Command[] = [];
  respond: Responder = () => Promise.resolve({});
  readonly #handlers = new Map<CoreEventKind, Set<(event: CoreEvent) => void>>();
  readonly #stateListeners = new Set<(state: ConnectionState) => void>();

  readonly client: ProjectSyncClient = {
    state: 'open',
    // The fake answers whatever the test's responder returns for the command.
    send: ((command: Command) => {
      this.sent.push(command);
      return this.respond(command);
    }) as ProjectSyncClient['send'],
    on: ((kind: CoreEventKind, handler: (event: CoreEvent) => void) => {
      const handlers = this.#handlers.get(kind) ?? new Set();
      handlers.add(handler);
      this.#handlers.set(kind, handlers);
      return () => handlers.delete(handler);
    }) as ProjectSyncClient['on'],
    onStateChange: (listener) => {
      this.#stateListeners.add(listener);
      return () => this.#stateListeners.delete(listener);
    },
  };

  emit(event: CoreEvent): void {
    for (const handler of this.#handlers.get(event.kind) ?? []) handler(event);
  }

  changeState(state: ConnectionState): void {
    this.state = state;
    for (const listener of this.#stateListeners) listener(state);
  }
}

function setUp() {
  const fake = new FakeCoreClient();
  const store = createProjectStore();
  const sync = connectProjectSync(fake.client, store, { warn: vi.fn() });
  sync.load(session(3));
  return { fake, store, sync };
}

function error(code: ErrorCode): CoreRequestError {
  return new CoreRequestError('getProjectSnapshot', { code, message: code });
}

describe('connectProjectSync', () => {
  it('applies patches in order and follows the history', () => {
    const { fake, store } = setUp();
    fake.emit({ kind: 'projectPatched', ...emptyPatch(4) });
    fake.emit({
      kind: 'historyChanged',
      projectId: IDS.project,
      history: { canUndo: true, canRedo: false, undoLabel: 'Move', redoLabel: null },
    });
    const mirror = store.getState().mirror;
    expect(mirror?.revision).toBe(4);
    expect(mirror?.history.undoLabel).toBe('Move');
    expect(fake.sent).toEqual([]);
  });

  it('reloads the snapshot when a patch is missing', async () => {
    const { fake, store } = setUp();
    fake.respond = () => Promise.resolve({ kind: 'getProjectSnapshot', ...session(9) });
    fake.emit({ kind: 'projectPatched', ...emptyPatch(6) });
    await vi.waitFor(() => {
      expect(store.getState().mirror?.revision).toBe(9);
    });
    expect(fake.sent).toEqual([{ kind: 'getProjectSnapshot', projectId: IDS.project }]);
  });

  it('reloads after a reconnection and reopens the project if the core restarted', async () => {
    const { fake, store } = setUp();
    fake.respond = (command) =>
      command.kind === 'getProjectSnapshot'
        ? Promise.reject(error('PROJECT_NOT_OPEN'))
        : Promise.resolve({ kind: 'openProject', ...session(5) });
    fake.changeState('reconnecting');
    fake.changeState('open');
    await vi.waitFor(() => {
      expect(store.getState().mirror?.revision).toBe(5);
    });
    expect(fake.sent.map((command) => command.kind)).toEqual(['getProjectSnapshot', 'openProject']);
  });

  it('marks the project closed when it cannot be reopened', async () => {
    const { fake, store, sync } = setUp();
    fake.respond = () => Promise.reject(error('PROJECT_NOT_OPEN'));
    await sync.resync();
    expect(store.getState()).toEqual({
      mirror: null,
      closedProjectId: IDS.project,
      saveState: null,
    });
  });

  it('clears the mirror when the core closes the project', () => {
    const { fake, store } = setUp();
    fake.emit({ kind: 'projectClosed', projectId: 'another' });
    expect(store.getState().mirror).not.toBeNull();
    fake.emit({ kind: 'projectClosed', projectId: IDS.project });
    expect(store.getState()).toEqual({
      mirror: null,
      closedProjectId: IDS.project,
      saveState: null,
    });
  });

  it('follows the save state of the loaded project and forgets it with the project', () => {
    const { fake, store, sync } = setUp();
    const failing = { projectId: IDS.project, failure: 'The disk is full.', unsavedChanges: 4 };
    fake.emit({
      kind: 'projectSaveStateChanged',
      projectId: 'another',
      failure: 'x',
      unsavedChanges: 1,
    });
    expect(store.getState().saveState).toBeNull();
    fake.emit({ kind: 'projectSaveStateChanged', ...failing });
    expect(store.getState().saveState).toEqual(failing);
    fake.emit({ kind: 'projectPatched', ...emptyPatch(4) });
    expect(store.getState().saveState).toEqual(failing);
    sync.forget();
    expect(store.getState().saveState).toBeNull();
  });

  it('keeps the recovery report of the session in the mirror', () => {
    const { store, sync } = setUp();
    expect(store.getState().mirror?.recovery).toBeNull();
    const recovery = { replayedChanges: 3, restoredFrom: null, problem: null, lostChanges: 0 };
    sync.load({ ...session(5), recovery });
    expect(store.getState().mirror?.recovery).toEqual(recovery);
  });

  it('stops listening once disposed', () => {
    const { fake, store, sync } = setUp();
    sync.dispose();
    fake.emit({ kind: 'projectPatched', ...emptyPatch(4) });
    expect(store.getState().mirror?.revision).toBe(3);
  });
});
