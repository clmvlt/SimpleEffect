import type { SequenceId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** The sequence dialog shown: change the format, create a version, or confirm a removal. */
export type SequenceDialog =
  | { readonly kind: 'format'; readonly sequenceId: SequenceId }
  | { readonly kind: 'version'; readonly sequenceId: SequenceId }
  | { readonly kind: 'remove'; readonly sequenceId: SequenceId };

interface SequenceDialogState {
  readonly dialog: SequenceDialog | null;
}

/** Which sequence dialog is open (one at a time; the host is `SequenceDialogs`). */
export const sequenceDialogStore = createStore<SequenceDialogState>()(() => ({ dialog: null }));

/** Opens a sequence dialog (from a tab menu, a toolbar…). */
export function openSequenceDialog(dialog: SequenceDialog): void {
  sequenceDialogStore.setState({ dialog });
}

/** Closes the open sequence dialog. */
export function closeSequenceDialog(): void {
  sequenceDialogStore.setState({ dialog: null });
}

/** The open sequence dialog, or `null`. */
export function useSequenceDialog(): SequenceDialog | null {
  return useStore(sequenceDialogStore, (state) => state.dialog);
}
