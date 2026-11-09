import type { CaptionStyleId, LibraryTemplate, MotionTemplate } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** What the library information dialog is doing. */
export type LibraryDialogRequest =
  | {
      readonly kind: 'saveTemplate';
      readonly template: MotionTemplate;
      /** The library entry it is a copy of: the save is a new version of it. */
      readonly linked: LibraryTemplate | null;
    }
  | { readonly kind: 'saveCaptionStyle'; readonly styleId: CaptionStyleId; readonly name: string }
  | { readonly kind: 'rename'; readonly entry: LibraryTemplate };

/** The open request (`null` when the dialog is closed). */
export const libraryDialogStore = createStore<{ readonly request: LibraryDialogRequest | null }>()(
  () => ({ request: null }),
);

/**
 * Opens « Enregistrer dans la bibliothèque » for a project template, or « Enregistrer une
 * nouvelle version » when it is a copy of library entry `linked` (its information prefilled).
 */
export function openSaveTemplate(
  template: MotionTemplate,
  linked: LibraryTemplate | null = null,
): void {
  libraryDialogStore.setState({ request: { kind: 'saveTemplate', template, linked } });
}

/** Opens « Enregistrer dans la bibliothèque » for a caption style (`name`: a suggestion). */
export function openSaveCaptionStyle(styleId: CaptionStyleId, name: string): void {
  libraryDialogStore.setState({ request: { kind: 'saveCaptionStyle', styleId, name } });
}

/** Opens « Renommer » for a library entry. */
export function openRenameEntry(entry: LibraryTemplate): void {
  libraryDialogStore.setState({ request: { kind: 'rename', entry } });
}

/** Closes the dialog. */
export function closeLibraryDialog(): void {
  libraryDialogStore.setState({ request: null });
}

/** The open request. */
export function useLibraryDialogRequest(): LibraryDialogRequest | null {
  return useStore(libraryDialogStore, (state) => state.request);
}
