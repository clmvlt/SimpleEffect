import type { Asset, FrameRate, ItemEntity, ItemId, MotionTemplate } from '@se/protocol';

import type { ClipEdit } from '../services/preview';
import { cn } from '@se/ui';

import type { TimelineCommands } from '../hooks/use-timeline-commands';
import { itemBox } from '../services/item-geometry';
import { itemDisplay } from '../services/item-display';
import { useKeyframeLanes } from '../store/keyframe-store';
import { KeyframeLane } from './KeyframeLane';
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
  /** Fades and volumes a gesture is dragging. */
  readonly edits?: ReadonlyMap<ItemId, ClipEdit> | undefined;
  /** Hatch the silences of audio clips. */
  readonly showSilences?: boolean | undefined;
  /** What unfolded keyframe lanes need: the commands they send and the frame rate they snap to. */
  readonly keyframes: { readonly commands: TimelineCommands; readonly frameRate: FrameRate };
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
  edits,
  showSilences,
  keyframes,
}: TrackLaneProps) {
  const { track } = row;
  const { expanded } = useKeyframeLanes();
  const dimmed = track.kind === 'video' ? track.hidden : track.muted;
  return (
    <div
      role="listbox"
      aria-label={track.name}
      aria-multiselectable
      data-track-id={track.id}
      className={cn('absolute inset-x-0 border-b border-border/60', track.locked && 'bg-hover')}
      style={{ top: row.top, height: row.height + row.laneHeight }}
    >
      {items.map((item) => {
        const display = itemDisplay(item, track.kind, assets, templates);
        const box = itemBox(item, viewport);
        const unfolded = expanded.has(item.id) && !hidden.has(item.id);
        return [
          <TimelineItem
            key={item.id}
            item={item}
            display={display}
            box={box}
            height={row.height}
            pixelsPerSecond={viewport.pixelsPerSecond}
            selected={selected.has(item.id)}
            dimmed={dimmed}
            hidden={hidden.has(item.id)}
            editable={!track.locked}
            trackKind={track.kind}
            edit={edits?.get(item.id)}
            showSilences={showSilences}
            keyframesShown={expanded.has(item.id)}
          />,
          unfolded && (
            <KeyframeLane
              key={`${item.id}-keyframes`}
              item={item}
              label={display.label ?? item.id}
              box={box}
              top={row.height}
              pixelsPerSecond={viewport.pixelsPerSecond}
              frameRate={keyframes.frameRate}
              commands={keyframes.commands}
              editable={!track.locked}
            />
          ),
        ];
      })}
    </div>
  );
}
