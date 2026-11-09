import type { ItemId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { showSidePanelTab } from '../side-panel';

/**
 * Requests to open the AI composer from anywhere (Ctrl/⌘+I, « Refaire avec l'IA » on a timeline
 * item): the AI panel is shown, and the AI feature focuses its composer and applies the request.
 */
export interface ComposerRequest {
  /** Changes with every request, so the same request twice is seen twice. */
  readonly nonce: number;
  /** « Refaire avec l'IA »: the item to redo (its range and track become the scope). */
  readonly itemId: ItemId | null;
  /** Text put in the composer (« Demander une modification à l'IA » of a template), if any. */
  readonly draft: string | null;
}

interface ComposerRequestState {
  readonly request: ComposerRequest | null;
}

/** The latest request (vanilla store: the AI feature also reads it outside React). */
export const composerRequestStore = createStore<ComposerRequestState>()(() => ({ request: null }));

let nonce = 0;

/**
 * Shows the AI panel and asks its composer to take the focus, about `itemId` if given, with
 * `draft` as its text if given.
 */
export function openComposer(itemId: ItemId | null = null, draft: string | null = null): void {
  nonce += 1;
  composerRequestStore.setState({ request: { nonce, itemId, draft } });
  showSidePanelTab('ai');
}

/** The latest composer request, if any. */
export function useComposerRequest(): ComposerRequest | null {
  return useStore(composerRequestStore, (state) => state.request);
}
