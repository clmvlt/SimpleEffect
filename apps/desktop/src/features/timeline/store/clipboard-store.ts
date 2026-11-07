import { createStore } from 'zustand/vanilla';

import type { Clipboard } from '../services/clipboard';

/** The timeline's internal clipboard (copies of items, for the project they come from). */
export interface ClipboardState {
  readonly clipboard: Clipboard | null;
}

/** The clipboard store. */
export const clipboardStore = createStore<ClipboardState>()(() => ({ clipboard: null }));

/** Replaces what is copied. */
export function setClipboard(clipboard: Clipboard | null): void {
  clipboardStore.setState({ clipboard });
}

/** What is copied for project `projectId`, if anything. */
export function clipboardFor(projectId: string): Clipboard | null {
  const { clipboard } = clipboardStore.getState();
  return clipboard?.projectId === projectId ? clipboard : null;
}
