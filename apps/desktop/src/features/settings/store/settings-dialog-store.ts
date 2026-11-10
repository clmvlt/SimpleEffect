import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** Sections of « Réglages », in tab order. */
export const SETTINGS_SECTIONS = [
  'media',
  'ai',
  'transcription',
  'brandKits',
  'performance',
] as const;

/** A section of « Réglages ». */
export type SettingsSection = (typeof SETTINGS_SECTIONS)[number];

/** The section named `value` (a tab value), or « Médias » for anything else. */
export function settingsSectionOf(value: string): SettingsSection {
  return SETTINGS_SECTIONS.find((section) => section === value) ?? 'media';
}

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

/**
 * Opens « Réglages » on `section` (the status bar opens « IA »; the transcript panel opens
 * « Transcription » when no model is installed).
 */
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
