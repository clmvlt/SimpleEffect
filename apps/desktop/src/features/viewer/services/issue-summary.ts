import type { MediaIssue, MediaIssueKind } from '@se/engine/preview';

/** Issue kinds from the most to the least severe: the viewer reports the first one present. */
const SEVERITY: readonly MediaIssueKind[] = [
  'decodeFailed',
  'mezzanineFailed',
  'missing',
  'preparing',
];

/** The most severe kind of media problem in a frame and how many assets have it. */
export interface IssueSummary {
  readonly kind: MediaIssueKind;
  readonly count: number;
}

/** Summarises the frame's media issues for the viewer badge; `null` when there are none. */
export function summarizeIssues(issues: readonly MediaIssue[]): IssueSummary | null {
  for (const kind of SEVERITY) {
    const count = issues.filter((issue) => issue.kind === kind).length;
    if (count > 0) return { kind, count };
  }
  return null;
}
