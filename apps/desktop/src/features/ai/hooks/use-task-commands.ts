import type { SubmitTaskRequest } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { cancelTask, followUpTask, retryTask, submitTask } from '../api';

/** Task commands; failures are shown as translated toasts. */
export interface TaskCommands {
  /** Resolves with the new task's id, or `null` when refused. */
  readonly submit: (request: SubmitTaskRequest) => Promise<string | null>;
  readonly cancel: (projectId: string, taskId: string) => void;
  readonly retry: (projectId: string, taskId: string) => void;
  readonly followUp: (projectId: string, taskId: string, prompt: string) => void;
}

/** Sends task commands to the core. */
export function useTaskCommands(): TaskCommands {
  const client = useCoreClient();
  const { t } = useTranslation();
  const fail = (error: unknown) => {
    console.warn('[ai] task command failed', error);
    toast.error(t(errorMessageKey(error)));
  };
  return {
    submit: (request) =>
      submitTask(client, request).then(
        (task) => task.id,
        (error: unknown) => {
          fail(error);
          return null;
        },
      ),
    cancel: (projectId, taskId) => {
      cancelTask(client, projectId, taskId).catch(fail);
    },
    retry: (projectId, taskId) => {
      retryTask(client, projectId, taskId).catch(fail);
    },
    followUp: (projectId, taskId, prompt) => {
      followUpTask(client, projectId, taskId, prompt).catch(fail);
    },
  };
}
