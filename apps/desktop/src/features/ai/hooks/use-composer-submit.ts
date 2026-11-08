import type { ItemEntity, Task } from '@se/protocol';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { isTaskActive, showTask, useAiTasks } from '../../../shared/ai-tasks';
import { useProjectSelector } from '../../../shared/project-store';
import { itemLabel } from '../services/item-label';
import { widenedScope } from '../services/task-scope';
import { clearAfterSubmit, composerStore, useComposer } from '../store/composer-store';
import type { ComposerScope } from './use-composer-scope';
import { useTaskCommands } from './use-task-commands';

/** Sending the composer's prompt. */
export interface ComposerSubmit {
  readonly submit: (text: string) => void;
  /** A command is on its way. */
  readonly busy: boolean;
  /** The task whose session the prompt will resume (« Refaire avec l'IA »), when it can. */
  readonly resumable: Task | null;
}

/**
 * Sends the prompt: as a follow-up of the redo item's task when « reprendre la session » is on
 * and possible (its scope widened to the item if it moved out), else as a new task over the
 * composer's scope, with the model and effort chosen. A redo appends a reference to the item so
 * the agent knows which one is meant.
 */
export function useComposerSubmit(scope: ComposerScope | null, projectId: string | null) {
  const { t } = useTranslation();
  const commands = useTaskCommands();
  const tasks = useAiTasks((state) => state.tasks);
  const redo = useComposer((state) => state.redo);
  const resume = useComposer((state) => state.resume);
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const [busy, setBusy] = useState(false);
  const labelOf = (item: ItemEntity) => {
    const { label, fallback } = itemLabel(item, assets ?? {});
    return label ?? t(`editor.timeline.items.${fallback}`);
  };
  const origin = redo?.taskId == null ? null : tasks.find((task) => task.id === redo.taskId);
  const resumable =
    origin != null && !isTaskActive(origin) && origin.agentSessionId !== null ? origin : null;

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
    const sent =
      resume && resumable !== null && redo !== null
        ? commands.followUp(
            projectId,
            resumable.id,
            prompt,
            widenedScope(resumable.scope, redo) ?? undefined,
          )
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
  return { submit, busy, resumable } satisfies ComposerSubmit;
}
