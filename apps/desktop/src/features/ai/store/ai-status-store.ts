import type { AiStatus } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

/** What the app knows about the AI provider. */
export interface AiStatusState {
  /** `null` until the core answered (or while disconnected). */
  readonly status: AiStatus | null;
  /** A refresh asked by the user is running. */
  readonly refreshing: boolean;
}

/** The AI status store (fed by `services/watch-ai-status.ts`, one app-wide copy). */
export const aiStatusStore = createStore<AiStatusState>()(() => ({
  status: null,
  refreshing: false,
}));

/** Replaces the known status. */
export function setAiStatus(status: AiStatus | null): void {
  aiStatusStore.setState({ status });
}

/** Marks a user refresh as running or done. */
export function setAiRefreshing(refreshing: boolean): void {
  aiStatusStore.setState({ refreshing });
}
