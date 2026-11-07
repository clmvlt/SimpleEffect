import type { ShapeKind } from '@se/protocol';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  ScrubInput,
  Tooltip,
} from '@se/ui';
import { Shapes, Type } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { useShortcutAction, useShortcutHint } from '../../../shared/shortcuts';
import { useCreateGraphic } from '../hooks/use-create-graphic';
import { usePersistedGraphicsPreferences } from '../hooks/use-graphics-preferences';
import { GRAPHIC_SECONDS_RANGE, setGraphicsPreferences } from '../store/graphics-preferences-store';

/** Shapes offered, in menu order. */
export const SHAPE_KINDS: readonly ShapeKind[] = [
  'rectangle',
  'roundedRectangle',
  'ellipse',
  'line',
];

function formatSeconds(value: number): string {
  return `${value.toFixed(1).replace('.', ',')} s`;
}

/**
 * Creation tools of the viewer's toolbar: « Texte » (T), « Forme » (menu of shapes) and the length
 * of new graphics. New items land at the playhead on the highest free video track.
 */
export function GraphicsTools() {
  const { t } = useTranslation();
  const creator = useCreateGraphic();
  const preferences = usePersistedGraphicsPreferences();
  const textHint = useShortcutHint('addText');
  useShortcutAction('addText', () => creator?.createText(), creator !== null);
  return (
    <div className="flex items-center gap-1" role="group" aria-label={t('editor.graphics.tools')}>
      <Tooltip
        content={t('editor.graphics.addText')}
        {...(textHint === undefined ? {} : { shortcut: textHint })}
      >
        <IconButton
          icon={Type}
          size="xs"
          variant="ghost"
          label={t('editor.graphics.addText')}
          disabled={creator === null}
          onClick={() => creator?.createText()}
        />
      </Tooltip>
      <DropdownMenu>
        <Tooltip content={t('editor.graphics.addShape')}>
          <DropdownMenuTrigger asChild>
            <IconButton
              icon={Shapes}
              size="xs"
              variant="ghost"
              label={t('editor.graphics.addShape')}
              disabled={creator === null}
            />
          </DropdownMenuTrigger>
        </Tooltip>
        <DropdownMenuContent align="end">
          {SHAPE_KINDS.map((shape) => (
            <DropdownMenuItem
              key={shape}
              onSelect={() => {
                creator?.createShape(shape);
              }}
            >
              {t(`editor.graphics.shapes.${shape}`)}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
      <ScrubInput
        size="xs"
        className="w-16"
        label={t('editor.graphics.duration')}
        value={preferences.graphicSeconds}
        min={GRAPHIC_SECONDS_RANGE.min}
        max={GRAPHIC_SECONDS_RANGE.max}
        step={0.5}
        format={formatSeconds}
        onChange={(graphicSeconds) => {
          setGraphicsPreferences({ graphicSeconds });
        }}
      />
    </div>
  );
}
