import type { Flicks, FrameRate, ItemEntity } from '@se/protocol';
import {
  EasingCurveEditor,
  Popover,
  PopoverAnchor,
  PopoverContent,
  cn,
  type EasingCurve,
} from '@se/ui';
import { useRef, useState, type PointerEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { curveToEasing, easingToCurve } from '../../../shared/animation';
import { FLICKS_PER_SECOND, formatTimecode, framesToFlicks } from '../../../shared/time';
import type { TimelineCommands } from '../hooks/use-timeline-commands';
import type { ItemBox } from '../services/item-geometry';
import {
  KEYFRAME_ROW_HEIGHT,
  animatedProperties,
  easedKeyframeAnimation,
  easingOf,
  keyframeTimes,
  movedKeyframeAnimations,
} from '../services/keyframe-lane';
import {
  clearKeyframeSelection,
  keyframeLaneStore,
  sameKey,
  selectKeyframe,
  setKeyframeDrag,
  useKeyframeLanes,
  type KeyframeKey,
} from '../store/keyframe-store';

/** Props of {@link KeyframeLane}. */
export interface KeyframeLaneProps {
  readonly item: ItemEntity;
  /** Name of the item (the lane's accessible name). */
  readonly label: string;
  readonly box: ItemBox;
  /** Top of the lane in its track row (below the item). */
  readonly top: number;
  readonly pixelsPerSecond: number;
  readonly frameRate: FrameRate;
  readonly commands: TimelineCommands;
  readonly editable: boolean;
}

const DIAMOND = 9;

/**
 * The unfolded keyframes of an item, under it: one row per animated property with a diamond per
 * keyframe. Click selects (Maj adds), dragging moves the selected keyframes by whole frames (one
 * command on release), `Suppr` deletes them (edit actions), double-click edits the easing leaving
 * the keyframe (`EasingCurveEditor`, one command per change).
 */
export function KeyframeLane({
  item,
  label,
  box,
  top,
  pixelsPerSecond,
  frameRate,
  commands,
  editable,
}: KeyframeLaneProps) {
  const { t } = useTranslation();
  const { selection, dragDelta } = useKeyframeLanes();
  const lane = useRef<HTMLDivElement | null>(null);
  const drag = useRef<{ startX: number; keys: readonly KeyframeKey[] } | null>(null);
  const [easing, setEasing] = useState<KeyframeKey | null>(null);
  const properties = animatedProperties(item.kind);
  const selectedKeys = selection?.itemId === item.id ? selection.keys : [];
  const isSelected = (key: KeyframeKey) => selectedKeys.some((other) => sameKey(other, key));
  const x = (time: number) => (time / FLICKS_PER_SECOND) * pixelsPerSecond - box.offsetPixels;
  const frames = (pixels: number) =>
    Math.round((pixels / pixelsPerSecond) * (frameRate.numerator / frameRate.denominator));
  const delta = (pixels: number): Flicks => {
    const shift = framesToFlicks(frames(pixels), frameRate);
    return shift.ok ? shift.value : (0 as Flicks);
  };
  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (drag.current === null) return;
    setKeyframeDrag(delta(event.clientX - drag.current.startX));
  };
  const onPointerUp = (event: PointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    drag.current = null;
    if (current === null) return;
    const shift = delta(event.clientX - current.startX);
    setKeyframeDrag(null);
    if (shift === 0) return;
    const changes = movedKeyframeAnimations(item, current.keys, shift);
    if (changes === null) return;
    void commands.setItemAnimations(changes).then((done) => {
      if (done === null) return;
      keyframeLaneStore.setState({
        selection: {
          itemId: item.id,
          keys: current.keys.map((key) => ({ ...key, time: (key.time + shift) as Flicks })),
        },
      });
    });
  };
  return (
    <div
      ref={lane}
      role="group"
      aria-label={t('editor.timeline.keyframes.lane', { name: label })}
      data-keyframe-lane={item.id}
      className="absolute overflow-hidden border-x border-b border-border bg-panel-raised/80"
      style={{
        left: box.left,
        width: box.width,
        top,
        height: properties.length * KEYFRAME_ROW_HEIGHT,
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
        if (event.target === event.currentTarget) clearKeyframeSelection();
      }}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    >
      {properties.map((property, row) => (
        <div
          key={property}
          className="absolute inset-x-0 border-b border-border/40"
          style={{ top: row * KEYFRAME_ROW_HEIGHT, height: KEYFRAME_ROW_HEIGHT }}
        >
          <span className="pointer-events-none absolute left-1 text-2xs leading-4 text-text-subtle">
            {t(`editor.timeline.keyframes.properties.${property}`)}
          </span>
          {keyframeTimes(item.kind, property).map((time) => {
            const key = { property, time };
            const selected = isSelected(key);
            const shown = selected && dragDelta !== null ? time + dragDelta : time;
            return (
              <button
                key={time}
                type="button"
                aria-pressed={selected}
                aria-label={t('editor.timeline.keyframes.keyframe', {
                  time: formatTimecode((item.timelineRange.start + time) as Flicks, frameRate),
                })}
                data-keyframe={`${property}@${String(time)}`}
                disabled={!editable}
                className={cn(
                  'absolute top-1/2 rotate-45 border border-text/70',
                  selected ? 'bg-accent' : 'bg-panel',
                )}
                style={{
                  left: x(shown) - DIAMOND / 2,
                  marginTop: -DIAMOND / 2,
                  width: DIAMOND,
                  height: DIAMOND,
                }}
                onPointerDown={(event) => {
                  event.stopPropagation();
                  const additive = event.shiftKey || event.ctrlKey || event.metaKey;
                  if (!selected || additive) selectKeyframe(item.id, key, additive);
                  const keys = keyframeLaneStore.getState().selection?.keys ?? [key];
                  drag.current = { startX: event.clientX, keys };
                  lane.current?.setPointerCapture(event.pointerId);
                }}
                onDoubleClick={() => {
                  setEasing(key);
                }}
              />
            );
          })}
        </div>
      ))}
      {easing !== null && (
        <EasingPopover
          item={item}
          keyframe={easing}
          left={x(easing.time)}
          top={properties.indexOf(easing.property) * KEYFRAME_ROW_HEIGHT}
          commands={commands}
          onClose={() => {
            setEasing(null);
          }}
        />
      )}
    </div>
  );
}

/** The easing editor of one keyframe, in a popover anchored on its diamond. */
function EasingPopover({
  item,
  keyframe,
  left,
  top,
  commands,
  onClose,
}: {
  readonly item: ItemEntity;
  readonly keyframe: KeyframeKey;
  readonly left: number;
  readonly top: number;
  readonly commands: TimelineCommands;
  readonly onClose: () => void;
}) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<EasingCurve | null>(null);
  return (
    <Popover
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <PopoverAnchor className="absolute" style={{ left, top }} />
      <PopoverContent aria-label={t('editor.timeline.keyframes.easing')} className="w-64">
        <EasingCurveEditor
          value={draft ?? easingToCurve(easingOf(item, keyframe))}
          onChange={setDraft}
          onCommit={(curve) => {
            setDraft(null);
            const change = easedKeyframeAnimation(item, keyframe, curveToEasing(curve));
            if (change !== null) void commands.setItemAnimations([change]);
          }}
        />
      </PopoverContent>
    </Popover>
  );
}
