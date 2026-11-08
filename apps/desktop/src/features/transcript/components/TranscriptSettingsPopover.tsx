import type { TranscriptionSettings } from '@se/protocol';
import {
  Button,
  Field,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
} from '@se/ui';
import { Languages } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  cleanVocabulary,
  languageName,
  TRANSCRIPTION_LANGUAGES,
} from '../services/transcript-format';

/** The select value of automatic detection. */
const DETECT = 'auto';

/** Props of {@link TranscriptSettingsPopover}. */
export interface TranscriptSettingsPopoverProps {
  readonly settings: TranscriptionSettings;
  readonly onSave: (settings: TranscriptionSettings) => void;
}

/**
 * The project's transcription language (or detection) and vocabulary (names the model should
 * spell this way), saved as one undoable change. They apply to the next transcriptions.
 */
export function TranscriptSettingsPopover({ settings, onSave }: TranscriptSettingsPopoverProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [language, setLanguage] = useState(settings.language ?? DETECT);
  const [vocabulary, setVocabulary] = useState(settings.vocabulary);
  const languages: string[] = [...TRANSCRIPTION_LANGUAGES];
  if (settings.language !== null && !languages.includes(settings.language)) {
    languages.push(settings.language);
  }
  return (
    <Popover
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (next) {
          setLanguage(settings.language ?? DETECT);
          setVocabulary(settings.vocabulary);
        }
      }}
    >
      <PopoverTrigger asChild>
        <IconButton
          icon={Languages}
          size="xs"
          variant="ghost"
          label={t('editor.transcript.settings.open')}
        />
      </PopoverTrigger>
      <PopoverContent className="flex w-72 flex-col gap-3">
        <Field label={t('editor.transcript.settings.language')}>
          <Select value={language} onValueChange={setLanguage}>
            <SelectTrigger size="xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={DETECT}>{t('editor.transcript.settings.detect')}</SelectItem>
              {languages.map((code) => (
                <SelectItem key={code} value={code}>
                  {languageName(code)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field
          label={t('editor.transcript.settings.vocabulary')}
          description={t('editor.transcript.settings.vocabularyHint')}
        >
          <Textarea
            rows={3}
            value={vocabulary}
            placeholder={t('editor.transcript.settings.vocabularyPlaceholder')}
            onChange={(event) => {
              setVocabulary(event.target.value);
            }}
          />
        </Field>
        <p className="text-2xs text-text-muted">{t('editor.transcript.settings.nextOnly')}</p>
        <Button
          size="sm"
          className="self-end"
          onClick={() => {
            onSave({
              language: language === DETECT ? null : language,
              vocabulary: cleanVocabulary(vocabulary),
            });
            setOpen(false);
          }}
        >
          {t('editor.transcript.settings.save')}
        </Button>
      </PopoverContent>
    </Popover>
  );
}
