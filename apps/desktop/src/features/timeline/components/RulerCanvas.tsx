import type { FrameRate } from '@se/protocol';
import { useEffect, useRef } from 'react';

import { prepareCanvas, tokenValue } from '../../../shared/canvas';
import { rulerTicks } from '../services/ruler-ticks';
import type { Viewport } from '../services/viewport';

/** Props of {@link RulerCanvas}. */
export interface RulerCanvasProps {
  readonly viewport: Viewport;
  readonly frameRate: FrameRate;
  readonly height: number;
}

const MAJOR_TICK = 10;
const MINOR_TICK = 4;
/** Top of the timecode labels: the band above them holds the markers. */
const LABEL_TOP = 12;

/** The graduations and timecode labels of the ruler, drawn in a canvas (redrawn on zoom/scroll). */
export function RulerCanvas({ viewport, frameRate, height }: RulerCanvasProps) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const { pixelsPerSecond, scrollTime, width } = viewport;
  useEffect(() => {
    const element = canvas.current;
    if (element === null || width <= 0) return;
    const context = prepareCanvas(element, width, height);
    if (context === null) return;
    const ticks = rulerTicks({ pixelsPerSecond, scrollTime, width }, frameRate);
    context.fillStyle = tokenValue('--color-border-strong');
    for (const x of ticks.minor)
      context.fillRect(Math.round(x), height - MINOR_TICK, 1, MINOR_TICK);
    for (const { x } of ticks.major)
      context.fillRect(Math.round(x), height - MAJOR_TICK, 1, MAJOR_TICK);
    context.fillStyle = tokenValue('--color-text-subtle');
    context.font = `10px ${tokenValue('--font-mono')}`;
    context.textBaseline = 'top';
    for (const { x, label } of ticks.major) context.fillText(label, Math.round(x) + 3, LABEL_TOP);
  }, [pixelsPerSecond, scrollTime, width, height, frameRate]);
  return (
    <canvas
      ref={canvas}
      aria-hidden
      className="pointer-events-none absolute inset-0"
      style={{ width, height }}
    />
  );
}
