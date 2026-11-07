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

import type { PendingRelink } from '../hooks/use-asset-actions';
import { fileName } from '../services/asset-list';

/** Props of {@link RelinkConfirmDialog}. */
export interface RelinkConfirmDialogProps {
  readonly pending: PendingRelink | null;
  readonly onConfirm: () => void;
  readonly onCancel: () => void;
}

/** Asks before relinking an asset to a file whose content differs from the imported one. */
export function RelinkConfirmDialog({ pending, onConfirm, onCancel }: RelinkConfirmDialogProps) {
  const { t } = useTranslation();
  const file = pending === null ? '' : fileName(pending.path);
  return (
    <AlertDialog
      open={pending !== null}
      onOpenChange={(open) => {
        if (!open) onCancel();
      }}
    >
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>{t('editor.media.relinkConfirm.title')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('editor.media.relinkConfirm.description', { file })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('editor.media.relinkConfirm.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>
            {t('editor.media.relinkConfirm.confirm')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
