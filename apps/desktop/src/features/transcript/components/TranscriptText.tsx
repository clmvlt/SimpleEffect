import { cn } from '@se/ui';
import type { RefObject } from 'react';
import { useTranslation } from 'react-i18next';

import type { SequenceTranscript } from '../../../shared/transcript';
import type { WordRange, WordSelection } from '../hooks/use-word-selection';
import type { WordMatch } from '../services/word-search';
import { segmentClock } from '../services/transcript-format';

/** Confidence under which a word is underlined as unsure (`Transcript::LOW_CONFIDENCE`). */
const LOW_CONFIDENCE = 0.5;

/** Props of {@link TranscriptText}. */
export interface TranscriptTextProps {
  readonly transcript: SequenceTranscript;
  /** Receives the element holding the words (the playhead highlight looks words up in it). */
  readonly containerRef: RefObject<HTMLDivElement | null>;
  readonly selection: WordSelection;
  readonly matches: readonly WordMatch[];
  /** Index of the current match in `matches`. */
  readonly currentMatch: number | null;
}

function inRange(index: number, range: WordRange | null | undefined): boolean {
  return range !== null && range !== undefined && index >= range.first && index <= range.last;
}

/**
 * The words as paragraphs (one per segment, with its start time): click a word to move the
 * playhead, drag across words to mark them as in/out. Unsure words are underlined, search matches
 * highlighted, and the word under the playhead is marked by `useWordHighlight` (`data-current`).
 */
export function TranscriptText({
  transcript,
  containerRef,
  selection,
  matches,
  currentMatch,
}: TranscriptTextProps) {
  const { t } = useTranslation();
  const matched = new Set<number>();
  for (const match of matches) {
    for (let index = match.first; index <= match.last; index += 1) matched.add(index);
  }
  const current = currentMatch === null ? null : matches[currentMatch];
  return (
    <div
      ref={containerRef}
      role="document"
      aria-label={t('editor.transcript.textLabel')}
      className="flex flex-col gap-3 p-3 text-sm leading-relaxed text-text select-none"
      onPointerDown={selection.onPointerDown}
      onPointerMove={selection.onPointerMove}
      onPointerUp={selection.onPointerUp}
    >
      {transcript.segments.map((segment) => {
        const words = transcript.words.slice(
          segment.firstWord,
          segment.firstWord + segment.wordCount,
        );
        return (
          <p
            key={`${segment.itemId}-${String(segment.firstWord)}`}
            className="flex flex-wrap gap-x-1"
          >
            <span className="mr-1 text-2xs text-text-subtle tabular-nums">
              {segmentClock(segment.start)}
            </span>
            {words.map((word, offset) => {
              const index = segment.firstWord + offset;
              return (
                <span
                  key={index}
                  data-word-index={index}
                  data-unsure={word.confidence < LOW_CONFIDENCE ? '' : undefined}
                  data-selected={inRange(index, selection.range) ? '' : undefined}
                  data-match={matched.has(index) ? '' : undefined}
                  data-current-match={inRange(index, current) ? '' : undefined}
                  title={
                    word.confidence < LOW_CONFIDENCE ? t('editor.transcript.unsureWord') : undefined
                  }
                  className={cn(
                    'cursor-pointer rounded-sm px-0.5 hover:bg-hover',
                    'data-[unsure]:underline data-[unsure]:decoration-warning data-[unsure]:decoration-dotted',
                    'data-[match]:bg-warning-muted data-[current-match]:ring-1 data-[current-match]:ring-warning',
                    'data-[selected]:bg-info-muted data-[current]:bg-accent-muted data-[current]:text-text',
                  )}
                >
                  {word.text}
                </span>
              );
            })}
          </p>
        );
      })}
    </div>
  );
}
