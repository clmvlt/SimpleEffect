import type { RecentProject } from '@se/protocol';
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

/** Props of {@link TrashProjectDialog}. */
export interface TrashProjectDialogProps {
  /** The project to trash; the dialog is open while it is set. */
  readonly project: RecentProject | null;
  readonly onClose: () => void;
  readonly onConfirm: (project: RecentProject) => void;
}

/** Confirms moving a project folder to the OS trash. */
export function TrashProjectDialog({ project, onClose, onConfirm }: TrashProjectDialogProps) {
  const { t } = useTranslation();
  return (
    <AlertDialog
      open={project !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>
            {t('projects.trashDialog.title', { name: project?.name ?? '' })}
          </AlertDialogTitle>
          <AlertDialogDescription>{t('projects.trashDialog.description')}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('projects.trashDialog.cancel')}</AlertDialogCancel>
          <AlertDialogAction
            variant="danger"
            onClick={() => {
              if (project !== null) onConfirm(project);
            }}
          >
            {t('projects.trashDialog.confirm')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
