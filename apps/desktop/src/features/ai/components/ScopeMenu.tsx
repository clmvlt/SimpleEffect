import type { SequenceEntity, TrackId } from '@se/protocol';
import {
  Checkbox,
  IconButton,
  Label,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@se/ui';
import { SlidersHorizontal } from '@se/ui/icons';
import { useId } from 'react';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import type { ComposerScope } from '../hooks/use-composer-scope';
import { updateComposer, useComposer } from '../store/composer-store';

/** Props of {@link ScopeMenu}. */
export interface ScopeMenuProps {
  readonly scope: ComposerScope;
  readonly disabled?: boolean | undefined;
}

/**
 * The composer's « Portée » popover: the sequence, the whole sequence instead of the automatic
 * range, and the tracks (every one, or some).
 */
export function ScopeMenu({ scope, disabled }: ScopeMenuProps) {
  const { t } = useTranslation();
  const ids = useId();
  const sequences = useProjectSelector((state) => state.mirror?.sequences);
  const order = useProjectSelector((state) => state.mirror?.sequenceOrder);
  const wholeSequence = useComposer((state) => state.wholeSequence);
  const chosenTracks = useComposer((state) => state.chosenTracks);
  const list = (order ?? []).flatMap((id): SequenceEntity[] => {
    const sequence = sequences?.[id];
    return sequence === undefined ? [] : [sequence];
  });
  const scopeTracks = scope.derived.scope.tracks;
  const toggleTrack = (trackId: TrackId, checked: boolean) => {
    const current = scopeTracks ?? scope.tracks.map((track) => track.id);
    const next = checked
      ? [...current.filter((id) => id !== trackId), trackId]
      : current.filter((id) => id !== trackId);
    updateComposer({ chosenTracks: next.length === scope.tracks.length ? 'all' : next });
  };
  return (
    <Popover>
      <PopoverTrigger asChild>
        <IconButton
          icon={SlidersHorizontal}
          label={t('ai.composer.scope.edit')}
          size="xs"
          disabled={disabled}
        />
      </PopoverTrigger>
      <PopoverContent align="start" className="flex w-72 flex-col gap-3 text-xs">
        <div className="flex flex-col gap-1">
          <Label htmlFor={`${ids}-sequence`}>{t('ai.composer.scope.sequenceLabel')}</Label>
          <Select
            value={scope.sequence.id}
            onValueChange={(value) => {
              updateComposer({ sequenceId: value, chosenTracks: null });
            }}
          >
            <SelectTrigger id={`${ids}-sequence`} size="xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {list.map((sequence) => (
                <SelectItem key={sequence.id} value={sequence.id}>
                  {sequence.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-center justify-between gap-2">
          <Label htmlFor={`${ids}-whole`}>{t('ai.composer.scope.wholeSequenceOption')}</Label>
          <Switch
            id={`${ids}-whole`}
            checked={wholeSequence}
            onCheckedChange={(checked) => {
              updateComposer({ wholeSequence: checked });
            }}
          />
        </div>
        <fieldset className="flex flex-col gap-1.5">
          <legend className="pb-1 font-medium text-text">
            {t('ai.composer.scope.tracksLabel')}
          </legend>
          <div className="flex items-center gap-2">
            <Checkbox
              id={`${ids}-all`}
              checked={scopeTracks === null}
              onCheckedChange={(checked) => {
                updateComposer({ chosenTracks: checked === true ? 'all' : null });
              }}
            />
            <Label htmlFor={`${ids}-all`}>{t('ai.composer.scope.allTracks')}</Label>
          </div>
          {scope.tracks.map((track) => (
            <div key={track.id} className="flex items-center gap-2 pl-4">
              <Checkbox
                id={`${ids}-${track.id}`}
                checked={scopeTracks === null || scopeTracks.includes(track.id)}
                onCheckedChange={(checked) => {
                  toggleTrack(track.id, checked === true);
                }}
              />
              <Label htmlFor={`${ids}-${track.id}`}>{track.name}</Label>
            </div>
          ))}
          {chosenTracks !== null && (
            <button
              type="button"
              className="self-start pt-1 text-text-muted underline-offset-2 hover:underline"
              onClick={() => {
                updateComposer({ chosenTracks: null });
              }}
            >
              {t('ai.composer.scope.automaticTracks')}
            </button>
          )}
        </fieldset>
      </PopoverContent>
    </Popover>
  );
}
