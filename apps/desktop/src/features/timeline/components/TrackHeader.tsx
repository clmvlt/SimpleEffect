import type { TrackPatch } from '@se/protocol';
import {
  cn,
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
  Icon,
} from '@se/ui';
import { AudioLines, Film, GripVertical } from '@se/ui/icons';
import { useState, type HTMLAttributes } from 'react';
import { useTranslation } from 'react-i18next';

import type { TrackPlacement, TrackRow } from '../services/track-layout';
import { TrackAudioControls } from './TrackAudioControls';
import { TrackHeaderButtons } from './TrackHeaderButtons';
import { TrackName } from './TrackName';

/** Props of {@link TrackHeader}. */
export interface TrackHeaderProps {
  readonly row: TrackRow;
  /** Faded while the track is being dragged to another place. */
  readonly dragging: boolean;
  readonly gripHandlers: HTMLAttributes<HTMLElement>;
  readonly resizeHandlers: HTMLAttributes<HTMLElement>;
  readonly onChange: (patch: TrackPatch) => void;
  readonly onAdd: (placement: TrackPlacement) => void;
  readonly onRemove: () => void;
}

/** Audio tracks at least this high show their volume and balance fields. */
const AUDIO_CONTROLS_MIN_HEIGHT = 56;

/**
 * The header of a track: grip (drag to reorder), kind icon, name (double-click to rename),
 * toggles, volume and balance on tall enough audio tracks, a bottom edge to resize, and a context
 * menu (add above/below, rename, delete).
 */
export function TrackHeader({
  row,
  dragging,
  gripHandlers,
  resizeHandlers,
  onChange,
  onAdd,
  onRemove,
}: TrackHeaderProps) {
  const { t } = useTranslation();
  const { track } = row;
  const [editing, setEditing] = useState(false);
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          role="group"
          aria-label={t('editor.timeline.track.label', { name: track.name })}
          data-track-header={track.id}
          className={cn(
            'absolute inset-x-0 flex items-start gap-1 border-b border-border/60 bg-panel px-1 pt-1',
            dragging && 'opacity-50',
          )}
          style={{ top: row.top, height: row.height }}
        >
          <span
            className="flex h-control-xs cursor-grab items-center text-text-subtle active:cursor-grabbing"
            {...gripHandlers}
          >
            <Icon icon={GripVertical} size="sm" />
          </span>
          <span className="flex h-control-xs items-center text-text-muted">
            <Icon icon={track.kind === 'video' ? Film : AudioLines} size="sm" />
          </span>
          <div className="flex h-control-xs min-w-0 flex-1 items-center">
            <TrackName
              name={track.name}
              editing={editing}
              onEditingChange={setEditing}
              onRename={(name) => {
                onChange({ name });
              }}
            />
          </div>
          <TrackHeaderButtons track={track} onChange={onChange} />
          {track.kind === 'audio' && row.height >= AUDIO_CONTROLS_MIN_HEIGHT && (
            <TrackAudioControls track={track} onChange={onChange} />
          )}
          <div
            aria-hidden
            className="absolute inset-x-0 -bottom-0.5 h-1.5 cursor-row-resize"
            {...resizeHandlers}
          />
        </div>
      </ContextMenuTrigger>
      <ContextMenuContent
        onCloseAutoFocus={(event) => {
          // Keep the focus in the rename field when « Renommer » was chosen.
          event.preventDefault();
        }}
      >
        <ContextMenuItem
          onSelect={() => {
            onAdd('above');
          }}
        >
          {t('editor.timeline.track.addAbove')}
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={() => {
            onAdd('below');
          }}
        >
          {t('editor.timeline.track.addBelow')}
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          onSelect={() => {
            setEditing(true);
          }}
        >
          {t('editor.timeline.track.rename')}
        </ContextMenuItem>
        <ContextMenuItem variant="danger" disabled={track.locked} onSelect={onRemove}>
          {t('editor.timeline.track.remove')}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
