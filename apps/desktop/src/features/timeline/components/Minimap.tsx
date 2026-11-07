import { useEffect, useRef, type PointerEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { prepareCanvas, tokenValue } from '../../../shared/canvas';
import type { TimelineModel } from '../hooks/use-timeline-model';
import type { TimelineViewportApi } from '../hooks/use-timeline-viewport';
import {
  overviewBars,
  overviewSeconds,
  viewportFromWindow,
  windowPercent,
  windowRect,
} from '../services/minimap';
import { allRows, type TrackLayout } from '../services/track-layout';

/** Height of the overview strip. */
export const MINIMAP_HEIGHT = 20;

/** Props of {@link Minimap}. */
export interface MinimapProps {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly api: TimelineViewportApi;
  /** Id of the lanes element the strip scrolls. */
  readonly lanesId: string;
}

type Grab = 'move' | 'start' | 'end';

interface Drag {
  readonly grab: Grab;
  readonly startX: number;
  readonly left: number;
  readonly right: number;
}

/**
 * The whole sequence in a thin strip (items as coloured bars per track, in a canvas) and the
 * visible window as a frame: drag it to scroll, drag its edges to zoom, click outside it to jump
 * there; arrow keys on the frame scroll.
 */
export function Minimap({ model, layout, api, lanesId }: MinimapProps) {
  const { t } = useTranslation();
  const element = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const drag = useRef<Drag | null>(null);
  const { viewport, limits } = api;
  const { width } = viewport;
  const total = overviewSeconds(viewport, model.contentEnd);
  const frame = windowRect(viewport, model.contentEnd, width);
  const bars = overviewBars(allRows(layout), model.itemsByTrack, total, width, MINIMAP_HEIGHT);

  useEffect(() => {
    const node = canvas.current;
    if (node === null || width <= 0) return;
    const context = prepareCanvas(node, width, MINIMAP_HEIGHT);
    if (context === null) return;
    const colors = {
      video: tokenValue('--color-item-video'),
      audio: tokenValue('--color-item-audio'),
    };
    for (const bar of bars) {
      context.fillStyle = colors[bar.kind];
      context.fillRect(bar.x, bar.y, bar.width, 1);
    }
  }, [bars, width]);

  const jumpTo = (left: number, right: number) => {
    api.setViewport(viewportFromWindow(viewport, { left, right }, model.contentEnd, width, limits));
  };
  const begin = (grab: Grab, event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0) return;
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = {
      grab,
      startX: event.clientX,
      left: frame.left,
      right: frame.left + frame.width,
    };
  };
  const move = (event: PointerEvent<HTMLElement>) => {
    const current = drag.current;
    if (current === null) return;
    const delta = event.clientX - current.startX;
    const left = current.grab === 'end' ? current.left : current.left + delta;
    const right = current.grab === 'start' ? current.right : current.right + delta;
    if (right - left >= 4) jumpTo(left, right);
  };
  const end = () => {
    drag.current = null;
  };
  const dragHandlers = { onPointerMove: move, onPointerUp: end, onPointerCancel: end };

  return (
    <div
      ref={element}
      data-minimap
      className="relative min-w-0 flex-1 overflow-hidden bg-bg"
      style={{ height: MINIMAP_HEIGHT }}
      onPointerDown={(event) => {
        if (event.button !== 0 || element.current === null) return;
        const x = event.clientX - element.current.getBoundingClientRect().left;
        jumpTo(x - frame.width / 2, x + frame.width / 2);
      }}
    >
      <canvas
        ref={canvas}
        aria-hidden
        className="pointer-events-none absolute inset-0"
        style={{ width, height: MINIMAP_HEIGHT }}
      />
      <div
        role="scrollbar"
        tabIndex={0}
        aria-label={t('editor.timeline.minimap')}
        aria-controls={lanesId}
        aria-orientation="horizontal"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={windowPercent(frame, width)}
        className="absolute inset-y-0 cursor-grab rounded-xs border border-accent bg-accent-muted/40 active:cursor-grabbing"
        style={{ left: frame.left, width: frame.width }}
        onPointerDown={(event) => {
          begin('move', event);
        }}
        {...dragHandlers}
        onKeyDown={(event) => {
          if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
          event.preventDefault();
          api.scrollBy((event.key === 'ArrowLeft' ? -0.1 : 0.1) * width);
        }}
      >
        <span
          aria-hidden
          className="absolute inset-y-0 left-0 w-1.5 cursor-ew-resize"
          onPointerDown={(event) => {
            begin('start', event);
          }}
          {...dragHandlers}
        />
        <span
          aria-hidden
          className="absolute inset-y-0 right-0 w-1.5 cursor-ew-resize"
          onPointerDown={(event) => {
            begin('end', event);
          }}
          {...dragHandlers}
        />
      </div>
    </div>
  );
}
