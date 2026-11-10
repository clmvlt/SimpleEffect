import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** Tabs of « À propos », in order. */
export const ABOUT_TABS = ['application', 'licences', 'performance'] as const;

/** A tab of « À propos ». */
export type AboutTab = (typeof ABOUT_TABS)[number];

/** The tab named `value` (a tab value), or « Application » for anything else. */
export function aboutTabOf(value: string): AboutTab {
  return ABOUT_TABS.find((tab) => tab === value) ?? 'application';
}

/** Whether « À propos » is open and on which tab. */
export interface AboutDialogState {
  readonly open: boolean;
  readonly tab: AboutTab;
}

/** The dialog's state, so the status bar (or anything else) can open it. */
export const aboutDialogStore = createStore<AboutDialogState>()(() => ({
  open: false,
  tab: 'application',
}));

/** Opens « À propos » on `tab`. */
export function openAbout(tab: AboutTab = 'application'): void {
  aboutDialogStore.setState({ open: true, tab });
}

/** Closes « À propos » or shows another tab. */
export function updateAboutDialog(changes: Partial<AboutDialogState>): void {
  aboutDialogStore.setState(changes);
}

/** The dialog's state, re-rendering when it changes. */
export function useAboutDialog(): AboutDialogState {
  return useStore(aboutDialogStore);
}
