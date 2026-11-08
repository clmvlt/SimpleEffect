import type {
  ItemId,
  ProjectId,
  SubmitTaskRequest,
  TaskId,
  TaskRevertReport,
  TaskScope,
} from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import {
  cancelTask,
  followUpTask,
  getTaskChanges,
  previewTaskRevert,
  retryTask,
  revertTask,
  submitTask,
} from '../api';
import { revertOutcome } from '../services/task-log';

/** Task commands; failures are shown as translated toasts and resolve with `null`. */
export interface TaskCommands {
  /** Resolves with the new task's id. */
  readonly submit: (request: SubmitTaskRequest) => Promise<TaskId | null>;
  readonly cancel: (projectId: ProjectId, taskId: TaskId) => void;
  readonly retry: (projectId: ProjectId, taskId: TaskId) => void;
  /** Resumes the task's session with `prompt` (and a new scope when given). */
  readonly followUp: (
    projectId: ProjectId,
    taskId: TaskId,
    prompt: string,
    scope?: TaskScope,
  ) => Promise<TaskId | null>;
  /** What reverting the task would do now. */
  readonly previewRevert: (
    projectId: ProjectId,
    taskId: TaskId,
  ) => Promise<TaskRevertReport | null>;
  /** Reverts a finished task; a toast says what was undone and what was kept. */
  readonly revert: (projectId: ProjectId, taskId: TaskId) => Promise<TaskRevertReport | null>;
  /** The items the task added or changed that still exist. */
  readonly changes: (projectId: ProjectId, taskId: TaskId) => Promise<readonly ItemId[] | null>;
}

/** Sends task commands to the core. */
export function useTaskCommands(): TaskCommands {
  const client = useCoreClient();
  const { t } = useTranslation();
  const fail = (error: unknown): null => {
    console.warn('[ai] task command failed', error);
    toast.error(t(errorMessageKey(error)));
    return null;
  };
  const announceRevert = (report: TaskRevertReport) => {
    const outcome = revertOutcome(report);
    switch (outcome.kind) {
      case 'reverted':
        toast.success(t('ai.revert.done'));
        break;
      case 'partial':
        toast.warning(t('ai.revert.partial', { count: outcome.kept }));
        break;
      case 'nothing':
        toast.warning(t('ai.revert.nothing', { count: outcome.kept }));
        break;
    }
    return report;
  };
  return {
    submit: (request) => submitTask(client, request).then((task) => task.id, fail),
    cancel: (projectId, taskId) => {
      cancelTask(client, projectId, taskId).catch(fail);
    },
    retry: (projectId, taskId) => {
      retryTask(client, projectId, taskId).catch(fail);
    },
    followUp: (projectId, taskId, prompt, scope) =>
      followUpTask(client, projectId, taskId, prompt, scope).then((task) => task.id, fail),
    previewRevert: (projectId, taskId) => previewTaskRevert(client, projectId, taskId).catch(fail),
    revert: (projectId, taskId) => revertTask(client, projectId, taskId).then(announceRevert, fail),
    changes: (projectId, taskId) =>
      getTaskChanges(client, projectId, taskId).then((items) => items, fail),
  };
}
