import type { ItemEntity, Task } from '@se/protocol';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { isTaskActive, showTask, useAiTasks } from '../../../shared/ai-tasks';
import { useProjectSelector } from '../../../shared/project-store';
import { continuableTask } from '../services/composer-notices';
import { itemLabel } from '../services/item-label';
import { widenedScope } from '../services/task-scope';
import {
  clearAfterSubmit,
  composerStore,
  updateComposer,
  useComposer,
} from '../store/composer-store';
import type { ComposerScope } from './use-composer-scope';
import { useNow } from './use-now';
import { useTaskCommands } from './use-task-commands';

/** How often the « continue the latest task » default is checked against the clock. */
const CONTINUE_CLOCK_MS = 60_000;

/** The task whose session the next prompt may resume, and why. */
export interface ResumeOption {
  readonly task: Task;
  /** `redo`: the task that made the item « Refaire avec l'IA » names; `latest`: the last task. */
  readonly kind: 'redo' | 'latest';
  /** Whether the prompt will resume it (else it starts a new task). */
  readonly checked: boolean;
  readonly setChecked: (checked: boolean) => void;
}

/** Sending the composer's prompt. */
export interface ComposerSubmit {
  readonly submit: (text: string) => void;
  /** A command is on its way. */
  readonly busy: boolean;
  /** The session the prompt can resume, when there is one. */
  readonly resume: ResumeOption | null;
}

/**
 * Sends the prompt: as a follow-up of the redo item's task when « reprendre la session » is on
 * and possible (its scope widened to the item if it moved out), as a follow-up of the latest task
 * when it finished recently and the user kept « continuer » (the composer's scope replaces the
 * task's), else as a new task over the composer's scope, with the model and effort chosen. A redo
 * appends a reference to the item so the agent knows which one is meant.
 */
export function useComposerSubmit(scope: ComposerScope | null, projectId: string | null) {
  const { t } = useTranslation();
  const commands = useTaskCommands();
  const tasks = useAiTasks((state) => state.tasks);
  const redo = useComposer((state) => state.redo);
  const resumeRedo = useComposer((state) => state.resume);
  const declined = useComposer((state) => state.declinedTaskId);
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const now = useNow(true, CONTINUE_CLOCK_MS);
  const [busy, setBusy] = useState(false);
  const labelOf = (item: ItemEntity) => {
    const { label, fallback } = itemLabel(item, assets ?? {});
    return label ?? t(`editor.timeline.items.${fallback}`);
  };
  const origin = redo?.taskId == null ? null : tasks.find((task) => task.id === redo.taskId);
  const redoTask =
    origin != null && !isTaskActive(origin) && origin.agentSessionId !== null ? origin : null;
  const latest =
    redo === null && scope !== null ? continuableTask(tasks, scope.sequence.id, now) : null;
  const resume: ResumeOption | null =
    redoTask !== null
      ? {
          task: redoTask,
          kind: 'redo',
          checked: resumeRedo,
          setChecked: (checked) => {
            updateComposer({ resume: checked });
          },
        }
      : latest !== null
        ? {
            task: latest,
            kind: 'latest',
            checked: declined !== latest.id,
            setChecked: (checked) => {
              updateComposer({ declinedTaskId: checked ? null : latest.id });
            },
          }
        : null;

  const submit = (text: string) => {
    if (scope === null || projectId === null || busy) return;
    const { model, effort } = composerStore.getState();
    const reference =
      redo === null || scope.redoItem === null
        ? ''
        : '\n\n' +
          t('ai.composer.itemReference', {
            label: labelOf(scope.redoItem),
            id: redo.itemId,
          });
    const prompt = text.trim() + reference;
    setBusy(true);
    const followed = resume?.checked === true ? resume : null;
    const followScope =
      followed === null
        ? undefined
        : followed.kind === 'redo' && redo !== null
          ? (widenedScope(followed.task.scope, redo) ?? undefined)
          : scope.derived.scope;
    const sent =
      followed !== null
        ? commands.followUp(projectId, followed.task.id, prompt, followScope)
        : commands.submit({
            projectId,
            prompt,
            scope: scope.derived.scope,
            ...(model === null ? {} : { model }),
            ...(effort === null ? {} : { effort }),
          });
    void sent.then((taskId) => {
      setBusy(false);
      if (taskId === null) return;
      clearAfterSubmit(text);
      showTask(taskId);
    });
  };
  return { submit, busy, resume } satisfies ComposerSubmit;
}
