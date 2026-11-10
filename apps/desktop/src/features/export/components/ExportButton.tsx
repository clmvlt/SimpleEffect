import { Button, Icon, Tooltip } from '@se/ui';
import { Upload } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useShortcutAction, useShortcutHint } from '../../../shared/shortcuts';
import { useActiveSequence } from '../../timeline';
import { probeEncoders } from '../hooks/use-encoder-support';
import type { ExportFormDeps } from '../hooks/use-export-form';
import { ExportDialog } from './ExportDialog';

const DEFAULT_DEPS: ExportFormDeps = {
  probe: probeEncoders,
  now: () => new Date(),
  newId: () => crypto.randomUUID(),
};

/** Props of {@link ExportButton}. */
export interface ExportButtonProps {
  /** Overrides the encoder probe, clock and ids (tests). */
  readonly deps?: ExportFormDeps;
}

/** « Exporter » in the editor header (Ctrl/⌘+M): opens the export dialog of the active sequence. */
export function ExportButton({ deps = DEFAULT_DEPS }: ExportButtonProps) {
  const { t } = useTranslation();
  const sequence = useActiveSequence();
  const [open, setOpen] = useState(false);
  const hint = useShortcutHint('exportSequence');
  useShortcutAction(
    'exportSequence',
    () => {
      setOpen(true);
    },
    sequence !== null,
  );
  return (
    <>
      <Tooltip
        content={t('export.button.tooltip')}
        {...(hint === undefined ? {} : { shortcut: hint })}
      >
        <Button
          variant="primary"
          size="sm"
          disabled={sequence === null}
          onClick={() => {
            setOpen(true);
          }}
        >
          <Icon icon={Upload} size="sm" />
          {t('export.button.label')}
        </Button>
      </Tooltip>
      {open && sequence !== null && (
        <ExportDialog
          sequence={sequence}
          deps={deps}
          onClose={() => {
            setOpen(false);
          }}
        />
      )}
    </>
  );
}
