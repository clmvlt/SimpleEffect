import type { CreateProjectRequest, NewProjectDefaults } from '@se/protocol';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Spinner,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useNewProjectDefaults } from '../hooks/use-new-project-defaults';
import { NewProjectForm } from './NewProjectForm';

/** Props of {@link NewProjectDialog}. */
export interface NewProjectDialogProps {
  readonly open: boolean;
  readonly onOpenChange: (open: boolean) => void;
  /** Asks the core for the proposed folder, formats and frame rates. */
  readonly loadDefaults: () => Promise<NewProjectDefaults>;
  /** Shows the native folder picker starting at `current`; `null` when cancelled. */
  readonly chooseFolder: (current: string) => Promise<string | null>;
  /** Creates the project; resolves `false` when it failed (the dialog stays open). */
  readonly onCreate: (request: CreateProjectRequest) => Promise<boolean>;
  /** Called when the defaults cannot be loaded. */
  readonly onError: (error: unknown) => void;
}

/** The « Nouveau projet » dialog: name, location, format, resolution and frame rate. */
export function NewProjectDialog({
  open,
  onOpenChange,
  loadDefaults,
  chooseFolder,
  onCreate,
  onError,
}: NewProjectDialogProps) {
  const { t } = useTranslation();
  const defaults = useNewProjectDefaults(open, loadDefaults, onError);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="md" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t('projects.newDialog.title')}</DialogTitle>
          <DialogDescription>{t('projects.newDialog.description')}</DialogDescription>
        </DialogHeader>
        {defaults === null ? (
          <div className="flex justify-center py-8">
            <Spinner size="lg" label={t('editor.loading')} />
          </div>
        ) : (
          <NewProjectForm defaults={defaults} chooseFolder={chooseFolder} onCreate={onCreate} />
        )}
        {defaults === null && (
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => {
                onOpenChange(false);
              }}
            >
              {t('projects.newDialog.cancel')}
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
