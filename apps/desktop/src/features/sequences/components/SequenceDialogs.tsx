import type { SequenceEntity } from '@se/protocol';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { useFrameRates } from '../hooks/use-frame-rates';
import type { SequenceActions } from '../hooks/use-sequence-actions';
import { holdsItems } from '../services/sequence-format';
import { closeSequenceDialog, useSequenceDialog } from '../store/sequence-dialog-store';
import { SequenceFormatForm } from './SequenceFormatForm';

/** Props of {@link SequenceDialogs}. */
export interface SequenceDialogsProps {
  readonly actions: SequenceActions;
}

/**
 * The host of the sequence dialogs (opened with `openSequenceDialog`): « Format de la séquence »,
 * « Créer une version… » and the removal confirmation. Mount it once where sequences are shown.
 */
export function SequenceDialogs({ actions }: SequenceDialogsProps) {
  const dialog = useSequenceDialog();
  const sequence = useProjectSelector((state) =>
    dialog === null ? undefined : state.mirror?.sequences[dialog.sequenceId],
  );
  const tracks = useProjectSelector((state) => state.mirror?.tracks);
  if (dialog === null || sequence === undefined || tracks === undefined) return null;
  if (dialog.kind === 'remove') {
    return <RemoveSequenceDialog sequence={sequence} onConfirm={actions.remove} />;
  }
  const submit =
    dialog.kind === 'format'
      ? (values: Parameters<SequenceActions['changeFormat']>[1]) =>
          actions.changeFormat(sequence.id, values)
      : (values: Parameters<SequenceActions['createVersion']>[1]) =>
          actions.createVersion(sequence.id, values);
  return (
    <FormatDialog
      key={`${dialog.kind}-${sequence.id}`}
      mode={dialog.kind}
      sequence={sequence}
      holdsItems={holdsItems({ sequence, tracks })}
      onSubmit={async (values) => {
        const done = await submit(values);
        if (done) closeSequenceDialog();
        return done;
      }}
    />
  );
}

interface FormatDialogProps {
  readonly mode: 'format' | 'version';
  readonly sequence: SequenceEntity;
  readonly holdsItems: boolean;
  readonly onSubmit: Parameters<typeof SequenceFormatForm>[0]['onSubmit'];
}

function FormatDialog({ mode, sequence, holdsItems: held, onSubmit }: FormatDialogProps) {
  const { t } = useTranslation();
  const frameRates = useFrameRates();
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) closeSequenceDialog();
      }}
    >
      <DialogContent size="md" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>
            {t(`editor.sequences.dialog.title.${mode}`, { name: sequence.name })}
          </DialogTitle>
          <DialogDescription>{t(`editor.sequences.dialog.description.${mode}`)}</DialogDescription>
        </DialogHeader>
        <SequenceFormatForm
          mode={mode}
          sequence={sequence}
          holdsItems={held}
          frameRates={frameRates}
          onSubmit={onSubmit}
        />
      </DialogContent>
    </Dialog>
  );
}

function RemoveSequenceDialog({
  sequence,
  onConfirm,
}: {
  readonly sequence: SequenceEntity;
  readonly onConfirm: SequenceActions['remove'];
}) {
  const { t } = useTranslation();
  return (
    <AlertDialog
      open
      onOpenChange={(open) => {
        if (!open) closeSequenceDialog();
      }}
    >
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>
            {t('editor.sequences.remove.title', { name: sequence.name })}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {t('editor.sequences.remove.description')}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('editor.sequences.remove.cancel')}</AlertDialogCancel>
          <AlertDialogAction
            variant="danger"
            onClick={() => {
              closeSequenceDialog();
              void onConfirm(sequence.id);
            }}
          >
            {t('editor.sequences.remove.confirm')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
