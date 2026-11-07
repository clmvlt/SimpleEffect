import type { RecentProject } from '@se/protocol';
import {
  Button,
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Field,
  Input,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

/** Props of {@link RenameProjectDialog}. */
export interface RenameProjectDialogProps {
  /** The project to rename; the dialog is open while it is set. */
  readonly project: RecentProject | null;
  readonly onClose: () => void;
  /** Renames; resolves `false` when it failed (the dialog stays open). */
  readonly onRename: (project: RecentProject, name: string) => Promise<boolean>;
}

/** Asks for a project's new name. */
export function RenameProjectDialog({ project, onClose, onRename }: RenameProjectDialogProps) {
  const { t } = useTranslation();
  return (
    <Dialog
      open={project !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent size="sm" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t('projects.renameDialog.title')}</DialogTitle>
        </DialogHeader>
        {project !== null && (
          <RenameForm key={project.path} project={project} onClose={onClose} onRename={onRename} />
        )}
      </DialogContent>
    </Dialog>
  );
}

function RenameForm({
  project,
  onClose,
  onRename,
}: RenameProjectDialogProps & { readonly project: RecentProject }) {
  const { t } = useTranslation();
  const [name, setName] = useState(project.name);
  const [tried, setTried] = useState(false);
  const empty = name.trim() === '';
  return (
    <form
      noValidate
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        setTried(true);
        if (empty) return;
        void onRename(project, name.trim()).then((renamed) => {
          if (renamed) onClose();
        });
      }}
    >
      <Field
        label={t('projects.renameDialog.name')}
        error={tried && empty ? t('projects.renameDialog.nameRequired') : undefined}
      >
        <Input
          autoFocus
          value={name}
          onChange={(event) => {
            setName(event.target.value);
          }}
        />
      </Field>
      <DialogFooter>
        <DialogClose asChild>
          <Button type="button" variant="ghost">
            {t('projects.renameDialog.cancel')}
          </Button>
        </DialogClose>
        <Button type="submit" variant="primary">
          {t('projects.renameDialog.confirm')}
        </Button>
      </DialogFooter>
    </form>
  );
}
