import { CoreRequestError, type CoreClient, type ProjectSession } from '@se/protocol';

import { applyPatch, mirrorFromSession, type ProjectMirror } from './project-mirror';
import type { ProjectStore } from './project-store';

/** What the sync needs from the core client. */
export type ProjectSyncClient = Pick<CoreClient, 'send' | 'on' | 'onStateChange' | 'state'>;

/** Keeps the project store in step with the core. */
export interface ProjectSync {
  /** Shows `session` (the editor loads it after opening a project). */
  load(session: ProjectSession): void;
  /** Forgets the loaded project (before closing it). */
  forget(): void;
  /** Reloads the loaded project's snapshot from the core. */
  resync(): Promise<void>;
  /** Stops listening to the core. */
  dispose(): void;
}

/**
 * Feeds `store` from core events: applies `projectPatched` in revision order, follows
 * `historyChanged`, `projectSaveStateChanged` and `projectClosed`, and reloads the snapshot when a patch is missing, when
 * the core says events were dropped (`resyncRequired`) and after a reconnection. If the core
 * restarted meanwhile (`PROJECT_NOT_OPEN`), the project is opened again from its folder.
 */
export function connectProjectSync(
  client: ProjectSyncClient,
  store: ProjectStore,
  logger: Pick<Console, 'warn'> = console,
): ProjectSync {
  let inFlight: Promise<void> | null = null;
  let resyncAgain = false;
  let previousState = client.state;

  const current = () => store.getState().mirror;
  const show = (projectId: string, session: ProjectSession) => {
    if (current()?.projectId === projectId) {
      store.setState({ mirror: mirrorFromSession(session), closedProjectId: null });
    }
  };

  async function reload(mirror: ProjectMirror): Promise<void> {
    try {
      show(
        mirror.projectId,
        await client.send({ kind: 'getProjectSnapshot', projectId: mirror.projectId }),
      );
    } catch (error) {
      if (!(error instanceof CoreRequestError && error.code === 'PROJECT_NOT_OPEN')) {
        logger.warn(`[project-sync] cannot reload the project: ${String(error)}`);
        return;
      }
      try {
        show(mirror.projectId, await client.send({ kind: 'openProject', path: mirror.path }));
      } catch (openError) {
        logger.warn(`[project-sync] cannot reopen the project: ${String(openError)}`);
        if (current()?.projectId === mirror.projectId) {
          store.setState({ mirror: null, closedProjectId: mirror.projectId });
        }
      }
    }
  }

  function resync(): Promise<void> {
    if (inFlight !== null) {
      resyncAgain = true;
      return inFlight;
    }
    const mirror = current();
    if (mirror === null) {
      return Promise.resolve();
    }
    inFlight = reload(mirror).finally(() => {
      inFlight = null;
      if (resyncAgain) {
        resyncAgain = false;
        void resync();
      }
    });
    return inFlight;
  }

  const unsubscribers = [
    client.on('projectPatched', (patch) => {
      const mirror = current();
      if (mirror === null) return;
      const outcome = applyPatch(mirror, patch);
      if (outcome.kind === 'applied') {
        store.setState({ mirror: outcome.mirror });
      } else if (outcome.kind === 'gap') {
        void resync();
      }
    }),
    client.on('projectSaveStateChanged', ({ projectId, failure, unsavedChanges }) => {
      if (current()?.projectId === projectId) {
        store.setState({ saveState: { projectId, failure, unsavedChanges } });
      }
    }),
    client.on('historyChanged', ({ projectId, history }) => {
      const mirror = current();
      if (mirror?.projectId === projectId) {
        store.setState({ mirror: { ...mirror, history } });
      }
    }),
    client.on('projectClosed', ({ projectId }) => {
      if (current()?.projectId === projectId) {
        store.setState({ mirror: null, closedProjectId: projectId });
      }
    }),
    client.on('projectOpened', (session) => {
      const mirror = current();
      if (mirror !== null && session.snapshot.revision > mirror.revision) {
        show(session.snapshot.project.id, session);
      }
    }),
    client.on('resyncRequired', () => {
      void resync();
    }),
    client.onStateChange((state) => {
      if (state === 'open' && previousState === 'reconnecting') {
        void resync();
      }
      previousState = state;
    }),
  ];

  return {
    load(session) {
      store.setState({
        mirror: mirrorFromSession(session),
        closedProjectId: null,
        saveState: null,
      });
    },
    forget() {
      store.setState({ mirror: null, closedProjectId: null, saveState: null });
    },
    resync,
    dispose() {
      for (const unsubscribe of unsubscribers) unsubscribe();
    },
  };
}
