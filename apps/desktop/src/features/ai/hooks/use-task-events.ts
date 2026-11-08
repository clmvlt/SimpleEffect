import type { TaskEventRecord } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { getTaskEvents } from '../api';
import { appendEvent } from '../services/task-log';

/** Events loaded for one task: events of another task are not shown. */
interface Keyed {
  readonly key: string | null;
  readonly items: readonly TaskEventRecord[];
}

const NOTHING: readonly never[] = [];

/**
 * The event log of a task, kept up to date by `taskEventAppended` events (`null` loads nothing:
 * a collapsed card).
 */
export function useTaskEvents(
  projectId: string | null,
  taskId: string | null,
): readonly TaskEventRecord[] {
  const client = useCoreClient();
  const [state, setState] = useState<Keyed>({ key: null, items: NOTHING });
  useEffect(() => {
    if (projectId === null || taskId === null) return;
    let current = true;
    getTaskEvents(client, projectId, taskId).then(
      (log) => {
        if (!current) return;
        setState((known) => ({
          key: taskId,
          items: log.reduce(appendEvent, known.key === taskId ? [...known.items] : []),
        }));
      },
      (error: unknown) => {
        console.warn('[ai] reading task events failed', error);
      },
    );
    return () => {
      current = false;
    };
  }, [client, projectId, taskId]);
  useCoreEvent('taskEventAppended', (appended) => {
    if (appended.taskId !== taskId) return;
    setState((known) => ({
      key: taskId,
      items: appendEvent(known.key === taskId ? known.items : NOTHING, appended.event),
    }));
  });
  return state.key === taskId ? state.items : NOTHING;
}
