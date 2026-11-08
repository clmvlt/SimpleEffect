import type { FillerOccurrence, Flicks, ProjectId, SequenceId } from '@se/protocol';
import { Button, Checkbox, EmptyState, IconButton, ScrollArea, Spinner } from '@se/ui';
import { MessageCircleOff, Play } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { requestPlayRange } from '../../../shared/playback';
import { setPlayheadTime } from '../../timeline';
import { useFillers } from '../hooks/use-fillers';
import type { SmartCutActions } from '../hooks/use-smart-cuts';
import { useSmartCutSettings } from '../hooks/use-smart-cut-settings';
import { segmentClock } from '../services/transcript-format';
import { FillerListsPopover } from './FillerListsPopover';

/** Heard before and after a filler when listening to it: 0.4 s. */
const LISTEN_MARGIN = 282_240_000;

/** Props of {@link FillerPanel}. */
export interface FillerPanelProps {
  readonly projectId: ProjectId;
  readonly sequenceId: SequenceId;
  readonly cuts: SmartCutActions;
}

/** The key of an occurrence (its start is unique in a sequence). */
function keyOf(occurrence: FillerOccurrence): string {
  return String(occurrence.start);
}

/**
 * « Hésitations » (prompt 22): the filler words found in the sequence's transcript, each with a
 * box (ticked by default), its time and « Écouter »; « Supprimer la sélection » cuts the ticked
 * ones as one undoable cut. The lists per language are edited from the header.
 */
export function FillerPanel({ projectId, sequenceId, cuts }: FillerPanelProps) {
  const { t } = useTranslation();
  const { settings, save } = useSmartCutSettings();
  const listsKey = settings === null ? '' : JSON.stringify(settings.fillers);
  const search = useFillers(projectId, sequenceId, listsKey);
  const [unticked, setUnticked] = useState<ReadonlySet<string>>(new Set());
  const occurrences = search.occurrences ?? [];
  const ticked = occurrences.filter((occurrence) => !unticked.has(keyOf(occurrence)));
  const toggle = (occurrence: FillerOccurrence, checked: boolean) => {
    const next = new Set(unticked);
    if (checked) next.delete(keyOf(occurrence));
    else next.add(keyOf(occurrence));
    setUnticked(next);
  };
  const listen = (occurrence: FillerOccurrence) => {
    requestPlayRange({
      sequenceId,
      start: Math.max(0, occurrence.start - LISTEN_MARGIN) as Flicks,
      end: (occurrence.end + LISTEN_MARGIN) as Flicks,
    });
  };
  const remove = () => {
    const ranges = ticked.map((occurrence) => ({
      start: occurrence.start,
      duration: (occurrence.end - occurrence.start) as Flicks,
    }));
    void cuts.cutWords(ranges, 'fillers').then((done) => {
      if (done) setUnticked(new Set());
    });
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col" data-filler-panel>
      <div className="flex shrink-0 items-center justify-between gap-2 border-b border-border p-2">
        <span className="text-xs text-text-muted" aria-live="polite">
          {search.occurrences === null
            ? t('editor.transcript.fillers.searching')
            : t('editor.transcript.fillers.found', { count: occurrences.length })}
        </span>
        <div className="flex items-center gap-1">
          <Button
            size="xs"
            variant="ghost"
            disabled={occurrences.length === 0}
            onClick={() => {
              setUnticked(
                ticked.length === occurrences.length ? new Set(occurrences.map(keyOf)) : new Set(),
              );
            }}
          >
            {t(
              ticked.length === occurrences.length
                ? 'editor.transcript.fillers.tickNone'
                : 'editor.transcript.fillers.tickAll',
            )}
          </Button>
          {settings !== null && (
            <FillerListsPopover
              settings={settings}
              onSave={(next) => {
                void save(next);
              }}
            />
          )}
        </div>
      </div>
      {search.untranscribed.length > 0 && (
        <p className="shrink-0 px-3 pt-2 text-2xs text-warning">
          {t('editor.transcript.fillers.untranscribed', { count: search.untranscribed.length })}
        </p>
      )}
      {search.occurrences === null ? (
        <div className="flex flex-1 items-center justify-center">
          <Spinner label={t('editor.transcript.fillers.searching')} />
        </div>
      ) : occurrences.length === 0 ? (
        <EmptyState
          icon={MessageCircleOff}
          title={t(search.errorKey ?? 'editor.transcript.fillers.none')}
          description={t('editor.transcript.fillers.noneHint')}
        />
      ) : (
        <ScrollArea className="min-h-0 flex-1">
          <ul className="flex flex-col p-1">
            {occurrences.map((occurrence) => {
              const key = keyOf(occurrence);
              const id = `filler-${key}`;
              return (
                <li
                  key={key}
                  data-filler={key}
                  className="flex items-center gap-2 rounded-sm px-2 py-1 hover:bg-hover"
                >
                  <Checkbox
                    id={id}
                    checked={!unticked.has(key)}
                    aria-label={t('editor.transcript.fillers.tick', {
                      text: occurrence.text,
                      time: segmentClock(occurrence.start),
                    })}
                    onCheckedChange={(checked) => {
                      toggle(occurrence, checked === true);
                    }}
                  />
                  <button
                    type="button"
                    className="flex min-w-0 flex-1 items-center gap-2 text-left text-xs"
                    onClick={() => {
                      setPlayheadTime(sequenceId, occurrence.start);
                    }}
                  >
                    <span className="text-2xs text-text-subtle tabular-nums">
                      {segmentClock(occurrence.start)}
                    </span>
                    <span className="truncate">{occurrence.text}</span>
                  </button>
                  <IconButton
                    icon={Play}
                    size="xs"
                    variant="ghost"
                    label={t('editor.transcript.fillers.listen', { text: occurrence.text })}
                    onClick={() => {
                      listen(occurrence);
                    }}
                  />
                </li>
              );
            })}
          </ul>
        </ScrollArea>
      )}
      <div className="shrink-0 border-t border-border p-2">
        <Button
          size="sm"
          variant="primary"
          className="w-full"
          disabled={ticked.length === 0 || cuts.busy}
          onClick={remove}
        >
          {t('editor.transcript.fillers.remove', { count: ticked.length })}
        </Button>
      </div>
    </div>
  );
}
