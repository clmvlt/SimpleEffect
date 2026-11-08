import type { TaskRevertReport } from '@se/protocol';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { itemLabel } from '../services/item-label';
import { conflictLines, type ConflictLine } from '../services/revert-conflicts';

/** Props of {@link RevertDialog}. */
export interface RevertDialogProps {
  readonly number: number;
  /** What the revert would do (`previewTaskRevert`); `null` closes the dialog. */
  readonly preview: TaskRevertReport | null;
  readonly onConfirm: () => void;
  readonly onClose: () => void;
}

/**
 * Confirms reverting a task whose changes were partly edited since: lists what will stay as it
 * is (items changed or removed by someone else, edits that no longer apply); the rest is undone
 * as one undoable step.
 */
export function RevertDialog({ number, preview, onConfirm, onClose }: RevertDialogProps) {
  const { t } = useTranslation();
  const items = useProjectSelector((state) => state.mirror?.items);
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const describe = (line: ConflictLine): string => {
    switch (line.kind) {
      case 'changed': {
        const item = items?.[line.itemId];
        if (item === undefined || line.removed) return t('ai.revertDialog.removed');
        const { label, fallback } = itemLabel(item, assets ?? {});
        return t('ai.revertDialog.changed', {
          name: label ?? t(`editor.timeline.items.${fallback}`),
        });
      }
      case 'foreign':
        return t('ai.revertDialog.foreign', { count: line.count });
      case 'cannotApply':
        return t('ai.revertDialog.cannotApply');
    }
  };
  const reverts = preview?.reverted === true;
  return (
    <AlertDialog
      open={preview !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('ai.revertDialog.title', { number })}</AlertDialogTitle>
          <AlertDialogDescription>
            {reverts
              ? t('ai.revertDialog.description', { count: preview.revertedItems.length })
              : t('ai.revertDialog.nothing')}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <ul className="flex list-disc flex-col gap-1 pl-5 text-sm text-text">
          {conflictLines(preview?.conflicts ?? []).map((line) => (
            <li key={line.key}>{describe(line)}</li>
          ))}
        </ul>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('ai.revertDialog.keep')}</AlertDialogCancel>
          {reverts && (
            <AlertDialogAction onClick={onConfirm}>
              {t('ai.revertDialog.confirm')}
            </AlertDialogAction>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
