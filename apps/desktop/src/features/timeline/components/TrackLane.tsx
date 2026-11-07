import type { Asset, ItemEntity, ItemId, MotionTemplate } from '@se/protocol';
import { cn } from '@se/ui';

import { itemBox } from '../services/item-geometry';
import { itemDisplay } from '../services/item-display';
import type { TrackRow } from '../services/track-layout';
import type { Viewport } from '../services/viewport';
import { TimelineItem } from './TimelineItem';

/** Props of {@link TrackLane}. */
export interface TrackLaneProps {
  readonly row: TrackRow;
  /** The items of the track in view (already virtualised). */
  readonly items: readonly ItemEntity[];
  readonly viewport: Viewport;
  readonly assets: Readonly<Record<string, Asset>>;
  readonly templates: readonly MotionTemplate[];
  readonly selected: ReadonlySet<ItemId>;
  /** Items a gesture draws elsewhere for now. */
  readonly hidden: ReadonlySet<ItemId>;
}

/** The row of one track: its items in view, faded when the track is hidden or muted. */
export function TrackLane({
  row,
  items,
  viewport,
  assets,
  templates,
  selected,
  hidden,
}: TrackLaneProps) {
  const { track } = row;
  const dimmed = track.kind === 'video' ? track.hidden : track.muted;
  return (
    <div
      role="listbox"
      aria-label={track.name}
      aria-multiselectable
      data-track-id={track.id}
      className={cn('absolute inset-x-0 border-b border-border/60', track.locked && 'bg-hover')}
      style={{ top: row.top, height: row.height }}
    >
      {items.map((item) => (
        <TimelineItem
          key={item.id}
          item={item}
          display={itemDisplay(item, track.kind, assets, templates)}
          box={itemBox(item, viewport)}
          height={row.height}
          pixelsPerSecond={viewport.pixelsPerSecond}
          selected={selected.has(item.id)}
          dimmed={dimmed}
          hidden={hidden.has(item.id)}
          editable={!track.locked}
        />
      ))}
    </div>
  );
}
