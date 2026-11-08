import type { Flicks } from '@se/protocol';
import {
  Button,
  IconButton,
  Input,
  Label,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Tooltip,
} from '@se/ui';
import { MoveHorizontal } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import type { CaptionCommands } from '../hooks/use-caption-commands';
import type { CaptionRow } from '../services/caption-rows';
import { parseShiftSeconds } from '../services/caption-editing';

/** Props of {@link ShiftPopover}. */
export interface ShiftPopoverProps {
  readonly rows: readonly CaptionRow[];
  readonly commands: CaptionCommands;
}

/** « Décaler » every caption of the list by a number of seconds (one undo step). */
export function ShiftPopover({ rows, commands }: ShiftPopoverProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [text, setText] = useState('');
  const seconds = parseShiftSeconds(text);
  const apply = () => {
    if (seconds === null) return;
    setOpen(false);
    setText('');
    void commands.shift(
      rows.map((row) => row.itemId),
      Math.round(seconds * FLICKS_PER_SECOND) as Flicks,
    );
  };
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <Tooltip content={t('editor.captions.shift.title')}>
        <PopoverTrigger asChild>
          <IconButton
            icon={MoveHorizontal}
            size="xs"
            label={t('editor.captions.shift.title')}
            disabled={commands.busy || rows.length === 0}
          />
        </PopoverTrigger>
      </Tooltip>
      <PopoverContent className="flex w-64 flex-col gap-2">
        <Label htmlFor="captions-shift">{t('editor.captions.shift.label')}</Label>
        <Input
          id="captions-shift"
          size="xs"
          value={text}
          placeholder={t('editor.captions.shift.placeholder')}
          onChange={(event) => {
            setText(event.currentTarget.value);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter') apply();
          }}
        />
        <Button size="xs" variant="primary" disabled={seconds === null} onClick={apply}>
          {t('editor.captions.shift.apply')}
        </Button>
      </PopoverContent>
    </Popover>
  );
}
