import type { Conflict, DomainErrorCode, ItemId } from '@se/protocol';

/** One thing a revert would leave as it is, for the confirmation dialog. */
export type ConflictLine =
  | {
      readonly kind: 'changed';
      readonly key: string;
      readonly itemId: ItemId;
      /** Removed since (by someone else), rather than changed. */
      readonly removed: boolean;
    }
  | { readonly kind: 'foreign'; readonly key: string; readonly count: number }
  | { readonly kind: 'cannotApply'; readonly key: string; readonly code: DomainErrorCode };

/** The lines of a revert report's conflicts, in order. */
export function conflictLines(conflicts: readonly Conflict[]): ConflictLine[] {
  return conflicts.map((conflict, index): ConflictLine => {
    const key = String(index);
    switch (conflict.kind) {
      case 'itemChanged':
        return {
          kind: 'changed',
          key,
          itemId: conflict.item,
          removed: conflict.currentRevision === null,
        };
      case 'foreignItems':
        return { kind: 'foreign', key, count: conflict.items.length };
      case 'cannotApply':
        return { kind: 'cannotApply', key, code: conflict.code };
    }
  });
}
