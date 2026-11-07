import type { Task } from '@se/protocol';
import { Badge, Button, Textarea } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { selectProjectId, useProjectSelector } from '../../../../shared/project-store';
import { useActiveSequence } from '../../../timeline';
import { useAiStatus } from '../../hooks/use-ai-status';
import { useProjectTasks, useTaskEvents } from '../../hooks/use-project-tasks';
import { useTaskCommands } from '../../hooks/use-task-commands';
import { canSubmitTasks } from '../../services/ai-indicator';
import { describeEvent, taskActions } from '../../services/task-log';

/**
 * « Tester une tâche » (development builds only): sends a prompt as a task over the active
 * sequence, lists the project's tasks with cancel / retry / follow-up, and shows the selected
 * task's event stream as it arrives. Raw on purpose: prompt 20 builds the real task UX.
 */
export function AgentTestPanel() {
  const { t } = useTranslation();
  const projectId = useProjectSelector(selectProjectId);
  const sequence = useActiveSequence();
  const status = useAiStatus();
  const tasks = useProjectTasks(projectId);
  const commands = useTaskCommands();
  const [prompt, setPrompt] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const events = useTaskEvents(projectId, selected);
  const ready = canSubmitTasks(status) && projectId !== null && sequence !== null;
  const submit = () => {
    if (!ready || prompt.trim() === '') return;
    void commands
      .submit({ projectId, prompt, scope: { sequenceId: sequence.id, range: null, tracks: null } })
      .then((taskId) => {
        if (taskId !== null) setSelected(taskId);
      });
  };
  return (
    <section aria-label={t('ai.devTest.title')} className="flex min-h-0 flex-col gap-2 p-2 text-xs">
      <h3 className="font-semibold text-text">{t('ai.devTest.title')}</h3>
      <Textarea
        aria-label={t('ai.devTest.prompt')}
        placeholder={t('ai.devTest.placeholder')}
        value={prompt}
        rows={3}
        onChange={(event) => {
          setPrompt(event.target.value);
        }}
      />
      <Button
        size="xs"
        variant="primary"
        disabled={!ready || prompt.trim() === ''}
        onClick={submit}
      >
        {t('ai.devTest.submit')}
      </Button>
      <ul aria-label={t('ai.devTest.tasks')} className="flex flex-col gap-1">
        {tasks.map((task) => (
          <TaskRow
            key={task.id}
            task={task}
            selected={task.id === selected}
            onSelect={() => {
              setSelected(task.id);
            }}
            onCancel={() => {
              commands.cancel(task.projectId, task.id);
            }}
            onRetry={() => {
              commands.retry(task.projectId, task.id);
            }}
            onFollowUp={() => {
              if (prompt.trim() !== '') commands.followUp(task.projectId, task.id, prompt);
            }}
          />
        ))}
      </ul>
      {selected !== null && (
        <ol
          aria-label={t('ai.devTest.events')}
          className="flex min-h-0 flex-col gap-1 overflow-auto rounded-sm bg-input p-2 font-mono text-2xs text-text-muted select-text"
        >
          {events.map((record) => (
            <li key={record.sequence} className="break-words whitespace-pre-wrap">
              <span className="text-text-subtle">
                {record.run}:{record.sequence}{' '}
              </span>
              {describeEvent(record)}
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

interface TaskRowProps {
  readonly task: Task;
  readonly selected: boolean;
  readonly onSelect: () => void;
  readonly onCancel: () => void;
  readonly onRetry: () => void;
  readonly onFollowUp: () => void;
}

function TaskRow({ task, selected, onSelect, onCancel, onRetry, onFollowUp }: TaskRowProps) {
  const { t } = useTranslation();
  const actions = taskActions(task);
  return (
    <li
      data-task-status={task.status.kind}
      className={selected ? 'rounded-sm bg-hover p-1' : 'rounded-sm p-1'}
    >
      <button type="button" className="w-full truncate text-left text-text" onClick={onSelect}>
        <Badge variant="neutral">{t(`ai.devTest.status.${task.status.kind}`)}</Badge> {task.prompt}
      </button>
      <div className="flex gap-1 pt-1">
        {actions.cancel && (
          <Button size="xs" variant="ghost" onClick={onCancel}>
            {t('ai.devTest.cancel')}
          </Button>
        )}
        {actions.retry && (
          <Button size="xs" variant="ghost" onClick={onRetry}>
            {t('ai.devTest.retry')}
          </Button>
        )}
        {actions.followUp && (
          <Button size="xs" variant="ghost" onClick={onFollowUp}>
            {t('ai.devTest.followUp')}
          </Button>
        )}
      </div>
    </li>
  );
}
