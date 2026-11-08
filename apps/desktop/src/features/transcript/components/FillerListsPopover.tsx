import type { Flicks, SmartCutSettings } from '@se/protocol';
import {
  Button,
  Field,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Slider,
  Textarea,
} from '@se/ui';
import { ListFilter } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND, flicksToSeconds } from '../../../shared/time';
import { cutSeconds } from '../services/cut-format';
import { editedLanguages, withTexts, wordsText } from '../services/filler-lists';
import { languageName } from '../services/transcript-format';

/** Props of {@link FillerListsPopover}. */
export interface FillerListsPopoverProps {
  readonly settings: SmartCutSettings;
  readonly onSave: (settings: SmartCutSettings) => void;
}

/**
 * The filler lists per language (comma-separated words or phrases) and the longest occurrence
 * detected, saved as app settings (agents' `remove_fillers` uses them too).
 */
export function FillerListsPopover({ settings, onSave }: FillerListsPopoverProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const languages = editedLanguages(settings.fillers);
  const initialTexts = () =>
    Object.fromEntries(
      languages.map((language) => [
        language,
        wordsText(settings.fillers.find((list) => list.language === language)),
      ]),
    );
  const [texts, setTexts] = useState<Record<string, string>>(initialTexts);
  const [maxSeconds, setMaxSeconds] = useState(flicksToSeconds(settings.fillerMaxDuration));
  return (
    <Popover
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (next) {
          setTexts(initialTexts());
          setMaxSeconds(flicksToSeconds(settings.fillerMaxDuration));
        }
      }}
    >
      <PopoverTrigger asChild>
        <IconButton
          icon={ListFilter}
          size="xs"
          variant="ghost"
          label={t('editor.transcript.fillers.lists')}
        />
      </PopoverTrigger>
      <PopoverContent className="flex w-80 flex-col gap-3">
        {languages.map((language) => (
          <Field key={language} label={languageName(language)}>
            <Textarea
              rows={2}
              value={texts[language] ?? ''}
              onChange={(event) => {
                setTexts({ ...texts, [language]: event.target.value });
              }}
            />
          </Field>
        ))}
        <p className="text-2xs text-text-muted">{t('editor.transcript.fillers.listsHint')}</p>
        <div className="flex flex-col gap-1">
          <span className="text-xs">
            {t('editor.transcript.fillers.maxDuration', {
              seconds: cutSeconds((maxSeconds * FLICKS_PER_SECOND) as Flicks),
            })}
          </span>
          <Slider
            thumbLabels={[t('editor.transcript.fillers.maxDurationLabel')]}
            min={0.2}
            max={5}
            step={0.1}
            value={[maxSeconds]}
            onValueChange={([next]) => {
              if (next !== undefined) setMaxSeconds(next);
            }}
          />
        </div>
        <Button
          size="sm"
          className="self-end"
          onClick={() => {
            onSave({
              ...settings,
              fillers: withTexts(languages, texts),
              fillerMaxDuration: Math.round(maxSeconds * FLICKS_PER_SECOND) as Flicks,
            });
            setOpen(false);
          }}
        >
          {t('editor.transcript.fillers.saveLists')}
        </Button>
      </PopoverContent>
    </Popover>
  );
}
