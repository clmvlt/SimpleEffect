import type { ItemEntity, TrackKind } from '@se/protocol';
import { ClipBadge, ClipShell, cn } from '@se/ui';
import { Diamond } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { showTask, useItemPulse, useTaskNumber } from '../../../shared/ai-tasks';
import { useAssetSilences } from '../hooks/use-asset-silences';
import { mediaClipOf, silencesInItem } from '../services/audio-edits';
import type { ItemBox } from '../services/item-geometry';
import { animatedProperties } from '../services/keyframe-lane';
import { toggleKeyframeLane } from '../store/keyframe-store';
import { isAiMade, type ItemDisplay } from '../services/item-display';
import type { ClipEdit } from '../services/preview';
import { LABEL_BAR_HEIGHT } from '../services/volume-line';
import { ClipOverlay } from './ClipOverlay';
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
  /** Not drawn here: a gesture draws it elsewhere for now (moved, trimmed, shifted). */
  readonly hidden: boolean;
  /** Its edges can be dragged (the track is not locked). */
  readonly editable: boolean;
  /** Kind of its track: audio clips show their volume line and silences. */
  readonly trackKind: TrackKind;
  /** Fades or volume a gesture is dragging. */
  readonly edit?: ClipEdit | undefined;
  /** Hatch the silences of its sound (audio tracks). */
  readonly showSilences?: boolean | undefined;
  /** Its keyframe lane is unfolded below it. */
  readonly keyframesShown?: boolean | undefined;
}

/** Narrower clips draw no pictures: nothing would be legible, and zoomed out there are many. */
const MIN_PICTURE_WIDTH = 8;

/** Narrower clips show no trim handles: their whole body moves them. */
const MIN_TRIM_WIDTH = 16;

/** Where the pictures of a media item start in its source, and how fast it plays. */
function sourceOf(item: ItemEntity) {
  return item.kind.kind === 'mediaClip'
    ? { sourceStart: item.kind.sourceStart, speed: item.kind.speed }
    : { sourceStart: 0 as ItemEntity['timelineRange']['start'], speed: 1 };
}

/**
 * One item in a lane: a `ClipShell` positioned on its visible part, with its thumbnails or
 * waveform, its name, the AI badge when an AI task made it (a click shows the task), a brief glow
 * when an AI patch just changed it, trim handles on its edges and, for
 * media clips, its fades, volume line and silences (`ClipOverlay`). The lanes handle the pointer
 * (`data-item-id`, `data-trim-edge`, `data-fade-handle`, `data-volume-*`).
 */
export function TimelineItem({
  item,
  display,
  box,
  height,
  pixelsPerSecond,
  selected,
  dimmed,
  hidden,
  editable,
  trackKind,
  edit,
  showSilences = false,
  keyframesShown = false,
}: TimelineItemProps) {
  const { t } = useTranslation();
  const clip = mediaClipOf(item);
  const silences = useAssetSilences(
    clip === null ? null : display.asset,
    showSilences && trackKind === 'audio',
  );
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
  const trimmable = editable && box.width >= MIN_TRIM_WIDTH;
  const animated = trackKind === 'video' && animatedProperties(item.kind).length > 0;
  const pulse = useItemPulse(item.id);
  return (
    <div
      role="option"
      aria-selected={selected}
      aria-label={label}
      data-item-id={item.id}
      className={cn('absolute top-0 py-px [contain:strict]', hidden && 'invisible')}
      style={{ left: box.left, width: box.width, height }}
    >
      <ClipShell
        tone={display.tone}
        label={label}
        selected={selected}
        dimmed={dimmed}
        badges={isAiMade(item) ? <AiOriginBadge item={item} /> : undefined}
      >
        {asset !== null && display.picture === 'thumbnails' && (
          <ThumbnailStrip asset={asset} {...picture} height={height - 2} />
        )}
        {asset !== null && display.picture === 'waveform' && height > LABEL_BAR_HEIGHT + 4 && (
          <WaveformCanvas asset={asset} {...picture} height={height - LABEL_BAR_HEIGHT} />
        )}
      </ClipShell>
      {clip !== null && (
        <ClipOverlay
          item={item}
          clip={clip}
          trackKind={trackKind}
          box={box}
          height={height}
          pixelsPerSecond={pixelsPerSecond}
          editable={editable}
          edit={edit}
          silences={silencesInItem(item, silences)}
        />
      )}
      {animated && box.width >= MIN_TRIM_WIDTH * 2 && (
        <button
          type="button"
          data-keyframes-toggle=""
          aria-pressed={keyframesShown}
          aria-label={t('editor.timeline.keyframes.toggle')}
          className={cn(
            'absolute top-0.5 right-2 z-20 flex size-4 items-center justify-center rounded-xs',
            keyframesShown ? 'bg-accent text-accent-foreground' : 'text-text-muted hover:bg-hover',
          )}
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
          onClick={() => {
            toggleKeyframeLane(item.id);
          }}
        >
          <Diamond className="size-2.5" />
        </button>
      )}
      {pulse !== undefined && (
        <span
          key={pulse}
          aria-hidden
          data-ai-pulse=""
          className="pointer-events-none absolute inset-0 z-10 rounded-sm animate-ai-pulse"
        />
      )}
      {trimmable && box.startVisible && <TrimHandle edge="start" />}
      {trimmable && box.endVisible && <TrimHandle edge="end" />}
    </div>
  );
}

/** The « IA » badge of an item an AI task made: « Créé par la tâche #3 — voir » shows the task. */
function AiOriginBadge({ item }: { readonly item: ItemEntity }) {
  const { t } = useTranslation();
  const taskId = item.origin.kind === 'aiTask' ? item.origin.taskId : null;
  const number = useTaskNumber(taskId);
  const description =
    number === undefined
      ? t('editor.timeline.items.aiDescription')
      : t('editor.timeline.items.aiOrigin', { number });
  if (taskId === null || number === undefined) {
    return <ClipBadge label={t('editor.timeline.items.aiBadge')} description={description} />;
  }
  return (
    <button
      type="button"
      data-ai-origin={taskId}
      className="relative z-20 shrink-0 rounded-xs"
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      onClick={() => {
        showTask(taskId);
      }}
    >
      <ClipBadge label={t('editor.timeline.items.aiBadge')} description={description} />
    </button>
  );
}

/** A strip on one edge of a clip: dragging it trims that edge (Ctrl/⌘: ripple). */
function TrimHandle({ edge }: { readonly edge: 'start' | 'end' }) {
  return (
    <div
      aria-hidden
      data-trim-edge={edge}
      className={cn(
        'absolute inset-y-0 z-10 w-1.5 cursor-ew-resize hover:bg-accent/60',
        edge === 'start' ? 'left-0 rounded-l-sm' : 'right-0 rounded-r-sm',
      )}
    />
  );
}
