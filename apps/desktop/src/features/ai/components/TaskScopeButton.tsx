import type { Task } from '@se/protocol';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { useActiveSequence } from '../../timeline';
import { shortRange } from '../services/short-time';

/** Props of {@link TaskScopeButton}. */
export interface TaskScopeButtonProps {
  readonly task: Task;
  readonly onReveal: () => void;
}

/**
 * The scope of a task in words (« 00:51 → 00:54 · V2, V3 », « Toute la séquence », with the
 * sequence's name when it is not the one shown); a click shows that range on the timeline.
 */
export function TaskScopeButton({ task, onReveal }: TaskScopeButtonProps) {
  const { t } = useTranslation();
  const active = useActiveSequence();
  const sequence = useProjectSelector((state) => state.mirror?.sequences[task.scope.sequenceId]);
  const tracks = useProjectSelector((state) => state.mirror?.tracks);
  const { range, tracks: trackIds } = task.scope;
  const parts = [range === null ? t('ai.composer.scope.wholeSequence') : shortRange(range)];
  if (trackIds !== null) {
    parts.push(trackIds.map((id) => tracks?.[id]?.name ?? '?').join(', '));
  }
  if (sequence !== undefined && sequence.id !== active?.id) parts.push(sequence.name);
  return (
    <button
      type="button"
      title={t('ai.tasks.revealScope')}
      className="timecode text-left underline-offset-2 hover:text-text hover:underline"
      onClick={onReveal}
    >
      {parts.join(' · ')}
    </button>
  );
}
