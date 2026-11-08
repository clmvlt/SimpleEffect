import type { Task } from '@se/protocol';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { TaskLogEntry } from '../services/task-entries';
import { TaskLogLines } from './TaskLogLines';

/** Props of {@link TaskLogDialog}. */
export interface TaskLogDialogProps {
  readonly task: Task;
  readonly number: number;
  readonly entries: readonly TaskLogEntry[];
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  readonly onSelectItems: (ids: readonly string[]) => void;
}

/** « Journal complet »: every entry of a task's log, with each tool call's input and output. */
export function TaskLogDialog({
  task,
  number,
  entries,
  open,
  onOpenChange,
  onSelectItems,
}: TaskLogDialogProps) {
  const { t } = useTranslation();
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg" closeLabel={t('common.close')} className="max-h-[80vh]">
        <DialogHeader>
          <DialogTitle>{t('ai.log.title', { number })}</DialogTitle>
          <DialogDescription className="break-words">{task.prompt}</DialogDescription>
        </DialogHeader>
        <div className="timecode min-h-0 overflow-auto rounded-sm border border-border bg-input p-2 text-xs leading-relaxed select-text">
          {entries.length === 0 ? (
            <p className="text-text-muted">{t('ai.log.empty')}</p>
          ) : (
            <TaskLogLines entries={entries} detailed onSelectItems={onSelectItems} />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
