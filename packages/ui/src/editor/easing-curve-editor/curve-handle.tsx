import type { KeyboardEvent, PointerEvent as ReactPointerEvent } from 'react';

import { cn } from '../../lib/cn';
import { clampHandle, toPlot, type CurvePoint } from './easing-math';

/** Props of {@link CurveHandle}. */
export interface CurveHandleProps {
  point: CurvePoint;
  /** Where the handle's guide line starts: (0, 0) for the first handle, (1, 1) for the second. */
  anchor: CurvePoint;
  label: string;
  /** Keyboard move: the new point, changed and committed at once. */
  onKeyboardMove: (point: CurvePoint) => void;
  onPointerDown: (event: ReactPointerEvent<SVGGElement>) => void;
}

const ARROW_DELTAS: Readonly<Record<string, CurvePoint>> = {
  ArrowLeft: { x: -1, y: 0 },
  ArrowRight: { x: 1, y: 0 },
  ArrowUp: { x: 0, y: 1 },
  ArrowDown: { x: 0, y: -1 },
};

/**
 * A draggable control point of the Bézier curve with its guide line. Focusable: arrow keys move it
 * by 0.01 (Shift 0.1, Alt 0.001).
 */
export function CurveHandle({
  point,
  anchor,
  label,
  onKeyboardMove,
  onPointerDown,
}: CurveHandleProps) {
  const plotted = toPlot(point);
  const plottedAnchor = toPlot(anchor);

  const handleKeyDown = (event: KeyboardEvent<SVGGElement>) => {
    const delta = ARROW_DELTAS[event.key];
    if (delta === undefined) {
      return;
    }
    event.preventDefault();
    const step = event.shiftKey ? 0.1 : event.altKey ? 0.001 : 0.01;
    onKeyboardMove(clampHandle({ x: point.x + delta.x * step, y: point.y + delta.y * step }));
  };

  return (
    <>
      <line
        x1={plottedAnchor.x}
        y1={plottedAnchor.y}
        x2={plotted.x}
        y2={plotted.y}
        className="stroke-text-subtle"
        strokeWidth={1}
        vectorEffect="non-scaling-stroke"
      />
      <g
        role="button"
        tabIndex={0}
        aria-label={`${label}: ${point.x.toFixed(2)}, ${point.y.toFixed(2)}`}
        onKeyDown={handleKeyDown}
        onPointerDown={onPointerDown}
        className="group cursor-grab touch-none outline-none active:cursor-grabbing"
      >
        {/* A larger transparent disc makes the handle easy to grab. */}
        <circle cx={plotted.x} cy={plotted.y} r={6} className="fill-transparent" />
        {/* Keyboard focus ring, drawn around the handle. */}
        <circle
          cx={plotted.x}
          cy={plotted.y}
          r={5.5}
          className="fill-none stroke-focus-ring opacity-0 group-focus-visible:opacity-100"
          strokeWidth={1.5}
          vectorEffect="non-scaling-stroke"
        />
        <circle
          cx={plotted.x}
          cy={plotted.y}
          r={3}
          className={cn(
            'fill-accent stroke-panel transition-colors duration-(--duration-fast)',
            'group-hover:stroke-text',
          )}
          strokeWidth={1.5}
          vectorEffect="non-scaling-stroke"
        />
      </g>
    </>
  );
}
