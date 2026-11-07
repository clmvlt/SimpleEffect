import { IconButton, Tooltip } from '@se/ui';
import { Settings } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { SettingsDialog } from './SettingsDialog';

/** Props of {@link SettingsButton}. */
export interface SettingsButtonProps {
  /** The open project (the dialog shows its cache), or `null` on the home screen. */
  readonly projectId: string | null;
}

/** The gear button opening « Réglages ». */
export function SettingsButton({ projectId }: SettingsButtonProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const label = t('settings.open');
  return (
    <>
      <Tooltip content={label}>
        <IconButton
          icon={Settings}
          label={label}
          onClick={() => {
            setOpen(true);
          }}
        />
      </Tooltip>
      <SettingsDialog open={open} onOpenChange={setOpen} projectId={projectId} />
    </>
  );
}
