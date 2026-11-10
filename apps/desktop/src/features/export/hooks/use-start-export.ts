import type { SequenceEntity } from '@se/protocol';
import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector, selectProjectId } from '../../../shared/project-store';
import { exportsApi } from '../api';
import type { ExportForm } from './use-export-form';

/** Starting an export from the dialog, with the « replace the file? » question. */
export interface StartExportFlow {
  readonly starting: boolean;
  /** The file an export would replace, while the user is asked. */
  readonly replacing: string | null;
  /** Starts the export, or asks first when its file exists. */
  readonly start: () => Promise<void>;
  readonly confirmReplace: () => Promise<void>;
  readonly cancelReplace: () => void;
}

/**
 * Starts the export the form describes: the destination is joined by the main process (path
 * rules of the platform), an existing file is replaced only once confirmed, then the core
 * queues it and the folder and file name template are remembered. `onStarted` closes the dialog.
 */
export function useStartExport(
  form: ExportForm,
  sequence: SequenceEntity,
  onStarted: () => void,
): StartExportFlow {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const projectId = useProjectSelector(selectProjectId);
  const { t } = useTranslation();
  const [starting, setStarting] = useState(false);
  const [replacing, setReplacing] = useState<string | null>(null);

  const launch = async (path: string) => {
    if (projectId === null) return;
    const { span, rangeKind } = form;
    await exportsApi(client).start({
      projectId,
      sequenceId: sequence.id,
      settings: form.plan.settings,
      ...(rangeKind === 'inOut'
        ? { range: { start: span.start, duration: (span.end - span.start) as typeof span.end } }
        : {}),
      path,
      presetName: form.presetName,
    });
    await form.rememberDestination();
    toast.success(t('export.started', { name: form.plan.fileName }));
    onStarted();
  };

  const run = async (step: () => Promise<void>) => {
    setStarting(true);
    try {
      await step();
    } catch (error) {
      toast.error(t(errorMessageKey(error)));
    } finally {
      setStarting(false);
    }
  };

  return {
    starting,
    replacing,
    start: () =>
      run(async () => {
        const target = await bridge.resolveFilePath({
          folder: form.directory,
          fileName: form.plan.fileName,
        });
        if (target.exists) setReplacing(target.path);
        else await launch(target.path);
      }),
    confirmReplace: () =>
      run(async () => {
        const path = replacing;
        setReplacing(null);
        if (path !== null) await launch(path);
      }),
    cancelReplace: () => {
      setReplacing(null);
    },
  };
}
