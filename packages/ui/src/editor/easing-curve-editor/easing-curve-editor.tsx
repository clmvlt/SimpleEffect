import { useRef, type PointerEvent as ReactPointerEvent } from 'react';

import { cn } from '../../lib/cn';
import { usePointerDrag } from '../../lib/use-pointer-drag';
import { ToggleGroup, ToggleGroupItem } from '../../primitives/toggle-group';
import { CurveHandle } from './curve-handle';
import {
  EASING_PRESETS,
  PLOT,
  cubicBezier,
  fromClientPoint,
  matchEasingPreset,
  toCssTimingFunction,
  toPlot,
  type CurvePoint,
  type EasingCurve,
  type EasingPresetId,
} from './easing-math';

/** Accessible names and preset labels; the app passes localised ones. */
export interface EasingCurveEditorLabels {
  editor: string;
  presets: string;
  firstHandle: string;
  secondHandle: string;
  preset: Record<EasingPresetId, string>;
}

/** Default (English) labels. */
export const DEFAULT_EASING_LABELS: EasingCurveEditorLabels = {
  editor: 'Easing curve',
  presets: 'Easing presets',
  firstHandle: 'First control point',
  secondHandle: 'Second control point',
  preset: {
    linear: 'Linear',
    easeIn: 'Ease in',
    easeOut: 'Ease out',
    easeInOut: 'Ease in-out',
    back: 'Back',
    hold: 'Hold',
  },
};

/** Props of {@link EasingCurveEditor}. */
export interface EasingCurveEditorProps {
  value: EasingCurve;
  /** Live updates while a handle is dragged. */
  onChange: (value: EasingCurve) => void;
  /** End of a gesture (handle released, key press, preset chosen): one undo step. */
  onCommit?: ((value: EasingCurve) => void) | undefined;
  labels?: Partial<Omit<EasingCurveEditorLabels, 'preset'>> & {
    preset?: Partial<Record<EasingPresetId, string>> | undefined;
  };
  className?: string | undefined;
}

const PRESET_ORDER: readonly EasingPresetId[] = [
  'linear',
  'easeIn',
  'easeOut',
  'easeInOut',
  'back',
  'hold',
];
const START = toPlot({ x: 0, y: 0 });
const END = toPlot({ x: 1, y: 1 });

/** SVG path of the curve: the Bézier itself, or a step for `hold`. */
function curvePath(curve: EasingCurve): string {
  if (curve.kind === 'hold') {
    return `M ${String(START.x)} ${String(START.y)} H ${String(END.x)} V ${String(END.y)}`;
  }
  const first = toPlot({ x: curve.x1, y: curve.y1 });
  const second = toPlot({ x: curve.x2, y: curve.y2 });
  return [START, first, second, END]
    .map(
      (point, index) =>
        `${index === 0 ? 'M' : index === 1 ? 'C' : ''} ${String(point.x)} ${String(point.y)}`,
    )
    .join(' ');
}

/**
 * Edits the easing between two keyframes: a cubic Bézier graph with two draggable (and
 * keyboard-movable) control points, presets (linear, ease in/out, back, hold) and an animated
 * preview. Emits `onChange` while dragging and `onCommit` once per gesture.
 */
export function EasingCurveEditor({
  value,
  onChange,
  onCommit,
  labels: overrides,
  className,
}: EasingCurveEditorProps) {
  const labels = {
    ...DEFAULT_EASING_LABELS,
    ...overrides,
    preset: { ...DEFAULT_EASING_LABELS.preset, ...overrides?.preset },
  };
  const svgRef = useRef<SVGSVGElement>(null);
  const draggedHandle = useRef<{ index: 1 | 2; last: EasingCurve } | null>(null);
  const preset = matchEasingPreset(value);

  const withHandle = (curve: EasingCurve, index: 1 | 2, point: CurvePoint): EasingCurve => {
    const base = curve.kind === 'cubicBezier' ? curve : cubicBezier(0, 0, 1, 1);
    return index === 1
      ? cubicBezier(point.x, point.y, base.x2, base.y2)
      : cubicBezier(base.x1, base.y1, point.x, point.y);
  };

  const drag = usePointerDrag({
    onMove: ({ clientX, clientY }) => {
      const state = draggedHandle.current;
      const rect = svgRef.current?.getBoundingClientRect();
      if (state === null || rect === undefined || rect.width === 0) {
        return;
      }
      state.last = withHandle(state.last, state.index, fromClientPoint(clientX, clientY, rect));
      onChange(state.last);
    },
    onEnd: ({ moved }) => {
      const state = draggedHandle.current;
      draggedHandle.current = null;
      if (moved && state !== null) {
        onCommit?.(state.last);
      }
    },
  });

  const startDrag = (index: 1 | 2, event: ReactPointerEvent<SVGGElement>) => {
    draggedHandle.current = { index, last: value };
    drag.onPointerDown(event);
  };

  const moveByKeyboard = (index: 1 | 2) => (point: CurvePoint) => {
    const next = withHandle(value, index, point);
    onChange(next);
    onCommit?.(next);
  };

  return (
    <div className={cn('flex flex-col gap-2', className)}>
      <svg
        ref={svgRef}
        role="group"
        aria-label={labels.editor}
        viewBox={`0 0 ${String(PLOT.size)} ${String(PLOT.size)}`}
        className="aspect-square w-full touch-none rounded-md border border-border bg-input"
      >
        <rect
          x={START.x}
          y={END.y}
          width={END.x - START.x}
          height={START.y - END.y}
          className="fill-none stroke-border"
          strokeDasharray="2 2"
          vectorEffect="non-scaling-stroke"
        />
        <path
          d={curvePath(value)}
          className="fill-none stroke-accent"
          strokeWidth={2}
          vectorEffect="non-scaling-stroke"
        />
        {value.kind === 'cubicBezier' && (
          <>
            <CurveHandle
              point={{ x: value.x1, y: value.y1 }}
              anchor={{ x: 0, y: 0 }}
              label={labels.firstHandle}
              onKeyboardMove={moveByKeyboard(1)}
              onPointerDown={(event) => {
                startDrag(1, event);
              }}
            />
            <CurveHandle
              point={{ x: value.x2, y: value.y2 }}
              anchor={{ x: 1, y: 1 }}
              label={labels.secondHandle}
              onKeyboardMove={moveByKeyboard(2)}
              onPointerDown={(event) => {
                startDrag(2, event);
              }}
            />
          </>
        )}
      </svg>
      <div aria-hidden className="relative h-3 rounded-full bg-control">
        <span
          className="absolute top-0 left-0 size-3 animate-easing-preview rounded-full bg-accent"
          style={{ animationTimingFunction: toCssTimingFunction(value) }}
        />
      </div>
      <ToggleGroup
        type="single"
        variant="segmented"
        size="xs"
        aria-label={labels.presets}
        value={preset ?? ''}
        onValueChange={(next) => {
          if (next === '') {
            return;
          }
          const curve = EASING_PRESETS[next as EasingPresetId];
          onChange(curve);
          onCommit?.(curve);
        }}
        className="flex-wrap"
      >
        {PRESET_ORDER.map((id) => (
          <ToggleGroupItem key={id} value={id}>
            {labels.preset[id]}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
