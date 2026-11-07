import { useShortcutAction } from '../../shared/shortcuts';
import { ShortcutsDialog } from './ShortcutsDialog';
import { useShortcutsDialogStore } from './shortcuts-dialog-store';

/** Mounts the shortcuts help dialog for every screen and opens it with `?`. */
export function ShortcutsDialogHost() {
  const open = useShortcutsDialogStore((state) => state.open);
  const setOpen = useShortcutsDialogStore((state) => state.setOpen);
  useShortcutAction('showShortcuts', () => {
    setOpen(true);
  });
  return <ShortcutsDialog open={open} onOpenChange={setOpen} />;
}
