import { IconButton, Tooltip } from '@se/ui';
import { Keyboard } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useShortcutHint } from '../../shared/shortcuts';
import { useShortcutsDialogStore } from './shortcuts-dialog-store';

/** Header button opening the « Raccourcis clavier » help dialog. */
export function ShortcutsButton() {
  const { t } = useTranslation();
  const setOpen = useShortcutsDialogStore((state) => state.setOpen);
  const hint = useShortcutHint('showShortcuts');
  return (
    <Tooltip content={t('shortcuts.open')} {...(hint === undefined ? {} : { shortcut: hint })}>
      <IconButton
        icon={Keyboard}
        label={t('shortcuts.open')}
        onClick={() => {
          setOpen(true);
        }}
      />
    </Tooltip>
  );
}
