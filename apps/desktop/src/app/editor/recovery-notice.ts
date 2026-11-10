import type { ProjectRecovery } from '@se/protocol';

/** The toast telling the user what opening the project repaired. */
export type RecoveryNotice =
  /** The journal was replayed over the last snapshot after an unexpected stop. */
  | { readonly kind: 'replayed'; readonly count: number }
  /** `project.json` was unreadable: the newest valid backup was restored. */
  | {
      readonly kind: 'restored';
      readonly backup: string;
      readonly replayed: number;
      readonly lost: number;
    };

/**
 * The notice to show for `recovery` (the session's report, `ProjectRecovery`), or `null` when
 * there is nothing worth telling. A restore wins over a replay: the user must learn that their
 * main file was set aside.
 */
export function recoveryNotice(recovery: ProjectRecovery | null): RecoveryNotice | null {
  if (recovery === null) return null;
  if (recovery.restoredFrom !== null) {
    return {
      kind: 'restored',
      backup: recovery.restoredFrom,
      replayed: recovery.replayedChanges,
      lost: recovery.lostChanges,
    };
  }
  return recovery.replayedChanges > 0
    ? { kind: 'replayed', count: recovery.replayedChanges }
    : null;
}
