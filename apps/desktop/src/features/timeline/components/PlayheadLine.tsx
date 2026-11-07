import type { SequenceId } from '@se/protocol';
import { cn } from '@se/ui';
import { useEffect, useRef, type ReactNode } from 'react';

import { timeToX, type Viewport } from '../services/viewport';
import { playheadStore, playheadTimeOf } from '../store/playhead-store';

/** Props of {@link PlayheadLine}. */
export interface PlayheadLineProps {
  readonly sequenceId: SequenceId;
  readonly viewport: Viewport;
  readonly className?: string | undefined;
  /** Drawn at the line's top (the ruler's handle). */
  readonly children?: ReactNode;
}

/**
 * The playhead: a line that follows the playhead store by setting its own transform, so playhead
 * moves (drags, playback) never re-render the timeline. Hidden while out of view.
 */
export function PlayheadLine({ sequenceId, viewport, className, children }: PlayheadLineProps) {
  const element = useRef<HTMLDivElement>(null);
  const { pixelsPerSecond, scrollTime, width } = viewport;
  useEffect(() => {
    const view = { pixelsPerSecond, scrollTime, width };
    const place = () => {
      const node = element.current;
      if (node === null) return;
      const x = timeToX(playheadTimeOf(playheadStore.getState(), sequenceId), view);
      node.style.transform = `translateX(${String(Math.round(x))}px)`;
      node.style.visibility = x < -1 || x > width + 1 ? 'hidden' : 'visible';
    };
    place();
    return playheadStore.subscribe(place);
  }, [sequenceId, pixelsPerSecond, scrollTime, width]);
  return (
    <div
      ref={element}
      aria-hidden
      data-playhead
      className={cn('pointer-events-none absolute inset-y-0 left-0 z-20 w-px bg-accent', className)}
    >
      {children}
    </div>
  );
}
