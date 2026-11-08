import type { ComponentProps, ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { Progress } from '../../primitives/progress';
import { ScrollArea } from '../../primitives/scroll-area';
import { StatusDot, type StatusDotProps } from '../status-dot';

/** Lifecycle of an AI task. */
export type AiTaskStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';

/** Visible texts of the card; the app passes localised ones. */
export interface AiTaskCardLabels {
  status: Record<AiTaskStatus, string>;
  /** Accessible name of the streaming log. */
  log: string;
  /** Accessible name of the progress bar. */
  progress: string;
}

/** Default (English) texts. */
export const DEFAULT_AI_TASK_LABELS: AiTaskCardLabels = {
  status: {
    queued: 'Queued',
    running: 'Running',
    completed: 'Completed',
    failed: 'Failed',
    cancelled: 'Cancelled',
  },
  log: 'Task log',
  progress: 'Task progress',
};

const STATUS_TONES: Record<AiTaskStatus, NonNullable<StatusDotProps['tone']>> = {
  queued: 'neutral',
  running: 'ai',
  completed: 'success',
  failed: 'danger',
  cancelled: 'neutral',
};

/** Props of {@link AiTaskCard}. */
export interface AiTaskCardProps extends Omit<ComponentProps<'article'>, 'title'> {
  status: AiTaskStatus;
  /** What was asked, e.g. « Retire les silences de la séquence ». */
  title: string;
  /** Secondary facts at the top right (elapsed time, model). */
  meta?: ReactNode | undefined;
  /** A line under the title (what the task may change: its scope). */
  context?: ReactNode | undefined;
  /** Streaming log of the agent (lines appended while it runs); announced politely. */
  log?: ReactNode | undefined;
  /** Why the task failed, shown in a danger box. */
  error?: ReactNode | undefined;
  /** Cost or token usage, at the bottom left. */
  cost?: ReactNode | undefined;
  /** Buttons (cancel, review changes, retry), at the bottom right. */
  actions?: ReactNode | undefined;
  /** Progress 0–100 while running; indeterminate when omitted. */
  progress?: number | undefined;
  /** Shown last, under the footer (an inline follow-up form). */
  children?: ReactNode | undefined;
  labels?: Partial<Omit<AiTaskCardLabels, 'status'>> & {
    status?: Partial<Record<AiTaskStatus, string>> | undefined;
  };
}

/**
 * One AI task in the tasks panel (visual only: the app maps task events to props). Shows the
 * status with a coloured dot, the request and its context (scope), a streaming log, progress while
 * running, the error if it failed, the cost and actions, then optional content (a follow-up form).
 */
export function AiTaskCard({
  status,
  title,
  meta,
  context,
  log,
  error,
  cost,
  actions,
  progress,
  children,
  labels: overrides,
  className,
  ...props
}: AiTaskCardProps) {
  const labels = {
    ...DEFAULT_AI_TASK_LABELS,
    ...overrides,
    status: { ...DEFAULT_AI_TASK_LABELS.status, ...overrides?.status },
  };
  const hasFooter = cost !== undefined || actions !== undefined;
  return (
    <article
      data-status={status}
      className={cn(
        'flex flex-col gap-2 rounded-md border border-border bg-panel-raised p-2.5',
        status === 'running' && 'border-ai-muted',
        status === 'cancelled' && 'opacity-75',
        className,
      )}
      {...props}
    >
      <header className="flex items-center gap-2 text-xs">
        <StatusDot tone={STATUS_TONES[status]} pulse={status === 'running'} />
        <span className="font-medium text-text-muted">{labels.status[status]}</span>
        {meta !== undefined && (
          <span className="ml-auto text-text-subtle tabular-nums">{meta}</span>
        )}
      </header>
      <h3 className="text-sm break-words text-text">{title}</h3>
      {context !== undefined && <div className="text-xs text-text-muted">{context}</div>}
      {status === 'running' && (
        <Progress value={progress ?? null} size="xs" tone="ai" aria-label={labels.progress} />
      )}
      {log !== undefined && (
        <ScrollArea className="max-h-40 rounded-sm border border-border bg-input">
          <div
            role="log"
            aria-label={labels.log}
            className="timecode p-2 text-2xs leading-relaxed whitespace-pre-wrap text-text-muted"
          >
            {log}
          </div>
        </ScrollArea>
      )}
      {error !== undefined && (
        <div role="alert" className="rounded-sm bg-danger-muted px-2 py-1.5 text-xs text-danger">
          {error}
        </div>
      )}
      {hasFooter && (
        <footer className="flex items-center gap-2">
          {cost !== undefined && (
            <span className="text-xs text-text-subtle tabular-nums">{cost}</span>
          )}
          {actions !== undefined && (
            <div className="ml-auto flex flex-wrap items-center justify-end gap-1">{actions}</div>
          )}
        </footer>
      )}
      {children}
    </article>
  );
}
