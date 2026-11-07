import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** Sections of « Réglages ». */
export type SettingsSection = 'media' | 'ai';

/** Whether « Réglages » is open and on which section. */
export interface SettingsDialogState {
  readonly open: boolean;
  readonly section: SettingsSection;
}

/** The dialog's state, so other parts of the app (the status bar) can open a section. */
export const settingsDialogStore = createStore<SettingsDialogState>()(() => ({
  open: false,
  section: 'media',
}));

/** Opens « Réglages » on `section`. */
export function openSettings(section: SettingsSection = 'media'): void {
  settingsDialogStore.setState({ open: true, section });
}

/** Closes « Réglages » or shows another section. */
export function updateSettingsDialog(changes: Partial<SettingsDialogState>): void {
  settingsDialogStore.setState(changes);
}

/** The dialog's state, re-rendering when it changes. */
export function useSettingsDialog(): SettingsDialogState {
  return useStore(settingsDialogStore);
}
