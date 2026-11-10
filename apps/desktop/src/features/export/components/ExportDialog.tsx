import type { Flicks, SequenceEntity } from '@se/protocol';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Spinner,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { byteSize } from '../../../shared/text';
import { formatDuration } from '../../../shared/time';
import { openSequenceDialog } from '../../sequences';
import { useExportForm, type ExportFormDeps } from '../hooks/use-export-form';
import { useStartExport } from '../hooks/use-start-export';
import { CustomSettingsFields } from './CustomSettingsFields';
import { DestinationFields, RangeField } from './DestinationFields';
import { ExportWarnings } from './ExportWarnings';
import { PresetPicker } from './PresetPicker';

/** Props of {@link ExportDialog}. */
export interface ExportDialogProps {
  readonly sequence: SequenceEntity;
  readonly onClose: () => void;
  readonly deps: ExportFormDeps;
}

/**
 * « Exporter » (prompt 27): a preset or custom settings, the range, the destination, then the
 * duration, size estimate and warnings; starting queues the export and closes the dialog (the
 * render runs in the background, followed in the « Exports » menu).
 */
export function ExportDialog({ sequence, onClose, deps }: ExportDialogProps) {
  const { t, i18n } = useTranslation();
  const form = useExportForm(sequence, deps);
  const flow = useStartExport(form, sequence, onClose);
  const { plan } = form;
  const ready = !form.loading && form.directory !== '' && form.templateValid && !plan.blocked;
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent size="lg" closeLabel={t('common.close')}>
        <DialogHeader>
          <DialogTitle>{t('export.dialog.title', { name: sequence.name })}</DialogTitle>
          <DialogDescription>{t('export.dialog.description')}</DialogDescription>
        </DialogHeader>
        <form
          noValidate
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            if (ready) void flow.start();
          }}
        >
          <PresetPicker form={form} />
          {form.choice.kind === 'custom' && (
            <CustomSettingsFields draft={form.draft} onChange={form.editCustom} />
          )}
          <RangeField form={form} />
          <DestinationFields form={form} />
          <p className="flex items-center gap-2 text-sm text-text-muted">
            {t('export.dialog.estimate', {
              duration: formatDuration(Math.max(0, form.span.end - form.span.start) as Flicks),
              size: byteSize(plan.estimatedBytes, i18n.language),
            })}
            {form.support === null && <Spinner size="sm" label={t('export.dialog.probing')} />}
          </p>
          <ExportWarnings
            warnings={plan.warnings}
            onCreateVersion={() => {
              onClose();
              openSequenceDialog({ kind: 'version', sequenceId: sequence.id });
            }}
          />
          <DialogFooter>
            <DialogClose asChild>
              <Button type="button" variant="ghost">
                {t('common.cancel')}
              </Button>
            </DialogClose>
            <Button type="submit" variant="primary" disabled={!ready} loading={flow.starting}>
              {t('export.dialog.start')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
      <ReplaceDialog
        path={flow.replacing}
        onConfirm={flow.confirmReplace}
        onCancel={flow.cancelReplace}
      />
    </Dialog>
  );
}

function ReplaceDialog({
  path,
  onConfirm,
  onCancel,
}: {
  readonly path: string | null;
  readonly onConfirm: () => Promise<void>;
  readonly onCancel: () => void;
}) {
  const { t } = useTranslation();
  return (
    <AlertDialog
      open={path !== null}
      onOpenChange={(open) => {
        if (!open) onCancel();
      }}
    >
      <AlertDialogContent size="sm">
        <AlertDialogHeader>
          <AlertDialogTitle>{t('export.replace.title')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('export.replace.description', { path: path ?? '' })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction
            variant="danger"
            onClick={() => {
              void onConfirm();
            }}
          >
            {t('export.replace.confirm')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
