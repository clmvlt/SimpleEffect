import type { Flicks, ItemId, Transition, TransitionId } from '@se/protocol';

import type { EvaluableItem } from './evaluate';

/** The parts of a transition the engine reads (a domain `Transition` fits). */
export type EvaluableTransition = Pick<
  Transition,
  'id' | 'kind' | 'leftItem' | 'rightItem' | 'duration' | 'alignment'
>;

/** Where a transition plays: its span around the cut, in sequence flicks. */
export interface TransitionSpan {
  readonly id: TransitionId;
  readonly leftItem: ItemId;
  readonly rightItem: ItemId;
  /** First instant of the cross-fade (the incoming item starts playing here). */
  readonly start: Flicks;
  /** The cut: end of the outgoing item, start of the incoming one. */
  readonly cut: Flicks;
  /** Instant after the cross-fade (the outgoing item stops playing here). */
  readonly end: Flicks;
}

/**
 * How much of a transition lies before and after its cut (same split as the domain's
 * `Transition::extent`): half each way when centred (the extra flick of an odd duration after
 * the cut), all after when it starts at the cut, all before when it ends there.
 */
export function transitionExtent(transition: Pick<Transition, 'duration' | 'alignment'>): {
  readonly before: number;
  readonly after: number;
} {
  const duration = transition.duration;
  switch (transition.alignment) {
    case 'center': {
      const before = Math.floor(duration / 2);
      return { before, after: duration - before };
    }
    case 'start':
      return { before: 0, after: duration };
    case 'end':
      return { before: duration, after: 0 };
  }
}

/** The span of `transition` around the cut at `cut`. */
export function transitionSpan(transition: EvaluableTransition, cut: Flicks): TransitionSpan {
  const { before, after } = transitionExtent(transition);
  return {
    id: transition.id,
    leftItem: transition.leftItem,
    rightItem: transition.rightItem,
    start: (cut - before) as Flicks,
    cut,
    end: (cut + after) as Flicks,
  };
}

/** Progress of a cross-fade at `time`: 0 at its start, 1 at its end. */
export function transitionProgress(span: TransitionSpan, time: number): number {
  const length = span.end - span.start;
  return length <= 0 ? 1 : Math.min(1, Math.max(0, (time - span.start) / length));
}

/**
 * The spans of a track's transitions whose items are adjacent on it (others are ignored: the
 * domain removes them, a stale mirror may still show one for an instant).
 */
export function trackTransitionSpans(
  items: readonly EvaluableItem[],
  transitions: readonly EvaluableTransition[],
): TransitionSpan[] {
  if (transitions.length === 0) return [];
  const ends = new Map<ItemId, number>();
  const starts = new Map<ItemId, number>();
  for (const item of items) {
    starts.set(item.id, item.timelineRange.start);
    ends.set(item.id, item.timelineRange.start + item.timelineRange.duration);
  }
  const spans: TransitionSpan[] = [];
  for (const transition of transitions) {
    const cut = ends.get(transition.leftItem);
    if (cut === undefined || starts.get(transition.rightItem) !== cut) continue;
    spans.push(transitionSpan(transition, cut as Flicks));
  }
  return spans;
}

/** How far each item of a track plays beyond its edges because of its transitions. */
export interface PlayExtension {
  /** Flicks played before the item's start (incoming side). */
  readonly before: number;
  /** Flicks played after the item's end (outgoing side). */
  readonly after: number;
}

/** The extension of every item joined by one of `spans`. */
export function playExtensions(spans: readonly TransitionSpan[]): Map<ItemId, PlayExtension> {
  const extensions = new Map<ItemId, PlayExtension>();
  const extend = (item: ItemId, part: Partial<PlayExtension>) => {
    const current = extensions.get(item) ?? { before: 0, after: 0 };
    extensions.set(item, { ...current, ...part });
  };
  for (const span of spans) {
    extend(span.leftItem, { after: span.end - span.cut });
    extend(span.rightItem, { before: span.cut - span.start });
  }
  return extensions;
}
