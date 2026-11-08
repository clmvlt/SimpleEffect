import type { Task, TaskEventRecord } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { getTaskEvents, listTasks } from '../api';
import { appendEvent, upsertTask } from '../services/task-log';

/** Items loaded for one key (a project, a task): items of another key are not shown. */
interface Keyed<T> {
  readonly key: string | null;
  readonly items: readonly T[];
}

const NOTHING: readonly never[] = [];

/** The tasks of `projectId`, kept up to date by `taskUpdated` events. */
export function useProjectTasks(projectId: string | null): readonly Task[] {
  const client = useCoreClient();
  const [state, setState] = useState<Keyed<Task>>({ key: null, items: NOTHING });
  useEffect(() => {
    if (projectId === null) return;
    let current = true;
    listTasks(client, projectId).then(
      (list) => {
        if (!current) return;
        setState((known) => ({
          key: projectId,
          items: list.reduce(upsertTask, known.key === projectId ? [...known.items] : []),
        }));
      },
      (error: unknown) => {
        console.warn('[ai] listing tasks failed', error);
      },
    );
    return () => {
      current = false;
    };
  }, [client, projectId]);
  useCoreEvent('taskUpdated', ({ task }) => {
    if (task.projectId !== projectId) return;
    setState((known) => ({
      key: projectId,
      items: upsertTask(known.key === projectId ? known.items : NOTHING, task),
    }));
  });
  return state.key === projectId ? state.items : NOTHING;
}

/** The event log of a task, kept up to date by `taskEventAppended` events. */
export function useTaskEvents(
  projectId: string | null,
  taskId: string | null,
): readonly TaskEventRecord[] {
  const client = useCoreClient();
  const [state, setState] = useState<Keyed<TaskEventRecord>>({ key: null, items: NOTHING });
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
