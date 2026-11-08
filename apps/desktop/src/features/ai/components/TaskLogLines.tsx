import { cn } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import type { TaskLogEntry } from '../services/task-entries';

/** Props of {@link TaskLogLines}. */
export interface TaskLogLinesProps {
  readonly entries: readonly TaskLogEntry[];
  /** Selects the items a tool call names (its line becomes a button when some still exist). */
  readonly onSelectItems?: ((ids: readonly string[]) => void) | undefined;
  /** The journal dialog: whole texts, and each call's input and output. */
  readonly detailed?: boolean | undefined;
}

const MARKS = { running: '…', ok: '✓', error: '✗' } as const;

/**
 * A task's log as lines: the agent's texts, its tool calls (« ✓ add_text “Abonne-toi”
 * 00:51→00:54 », clickable to select the items they made or changed), follow-up prompts, final
 * answers and failures (translated by code).
 */
export function TaskLogLines({ entries, onSelectItems, detailed = false }: TaskLogLinesProps) {
  const { t } = useTranslation();
  const items = useProjectSelector((state) => state.mirror?.items);
  return (
    <ol className="flex flex-col gap-1">
      {entries.map((entry) => {
        switch (entry.kind) {
          case 'run':
            return (
              <li key={entry.key} className="pt-1 font-medium text-text">
                {t(entry.runKind === 'retry' ? 'ai.log.retry' : 'ai.log.followUp', {
                  prompt: entry.prompt,
                })}
              </li>
            );
          case 'text':
          case 'answer':
            return (
              <li
                key={entry.key}
                className={cn('font-sans text-text', !detailed && 'line-clamp-3')}
              >
                {entry.text}
              </li>
            );
          case 'failed':
            return (
              <li key={entry.key} className="text-danger">
                {t(`ai.failure.${entry.failure}`)}
                {detailed && <span className="block text-text-muted">{entry.message}</span>}
              </li>
            );
          case 'tool': {
            const existing = entry.ids.filter((id) => items?.[id] !== undefined);
            const line = `${MARKS[entry.state]} ${entry.line}`;
            return (
              <li
                key={entry.key}
                data-tool={entry.tool}
                className={entry.state === 'error' ? 'text-danger' : 'text-text-muted'}
              >
                {existing.length > 0 && onSelectItems !== undefined ? (
                  <button
                    type="button"
                    title={t('ai.log.selectItems', { count: existing.length })}
                    className="text-left underline-offset-2 hover:text-text hover:underline"
                    onClick={() => {
                      onSelectItems(existing);
                    }}
                  >
                    {line}
                  </button>
                ) : (
                  line
                )}
                {detailed && (
                  <pre className="mt-0.5 overflow-x-auto rounded-xs bg-panel p-1 text-2xs whitespace-pre-wrap text-text-subtle">
                    {entry.input}
                    {entry.output !== null && `\n→ ${entry.output}`}
                  </pre>
                )}
              </li>
            );
          }
        }
      })}
    </ol>
  );
}
