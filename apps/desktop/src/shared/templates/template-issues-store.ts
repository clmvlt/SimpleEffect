import type { TemplateIssue } from '@se/engine/templates';
import type { ItemId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

interface TemplateIssuesState {
  /** The failing template instances of the frame on screen, by item. */
  readonly issues: Readonly<Record<ItemId, TemplateIssue>>;
}

/**
 * Motion template instances drawn as the error picture in the viewer's frame (prompt 24): the
 * viewer publishes them after each drawn frame, the inspector shows the message of the selected
 * instance.
 */
export const templateIssuesStore = createStore<TemplateIssuesState>()(() => ({ issues: {} }));

/** Replaces the issues with those of the frame just drawn. */
export function publishTemplateIssues(issues: readonly TemplateIssue[]): void {
  const current = templateIssuesStore.getState().issues;
  const next: Record<ItemId, TemplateIssue> = {};
  for (const issue of issues) next[issue.itemId] = issue;
  const same =
    Object.keys(current).length === issues.length &&
    issues.every((issue) => current[issue.itemId]?.message === issue.message);
  if (!same) templateIssuesStore.setState({ issues: next });
}

/** The issue of template instance `itemId` in the frame on screen, if it failed. */
export function useTemplateIssue(itemId: ItemId | null): TemplateIssue | null {
  return useStore(templateIssuesStore, (state) =>
    itemId === null ? null : (state.issues[itemId] ?? null),
  );
}
