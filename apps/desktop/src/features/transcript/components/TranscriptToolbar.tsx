import { IconButton, Input, ToggleGroup, ToggleGroupItem } from '@se/ui';
import { ChevronDown, ChevronUp } from '@se/ui/icons';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import type { TranscriptMode } from '../hooks/use-transcript-model';

/** Props of {@link TranscriptToolbar}. */
export interface TranscriptToolbarProps {
  readonly mode: TranscriptMode;
  readonly onModeChange: (mode: TranscriptMode) => void;
  readonly query: string;
  readonly onQueryChange: (query: string) => void;
  readonly matchCount: number;
  /** Index of the current match, or `null`. */
  readonly currentMatch: number | null;
  readonly onStep: (step: 1 | -1) => void;
  /** The project settings button (language, vocabulary). */
  readonly settings: ReactNode;
}

/** Mode (sequence or selected media), search with previous/next match, and project settings. */
export function TranscriptToolbar({
  mode,
  onModeChange,
  query,
  onQueryChange,
  matchCount,
  currentMatch,
  onStep,
  settings,
}: TranscriptToolbarProps) {
  const { t } = useTranslation();
  return (
    <div className="flex shrink-0 flex-col gap-2 border-b border-border p-2">
      <div className="flex items-center justify-between gap-2">
        <ToggleGroup
          type="single"
          variant="segmented"
          size="xs"
          value={mode}
          aria-label={t('editor.transcript.modeLabel')}
          onValueChange={(value) => {
            if (value === 'sequence' || value === 'media') onModeChange(value);
          }}
        >
          <ToggleGroupItem value="sequence">
            {t('editor.transcript.modes.sequence')}
          </ToggleGroupItem>
          <ToggleGroupItem value="media">{t('editor.transcript.modes.media')}</ToggleGroupItem>
        </ToggleGroup>
        {settings}
      </div>
      <div className="flex items-center gap-1">
        <Input
          size="xs"
          type="search"
          value={query}
          placeholder={t('editor.transcript.searchPlaceholder')}
          aria-label={t('editor.transcript.searchLabel')}
          className="min-w-0 flex-1"
          onChange={(event) => {
            onQueryChange(event.target.value);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              onStep(event.shiftKey ? -1 : 1);
            }
          }}
        />
        {query.trim() !== '' && (
          <span className="shrink-0 text-2xs text-text-muted tabular-nums" aria-live="polite">
            {matchCount === 0
              ? t('editor.transcript.noMatch')
              : t('editor.transcript.matchPosition', {
                  position: (currentMatch ?? 0) + 1,
                  count: matchCount,
                })}
          </span>
        )}
        <IconButton
          icon={ChevronUp}
          size="xs"
          variant="ghost"
          label={t('editor.transcript.previousMatch')}
          disabled={matchCount === 0}
          onClick={() => {
            onStep(-1);
          }}
        />
        <IconButton
          icon={ChevronDown}
          size="xs"
          variant="ghost"
          label={t('editor.transcript.nextMatch')}
          disabled={matchCount === 0}
          onClick={() => {
            onStep(1);
          }}
        />
      </div>
    </div>
  );
}
