import type { ItemEntity } from '@se/protocol';
import { ClipBadge, ClipShell } from '@se/ui';
import type { PointerEvent } from 'react';
import { useTranslation } from 'react-i18next';

import type { ItemBox } from '../services/item-geometry';
import { isAiMade, type ItemDisplay } from '../services/item-display';
import { ThumbnailStrip } from './ThumbnailStrip';
import { WaveformCanvas } from './WaveformCanvas';

/** Props of {@link TimelineItem}. */
export interface TimelineItemProps {
  readonly item: ItemEntity;
  readonly display: ItemDisplay;
  readonly box: ItemBox;
  /** Height of the track row in pixels. */
  readonly height: number;
  readonly pixelsPerSecond: number;
  readonly selected: boolean;
  /** Drawn faded (hidden or muted track). */
  readonly dimmed: boolean;
  readonly onPointerDown: (item: ItemEntity, event: PointerEvent<HTMLDivElement>) => void;
}

/** Height of the clip's label bar plus its borders: the waveform is drawn below it. */
const LABEL_BAR_HEIGHT = 18;

/** Narrower clips draw no pictures: nothing would be legible, and zoomed out there are many. */
const MIN_PICTURE_WIDTH = 8;

/** Where the pictures of a media item start in its source, and how fast it plays. */
function sourceOf(item: ItemEntity) {
  return item.kind.kind === 'mediaClip'
    ? { sourceStart: item.kind.sourceStart, speed: item.kind.speed }
    : { sourceStart: 0 as ItemEntity['timelineRange']['start'], speed: 1 };
}

/**
 * One item in a lane: a `ClipShell` positioned on its visible part, with its thumbnails or
 * waveform, its name and the AI badge when an AI task made it.
 */
export function TimelineItem({
  item,
  display,
  box,
  height,
  pixelsPerSecond,
  selected,
  dimmed,
  onPointerDown,
}: TimelineItemProps) {
  const { t } = useTranslation();
  const label = display.label ?? t(`editor.timeline.items.${display.fallback}`);
  const { sourceStart, speed } = sourceOf(item);
  const asset = box.width >= MIN_PICTURE_WIDTH ? display.asset : null;
  const picture = {
    sourceStart,
    speed,
    pixelsPerSecond,
    offsetPixels: box.offsetPixels,
    width: box.width,
  };
  return (
    <div
      role="option"
      aria-selected={selected}
      aria-label={label}
      data-item-id={item.id}
      className="absolute top-0 py-px [contain:strict]"
      style={{ left: box.left, width: box.width, height }}
      onPointerDown={(event) => {
        onPointerDown(item, event);
      }}
    >
      <ClipShell
        tone={display.tone}
        label={label}
        selected={selected}
        dimmed={dimmed}
        badges={
          isAiMade(item) ? (
            <ClipBadge
              label={t('editor.timeline.items.aiBadge')}
              description={t('editor.timeline.items.aiDescription')}
            />
          ) : undefined
        }
      >
        {asset !== null && display.picture === 'thumbnails' && (
          <ThumbnailStrip asset={asset} {...picture} height={height - 2} />
        )}
        {asset !== null && display.picture === 'waveform' && height > LABEL_BAR_HEIGHT + 4 && (
          <WaveformCanvas asset={asset} {...picture} height={height - LABEL_BAR_HEIGHT} />
        )}
      </ClipShell>
    </div>
  );
}
