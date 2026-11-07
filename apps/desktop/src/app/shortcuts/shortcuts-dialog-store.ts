import { create } from 'zustand';

/** Whether the « Raccourcis clavier » help dialog is open (opened by `?` or a header button). */
export const useShortcutsDialogStore = create<{
  readonly open: boolean;
  readonly setOpen: (open: boolean) => void;
}>((set) => ({
  open: false,
  setOpen: (open) => {
    set({ open });
  },
}));
