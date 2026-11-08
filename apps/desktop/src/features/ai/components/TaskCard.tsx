import type { Task, TaskRevertReport } from '@se/protocol';
import { AiTaskCard, PromptComposer, cn } from '@se/ui';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useTaskNumber } from '../../../shared/ai-tasks';
import { useAiStatus } from '../hooks/use-ai-status';
import { useNow } from '../hooks/use-now';
import { useTaskCommands } from '../hooks/use-task-commands';
import { useTaskEvents } from '../hooks/use-task-events';
import { useTaskNavigation } from '../hooks/use-task-navigation';
import { shortTime } from '../services/short-time';
import { taskLogEntries } from '../services/task-entries';
import { taskActions } from '../services/task-log';
import { elapsedMs, hasUsage } from '../services/task-view';
import { usageText } from '../services/usage-format';
import { RevertDialog } from './RevertDialog';
import { TaskActionButtons } from './TaskActionButtons';
import { TaskLogDialog } from './TaskLogDialog';
import { TaskLogLines } from './TaskLogLines';
import { TaskScopeButton } from './TaskScopeButton';

/** Log entries shown in a card (the journal dialog shows them all). */
const CARD_ENTRIES = 8;

/** Props of {@link TaskCard}. */
export interface TaskCardProps {
  readonly task: Task;
  /** Changes when a « Voir » asks to show this card (it scrolls into view and is outlined). */
  readonly focusNonce: number | null;
}

/**
 * One task of the panel, live: status, number and elapsed time, the request, its scope (a click
 * shows it on the timeline), the latest log lines (tool calls select what they touched), the
 * failure translated by code, tokens and cost, and its actions — cancel, retry, follow-up (an
 * inline field resuming the session), revert (with its conflicts listed first), the changes and
 * the complete log.
 */
export function TaskCard({ task, focusNonce }: TaskCardProps) {
  const { t, i18n } = useTranslation();
  const number = useTaskNumber(task.id) ?? 0;
  const status = useAiStatus();
  const commands = useTaskCommands();
  const navigation = useTaskNavigation();
  const events = useTaskEvents(task.projectId, task.id);
  const now = useNow(task.status.kind === 'running');
  const [followUp, setFollowUp] = useState<string | null>(null);
  const [logOpen, setLogOpen] = useState(false);
  const [preview, setPreview] = useState<TaskRevertReport | null>(null);
  const element = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (focusNonce !== null) element.current?.scrollIntoView({ block: 'nearest' });
  }, [focusNonce]);
  const entries = taskLogEntries(task, events);
  const elapsed = elapsedMs(task, now);
  const actions = taskActions(task, status?.capabilities.resumeSessions ?? false);
  const usage = usageText(task.usage, i18n.language);
  const revert = () => {
    void commands.previewRevert(task.projectId, task.id).then((report) => {
      if (report === null) return;
      if (report.reverted && report.conflicts.length === 0) {
        void commands.revert(task.projectId, task.id);
      } else {
        setPreview(report);
      }
    });
  };
  const sendFollowUp = (prompt: string) => {
    void commands.followUp(task.projectId, task.id, prompt).then((taskId) => {
      if (taskId !== null) setFollowUp(null);
    });
  };
  return (
    <div ref={element} data-task-id={task.id}>
      <AiTaskCard
        className={cn(focusNonce !== null && 'ring-2 ring-ai')}
        status={task.status.kind}
        title={task.prompt}
        meta={
          elapsed === null
            ? `#${String(number)}`
            : `#${String(number)} · ${shortTime(Math.round(elapsed / 1000))}`
        }
        context={
          <TaskScopeButton
            task={task}
            onReveal={() => {
              navigation.revealScope(task);
            }}
          />
        }
        log={
          entries.length === 0 ? undefined : (
            <TaskLogLines
              entries={entries.slice(-CARD_ENTRIES)}
              onSelectItems={navigation.selectExisting}
            />
          )
        }
        error={
          task.status.kind === 'failed' ? (
            <>
              {t(`ai.failure.${task.status.failure}`)}
              <span data-failure-detail="" className="mt-1 block text-2xs break-words opacity-80">
                {task.status.message}
              </span>
            </>
          ) : undefined
        }
        cost={
          hasUsage(task.usage)
            ? usage.cost === null
              ? t('ai.tasks.tokens', { tokens: usage.tokens })
              : t('ai.tasks.usage', { tokens: usage.tokens, cost: usage.cost })
            : undefined
        }
        actions={
          <TaskActionButtons
            actions={actions}
            followingUp={followUp !== null}
            on={{
              cancel: () => {
                commands.cancel(task.projectId, task.id);
              },
              retry: () => {
                commands.retry(task.projectId, task.id);
              },
              followUp: () => {
                setFollowUp((open) => (open === null ? '' : null));
              },
              revert,
              showChanges: () => {
                navigation.showChanges(task);
              },
              showLog: () => {
                setLogOpen(true);
              },
            }}
          />
        }
        labels={{
          status: {
            queued: t('ai.tasks.status.queued'),
            running: t('ai.tasks.status.running'),
            completed: t('ai.tasks.status.completed'),
            failed: t('ai.tasks.status.failed'),
            cancelled: t('ai.tasks.status.cancelled'),
          },
          log: t('ai.tasks.log', { number }),
          progress: t('ai.tasks.progress', { number }),
        }}
      >
        {followUp !== null && (
          <PromptComposer
            label={t('ai.tasks.followUpLabel', { number })}
            placeholder={t('ai.tasks.followUpPlaceholder')}
            submitLabel={t('ai.tasks.followUpSubmit')}
            value={followUp}
            onValueChange={setFollowUp}
            onSubmit={sendFollowUp}
          />
        )}
      </AiTaskCard>
      <TaskLogDialog
        task={task}
        number={number}
        entries={entries}
        open={logOpen}
        onOpenChange={setLogOpen}
        onSelectItems={navigation.selectExisting}
      />
      <RevertDialog
        number={number}
        preview={preview}
        onClose={() => {
          setPreview(null);
        }}
        onConfirm={() => {
          setPreview(null);
          void commands.revert(task.projectId, task.id);
        }}
      />
    </div>
  );
}
