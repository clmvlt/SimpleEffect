import { EmptyState } from '@se/ui';
import { ListChecks } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useAiTasks } from '../../../shared/ai-tasks';
import { hasUsage, orderedTasks, totalUsage } from '../services/task-view';
import { usageText } from '../services/usage-format';
import { TaskCard } from './TaskCard';

/**
 * The project's AI tasks, running ones first, then queued, then the most recent finished ones,
 * with the tokens and cost of every task in the header. Editing is never blocked by a task.
 */
export function TaskList() {
  const { t, i18n } = useTranslation();
  const tasks = useAiTasks((state) => state.tasks);
  const focus = useAiTasks((state) => state.focus);
  const total = totalUsage(tasks);
  const usage = usageText(total, i18n.language);
  if (tasks.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 flex-col justify-center overflow-auto">
        <EmptyState
          icon={ListChecks}
          title={t('ai.tasks.emptyTitle')}
          description={t('ai.tasks.emptyDescription')}
        />
      </div>
    );
  }
  return (
    <section aria-label={t('ai.tasks.title')} className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-2 px-3 pt-2 pb-1 text-xs text-text-muted">
        <span>{t('ai.tasks.count', { count: tasks.length })}</span>
        {hasUsage(total) && (
          <span className="ml-auto tabular-nums" title={t('ai.tasks.totalHint')}>
            {usage.cost === null
              ? t('ai.tasks.tokens', { tokens: usage.tokens })
              : t('ai.tasks.usage', { tokens: usage.tokens, cost: usage.cost })}
          </span>
        )}
      </header>
      <ol className="flex min-h-0 flex-1 flex-col gap-2 overflow-auto px-2 pb-2">
        {orderedTasks(tasks).map((task) => (
          <li key={task.id}>
            <TaskCard task={task} focusNonce={focus?.taskId === task.id ? focus.nonce : null} />
          </li>
        ))}
      </ol>
    </section>
  );
}
