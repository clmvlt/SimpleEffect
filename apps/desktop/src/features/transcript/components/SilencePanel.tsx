import type { Flicks, ProjectId, SequenceId, SilenceCutParams, TrackId } from '@se/protocol';
import {
  Button,
  Checkbox,
  Label,
  RadioGroup,
  RadioGroupItem,
  ScrollArea,
  Spinner,
  Switch,
} from '@se/ui';
import { useId, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { clearCutPreview, useInOutPoints } from '../../timeline';
import type { SilenceCutRequest } from '../api';
import type { SmartCutActions } from '../hooks/use-smart-cuts';
import { useSmartCutSettings } from '../hooks/use-smart-cut-settings';
import { useSilencePreview } from '../hooks/use-silence-preview';
import { cutSeconds } from '../services/cut-format';
import {
  inOutRange,
  listenedTracks,
  sequenceEnd,
  soundTracksOf,
} from '../services/silence-options';
import { SilenceParamFields } from './SilenceParamFields';

/** Props of {@link SilencePanel}. */
export interface SilencePanelProps {
  readonly projectId: ProjectId;
  readonly sequenceId: SequenceId;
  readonly cuts: SmartCutActions;
}

type Span = 'sequence' | 'inOut';

const SPANS: readonly Span[] = ['sequence', 'inOut'];

/**
 * « Supprimer les silences » (prompt 22): threshold, shortest silence, padding, the span (in/out
 * or the whole sequence) and the sound tracks listened to; the cuts are previewed on the timeline
 * as they are set, then made as one undoable cut. The parameters become the defaults.
 */
export function SilencePanel({ projectId, sequenceId, cuts }: SilencePanelProps) {
  const { t } = useTranslation();
  const ids = useId();
  const { settings, save } = useSmartCutSettings();
  const [edited, setEdited] = useState<SilenceCutParams | null>(null);
  const [span, setSpan] = useState<Span>('sequence');
  const [chosen, setChosen] = useState<ReadonlySet<TrackId> | null>(null);
  const [allTracks, setAllTracks] = useState(false);
  const mirror = useProjectSelector((state) => state.mirror);
  const revision = mirror?.revision ?? 0;
  const { inPoint, outPoint } = useInOutPoints(sequenceId);
  const tracks = mirror === null ? [] : soundTracksOf(mirror, sequenceId);
  const end = mirror === null ? (0 as Flicks) : sequenceEnd(mirror, sequenceId);
  const marked = inOutRange(inPoint, outPoint, end);
  const params = edited ?? settings?.silence ?? null;
  const listened =
    chosen ?? new Set(tracks.filter((track) => track.byDefault).map((track) => track.id));
  const request: SilenceCutRequest | null =
    params === null
      ? null
      : {
          projectId,
          sequenceId,
          range: span === 'inOut' ? marked : null,
          params,
          trackIds: listenedTracks(tracks, chosen),
          allTracks,
        };
  const preview = useSilencePreview(request, revision);
  const count = preview.summary?.cuts.length ?? 0;

  const apply = async () => {
    if (request === null || settings === null) return;
    clearCutPreview();
    const done = await cuts.cutSilences(request);
    if (done && edited !== null) void save({ ...settings, silence: edited });
  };

  if (params === null) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Spinner label={t('editor.transcript.loading')} />
      </div>
    );
  }
  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="flex flex-col gap-4 p-3" data-silence-panel>
        <p className="text-xs text-text-muted">{t('editor.transcript.silences.intro')}</p>
        <SilenceParamFields params={params} onChange={setEdited} />
        <div className="flex flex-col gap-1.5">
          <span id={`${ids}-span`} className="text-xs font-medium">
            {t('editor.transcript.silences.span')}
          </span>
          <RadioGroup
            aria-labelledby={`${ids}-span`}
            value={span === 'inOut' && marked === null ? 'sequence' : span}
            onValueChange={(value) => {
              setSpan(value === 'inOut' ? 'inOut' : 'sequence');
            }}
          >
            {SPANS.map((choice) => (
              <div key={choice} className="flex items-center gap-2">
                <RadioGroupItem
                  id={`${ids}-${choice}`}
                  value={choice}
                  disabled={choice === 'inOut' && marked === null}
                />
                <Label htmlFor={`${ids}-${choice}`} className="text-xs">
                  {t(`editor.transcript.silences.spans.${choice}`)}
                </Label>
              </div>
            ))}
          </RadioGroup>
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-xs font-medium">{t('editor.transcript.silences.tracks')}</span>
          {tracks.map((track) => (
            <div key={track.id} className="flex items-center gap-2">
              <Checkbox
                id={`${ids}-${track.id}`}
                checked={listened.has(track.id)}
                onCheckedChange={(checked) => {
                  const next = new Set(listened);
                  if (checked === true) next.add(track.id);
                  else next.delete(track.id);
                  setChosen(next);
                }}
              />
              <Label htmlFor={`${ids}-${track.id}`} className="text-xs">
                {track.name}
              </Label>
            </div>
          ))}
          <p className="text-2xs text-text-subtle">{t('editor.transcript.silences.tracksHint')}</p>
        </div>
        <div className="flex items-center gap-2">
          <Switch id={`${ids}-all`} checked={allTracks} onCheckedChange={setAllTracks} />
          <Label htmlFor={`${ids}-all`} className="text-xs">
            {t('editor.transcript.silences.allTracks')}
          </Label>
        </div>
        <div className="flex flex-col gap-2 rounded-sm border border-border p-2" aria-live="polite">
          <span className="text-xs" data-silence-summary>
            {preview.errorKey !== null
              ? t(preview.errorKey)
              : preview.summary === null
                ? t('editor.transcript.silences.previewing')
                : count === 0
                  ? t('editor.transcript.silences.none')
                  : t('editor.transcript.cuts.preview', {
                      count,
                      seconds: cutSeconds(preview.summary.removedDuration),
                    })}
          </span>
          {(preview.summary?.unanalysedAssets.length ?? 0) > 0 && (
            <span className="text-2xs text-warning">
              {t('editor.transcript.silences.unanalysed', {
                count: preview.summary?.unanalysedAssets.length ?? 0,
              })}
            </span>
          )}
          <Button
            size="sm"
            variant="primary"
            disabled={count === 0 || preview.loading || cuts.busy}
            onClick={() => {
              void apply();
            }}
          >
            {t('editor.transcript.silences.apply', { count })}
          </Button>
        </div>
      </div>
    </ScrollArea>
  );
}
