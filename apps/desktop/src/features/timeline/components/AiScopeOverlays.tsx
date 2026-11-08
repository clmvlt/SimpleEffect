import type { SequenceId } from '@se/protocol';
import { cn, Icon } from '@se/ui';
import { TriangleAlert } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { activeTasksOn, showTask, useAiTasks } from '../../../shared/ai-tasks';
import { aiScopeBoxes } from '../services/ai-scope-boxes';
import type { TrackLayout } from '../services/track-layout';
import type { Viewport } from '../services/viewport';

/** Props of {@link AiScopeOverlays}. */
export interface AiScopeOverlaysProps {
  readonly sequenceId: SequenceId;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
}

/**
 * The scopes of the AI tasks running or queued on the sequence shown: shaded in the AI colour,
 * labelled « IA #3 en cours » (a click shows the task), with a discreet mark when the user edited
 * inside a running task's scope meanwhile. Editing there stays allowed.
 */
export function AiScopeOverlays({ sequenceId, layout, viewport }: AiScopeOverlaysProps) {
  const { t } = useTranslation();
  const tasks = useAiTasks((state) => state.tasks);
  const numbers = useAiTasks((state) => state.numbers);
  const userEdited = useAiTasks((state) => state.userEdited);
  const active = activeTasksOn(tasks, sequenceId);
  return aiScopeBoxes(active, layout, viewport).map((box) => {
    const task = active.find((candidate) => candidate.id === box.taskId);
    if (task === undefined) return null;
    const running = task.status.kind === 'running';
    const number = numbers.get(task.id) ?? 0;
    const edited = running && userEdited.has(task.id);
    return (
      <div key={box.key}>
        <div
          aria-hidden
          data-ai-scope={task.id}
          className={cn(
            'pointer-events-none absolute z-10 border-x border-ai/70',
            running ? 'bg-ai-muted/50' : 'bg-ai-muted/25 border-dashed',
          )}
          style={{ left: box.left, width: box.width, top: box.top, height: box.height }}
        />
        {box.labelled && (
          <button
            type="button"
            data-ai-scope-label={task.id}
            title={edited ? t('editor.timeline.ai.userEdited') : undefined}
            className="absolute z-20 inline-flex h-4 max-w-40 items-center gap-1 truncate rounded-xs border border-ai bg-ai-muted px-1 text-2xs leading-3 font-medium text-text hover:bg-ai/40"
            style={{ left: Math.max(box.left, 0) + 2, top: box.top + 2 }}
            onPointerDown={(event) => {
              event.stopPropagation();
            }}
            onClick={() => {
              showTask(task.id);
            }}
          >
            {t(running ? 'editor.timeline.ai.running' : 'editor.timeline.ai.queued', { number })}
            {edited && (
              <>
                <Icon icon={TriangleAlert} size="xs" className="text-warning" />
                <span className="sr-only">{t('editor.timeline.ai.userEdited')}</span>
              </>
            )}
          </button>
        )}
      </div>
    );
  });
}
