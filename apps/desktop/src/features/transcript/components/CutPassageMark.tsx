import { IconButton } from '@se/ui';
import { Undo2 } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { cutSeconds, passageExcerpt } from '../services/cut-format';
import type { CutPassage } from '../services/cut-passages';

/** Props of {@link CutPassageMark}. */
export interface CutPassageMarkProps {
  readonly passage: CutPassage;
  readonly onRestore: (passage: CutPassage) => void;
}

/**
 * A passage cut out of the sequence, where it was: its words struck through and greyed (a cut
 * silence shows « ⋯ 1,2 s »), with « Restaurer » putting it back with a ripple trim.
 */
export function CutPassageMark({ passage, onRestore }: CutPassageMarkProps) {
  const { t } = useTranslation();
  const seconds = cutSeconds(passage.duration);
  const silence = passage.words.length === 0;
  return (
    <span
      data-cut-passage={passage.at}
      title={t('editor.transcript.passages.title', { seconds })}
      className="inline-flex items-center gap-0.5 rounded-sm bg-panel-raised px-1 text-text-subtle"
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
    >
      <span className={silence ? 'text-2xs tabular-nums' : 'line-through decoration-danger/70'}>
        {silence
          ? t('editor.transcript.passages.silence', { seconds })
          : passageExcerpt(passage.words)}
      </span>
      <IconButton
        icon={Undo2}
        size="xs"
        variant="ghost"
        label={t('editor.transcript.passages.restore')}
        onClick={() => {
          onRestore(passage);
        }}
      />
    </span>
  );
}
