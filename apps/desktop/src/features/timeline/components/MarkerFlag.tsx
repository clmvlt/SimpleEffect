import type { MarkerColor, MarkerEntity, MarkerPatch } from '@se/protocol';
import {
  cn,
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
  focusRing,
  Input,
  Popover,
  PopoverAnchor,
  PopoverContent,
  Tooltip,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

/** Fill class of each marker colour (written out so Tailwind sees every class). */
export const MARKER_COLOR_CLASSES: Readonly<Record<MarkerColor, string>> = {
  green: 'bg-marker-green',
  red: 'bg-marker-red',
  purple: 'bg-marker-purple',
  orange: 'bg-marker-orange',
  yellow: 'bg-marker-yellow',
  white: 'bg-marker-white',
  blue: 'bg-marker-blue',
  cyan: 'bg-marker-cyan',
};

const MARKER_COLORS = Object.keys(MARKER_COLOR_CLASSES) as MarkerColor[];

/** Props of {@link MarkerFlag}. */
export interface MarkerFlagProps {
  readonly marker: MarkerEntity;
  /** X of the marker in the ruler. */
  readonly x: number;
  readonly onGoTo: () => void;
  readonly onChange: (patch: MarkerPatch) => void;
  readonly onRemove: () => void;
}

/**
 * A marker on the ruler: a coloured flag showing its label on hover. Click moves the playhead to
 * it, double-click edits its label; its context menu changes the colour, goes to it or deletes it.
 */
export function MarkerFlag({ marker, x, onGoTo, onChange, onRemove }: MarkerFlagProps) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(marker.label);
  const name = marker.label === '' ? t('editor.timeline.marker.untitled') : marker.label;
  const startEditing = () => {
    setDraft(marker.label);
    setEditing(true);
  };
  const commit = () => {
    if (draft !== marker.label) onChange({ label: draft });
    setEditing(false);
  };
  return (
    <Popover open={editing} onOpenChange={setEditing}>
      <ContextMenu>
        <Tooltip content={name}>
          <ContextMenuTrigger asChild>
            <PopoverAnchor asChild>
              <button
                type="button"
                aria-label={t('editor.timeline.marker.label', { name })}
                data-marker-id={marker.id}
                className={cn(
                  'absolute top-0 z-10 h-3 w-2.5 -translate-x-1/2 rounded-b-xs',
                  focusRing,
                  MARKER_COLOR_CLASSES[marker.color],
                )}
                style={{ left: x }}
                onPointerDown={(event) => {
                  event.stopPropagation();
                }}
                onClick={onGoTo}
                onDoubleClick={startEditing}
              />
            </PopoverAnchor>
          </ContextMenuTrigger>
        </Tooltip>
        <ContextMenuContent>
          <ContextMenuItem onSelect={startEditing}>
            {t('editor.timeline.marker.edit')}
          </ContextMenuItem>
          <ContextMenuSub>
            <ContextMenuSubTrigger>{t('editor.timeline.marker.color')}</ContextMenuSubTrigger>
            <ContextMenuSubContent>
              <ContextMenuRadioGroup
                value={marker.color}
                onValueChange={(color) => {
                  onChange({ color: color as MarkerColor });
                }}
              >
                {MARKER_COLORS.map((color) => (
                  <ContextMenuRadioItem key={color} value={color}>
                    <span
                      aria-hidden
                      className={cn('size-2.5 rounded-full', MARKER_COLOR_CLASSES[color])}
                    />
                    {t(`editor.timeline.marker.colors.${color}`)}
                  </ContextMenuRadioItem>
                ))}
              </ContextMenuRadioGroup>
            </ContextMenuSubContent>
          </ContextMenuSub>
          <ContextMenuItem onSelect={onGoTo}>{t('editor.timeline.marker.goTo')}</ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem variant="danger" onSelect={onRemove}>
            {t('editor.timeline.marker.remove')}
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      <PopoverContent side="bottom" className="w-56 p-2">
        <Input
          autoFocus
          size="xs"
          aria-label={t('editor.timeline.marker.labelField')}
          placeholder={t('editor.timeline.marker.labelPlaceholder')}
          value={draft}
          onChange={(event) => {
            setDraft(event.target.value);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter') commit();
          }}
          onBlur={commit}
        />
      </PopoverContent>
    </Popover>
  );
}
