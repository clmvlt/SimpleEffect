import type { TemplateId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

interface CodeDialogState {
  /** The template whose code is shown, `null` when the dialog is closed. */
  readonly templateId: TemplateId | null;
}

/** The « Voir le code » dialog: opened from the inspector and the templates list. */
export const codeDialogStore = createStore<CodeDialogState>()(() => ({ templateId: null }));

/** Shows the code of `templateId` (read only). */
export function openTemplateCode(templateId: TemplateId): void {
  codeDialogStore.setState({ templateId });
}

/** Closes the code dialog. */
export function closeTemplateCode(): void {
  codeDialogStore.setState({ templateId: null });
}

/** The template whose code is shown, if any. */
export function useCodeDialogTemplate(): TemplateId | null {
  return useStore(codeDialogStore, (state) => state.templateId);
}
