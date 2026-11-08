import { IconButton, Tooltip } from '@se/ui';
import { Settings } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import {
  openSettings,
  updateSettingsDialog,
  useSettingsDialog,
} from '../store/settings-dialog-store';
import { SettingsDialog } from './SettingsDialog';

/** Props of {@link SettingsButton}. */
export interface SettingsButtonProps {
  /** The open project (the dialog shows its cache), or `null` on the home screen. */
  readonly projectId: string | null;
}

/**
 * The gear button opening « Réglages ». Its dialog follows the settings dialog store, so
 * `openSettings('ai')` (the status bar) opens it too; one button is mounted at a time (home or
 * editor header).
 */
export function SettingsButton({ projectId }: SettingsButtonProps) {
  const { t } = useTranslation();
  const { open, section } = useSettingsDialog();
  const label = t('settings.open');
  return (
    <>
      <Tooltip content={label}>
        <IconButton
          icon={Settings}
          label={label}
          onClick={() => {
            openSettings('media');
          }}
        />
      </Tooltip>
      <SettingsDialog
        open={open}
        onOpenChange={(next) => {
          updateSettingsDialog({ open: next });
        }}
        section={section}
        onSectionChange={(next) => {
          updateSettingsDialog({ section: next });
        }}
        projectId={projectId}
      />
    </>
  );
}
