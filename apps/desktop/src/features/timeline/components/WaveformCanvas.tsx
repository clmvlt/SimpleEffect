import { decodeWaveformPeaks, type Asset, type Flicks, type WaveformPeaks } from '@se/protocol';
import { useEffect, useRef, useState } from 'react';

import { prepareCanvas, tokenValue } from '../../../shared/canvas';
import { flicksToSeconds } from '../../../shared/time';
import { useAssetMediaUrl } from '../../media-bin';
import { useCachedResource } from '../hooks/use-cached-resource';
import { peakLevelFor, waveformColumns } from '../services/waveform';

/** Props of {@link WaveformCanvas}. */
export interface WaveformCanvasProps {
  readonly asset: Asset;
  readonly sourceStart: Flicks;
  readonly speed: number;
  readonly pixelsPerSecond: number;
  /** Pixels of the clip hidden left of the canvas. */
  readonly offsetPixels: number;
  readonly width: number;
  readonly height: number;
}

/**
 * Canvas widths are rounded up to this step: zooming changes the visible width of clips on every
 * frame, and resizing a canvas reallocates its pixels.
 */
const CANVAS_WIDTH_STEP = 128;

const readPeaks = async (response: Response) => decodeWaveformPeaks(await response.arrayBuffer());

/** Sample rate assumed before the media info is known. */
const FALLBACK_SAMPLE_RATE = 48_000;

/**
 * The waveform of the visible part of a clip, drawn in a canvas from the cached peaks at the level
 * matching the zoom (one column per pixel, mirrored around the middle line).
 */
export function WaveformCanvas({
  asset,
  sourceStart,
  speed,
  pixelsPerSecond,
  offsetPixels,
  width,
  height,
}: WaveformCanvasProps) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const sampleRate = asset.mediaInfo?.audioStreams[0]?.sampleRate ?? FALLBACK_SAMPLE_RATE;
  const level = peakLevelFor(sampleRate, speed, pixelsPerSecond);
  const url = useAssetMediaUrl(asset, { peaks: level }, 'waveformPeaks');
  const hasSound = (asset.mediaInfo?.audioStreams.length ?? 0) > 0;
  const fetched = useCachedResource(hasSound ? url : null, readPeaks);
  // While another level loads (the zoom changed), keep drawing the previous one.
  const [previous, setPrevious] = useState<WaveformPeaks | null>(null);
  if (fetched !== null && fetched !== previous) setPrevious(fetched);
  const peaks = fetched ?? previous;
  const pixelWidth = Math.max(1, Math.ceil(width));
  const pixelHeight = Math.max(1, Math.floor(height));
  const canvasWidth = Math.ceil(pixelWidth / CANVAS_WIDTH_STEP) * CANVAS_WIDTH_STEP;

  useEffect(() => {
    const element = canvas.current;
    if (element === null || peaks === null) return;
    const context = prepareCanvas(element, canvasWidth, pixelHeight);
    if (context === null) return;
    const columns = waveformColumns(peaks, {
      sourceStartSeconds: flicksToSeconds(sourceStart),
      speed,
      pixelsPerSecond,
      offsetPixels,
      width: pixelWidth,
    });
    const middle = pixelHeight / 2;
    context.fillStyle = tokenValue('--color-item-audio-strong');
    context.beginPath();
    for (let x = 0; x < pixelWidth; x += 1) {
      const top = middle - (columns.maximums[x] ?? 0) * middle;
      const bottom = middle - (columns.minimums[x] ?? 0) * middle;
      context.rect(x, top, 1, Math.max(1, bottom - top));
    }
    context.fill();
  }, [
    peaks,
    sourceStart,
    speed,
    pixelsPerSecond,
    offsetPixels,
    pixelWidth,
    pixelHeight,
    canvasWidth,
  ]);

  return (
    <canvas
      ref={canvas}
      aria-hidden
      className="pointer-events-none absolute bottom-0 left-0"
      style={{ width: canvasWidth, height: pixelHeight }}
    />
  );
}
