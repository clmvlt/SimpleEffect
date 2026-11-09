/**
 * Helpers of the golden-frame tests: render a scene with the real engine, read its pixels, compare
 * them with a reference PNG (or rewrite it with `SE_UPDATE_GOLDEN=1`), read the frame number the
 * fixtures encode.
 */
import type { AssetId, Flicks } from '@se/protocol';
import { expect, inject } from 'vitest';
import { commands } from 'vitest/browser';

import type { MediaVariant } from '../../src/decode/media-variant';
import type { FontLoader } from '../../src/fonts/font-registry';
import { MediabunnyVideoSource } from '../../src/decode/video-source';
import type { PreviewEngine } from '../../src/preview/engine';
import type { RenderScene } from '../../src/preview/frame-renderer';
import type { TemplateRenderer } from '../../src/templates/template-renderer';
import { createPreviewEngine } from '../../src/preview/preview-engine';

declare module 'vitest' {
  export interface ProvidedContext {
    /** `SE_UPDATE_GOLDEN=1`: write references instead of comparing. */
    updateGolden: boolean;
  }
}

/** Fixture URLs of each asset's files. */
export type FixtureFiles = Readonly<Record<AssetId, Partial<Record<MediaVariant, string>>>>;

/** Options of {@link renderScene}. */
export interface RenderOptions {
  readonly proxy?: boolean;
  readonly offscreen?: boolean;
  readonly transparent?: boolean;
  /** Fonts loaded before texts are drawn (`testFonts()` for the bundled Inter). */
  readonly fonts?: FontLoader;
  /** Renders motion template layers (prompt 24). */
  readonly templates?: TemplateRenderer;
}

/** A rendered frame and the engine that drew it (destroy it when done). */
export interface Rendered {
  readonly engine: PreviewEngine;
  readonly canvas: HTMLCanvasElement | OffscreenCanvas;
  readonly pixels: ImageData;
}

/** Creates an engine on a canvas of the sequence size, reading `files`. */
export async function createTestEngine(
  scene: RenderScene,
  files: FixtureFiles,
  options: RenderOptions = {},
): Promise<{ engine: PreviewEngine; canvas: HTMLCanvasElement | OffscreenCanvas }> {
  const { width, height } = scene.sequence.format;
  const canvas =
    options.offscreen === true
      ? new OffscreenCanvas(width, height)
      : document.createElement('canvas');
  const engine = await createPreviewEngine({
    canvas,
    width,
    height,
    preserveDrawingBuffer: true,
    background: options.transparent === true ? 'transparent' : 'black',
    locator: (assetId, variant) => files[assetId]?.[variant] ?? null,
    pool: { createVideoSource: (url) => new MediabunnyVideoSource(url, 'prefer-software') },
    ...(options.fonts === undefined ? {} : { fonts: options.fonts }),
    ...(options.templates === undefined ? {} : { templates: options.templates }),
  });
  engine.setScene(scene);
  engine.setProxy(options.proxy === true);
  return { engine, canvas };
}

/** Draws `scene` at `time` and reads the pixels back. */
export async function renderScene(
  scene: RenderScene,
  time: Flicks,
  files: FixtureFiles = {},
  options: RenderOptions = {},
): Promise<Rendered> {
  const { engine, canvas } = await createTestEngine(scene, files, options);
  engine.setTime(time);
  await engine.settled();
  const state = engine.getState();
  expect(state, state.error ?? undefined).toMatchObject({ status: 'ready', issues: [] });
  return { engine, canvas, pixels: readPixels(canvas) };
}

/** Moves an engine to `time` and reads the frame drawn. */
export async function frameAt(
  engine: PreviewEngine,
  canvas: HTMLCanvasElement | OffscreenCanvas,
  time: Flicks,
): Promise<ImageData> {
  engine.setTime(time);
  await engine.settled();
  return readPixels(canvas);
}

/** The pixels of a canvas (copied through a 2D canvas). */
export function readPixels(canvas: HTMLCanvasElement | OffscreenCanvas): ImageData {
  const copy = new OffscreenCanvas(canvas.width, canvas.height);
  const context = copy.getContext('2d');
  if (context === null) throw new Error('no 2D context');
  context.drawImage(canvas, 0, 0);
  return context.getImageData(0, 0, canvas.width, canvas.height);
}

/** Luminance (0–255) of the pixel at `(x, y)`. */
export function luminanceAt(pixels: ImageData, x: number, y: number): number {
  const offset = (Math.floor(y) * pixels.width + Math.floor(x)) * 4;
  const channel = (index: number) => pixels.data[offset + index] ?? 0;
  return 0.2126 * channel(0) + 0.7152 * channel(1) + 0.0722 * channel(2);
}

/**
 * The frame number a fixture encodes, read from a frame drawn over `area` of the canvas (the
 * video's box). `rotated`: the picture was turned 90° clockwise (bits along the right half).
 */
export function readFrameNumber(
  pixels: ImageData,
  area: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  rotated = false,
): number {
  let value = 0;
  for (let bit = 0; bit < 8; bit += 1) {
    const along = (bit + 0.5) / 8;
    const x = rotated ? area.x + area.width * 0.75 : area.x + area.width * along;
    const y = rotated ? area.y + area.height * along : area.y + area.height * 0.25;
    if (luminanceAt(pixels, x, y) > 128) value |= 1 << bit;
  }
  return value;
}

async function pngBytes(pixels: ImageData): Promise<Uint8Array> {
  const canvas = new OffscreenCanvas(pixels.width, pixels.height);
  canvas.getContext('2d')?.putImageData(pixels, 0, 0);
  const blob = await canvas.convertToBlob({ type: 'image/png' });
  return new Uint8Array(await blob.arrayBuffer());
}

function toBase64(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

async function decodePng(base64: string): Promise<ImageData> {
  const bytes = Uint8Array.from(atob(base64), (character) => character.charCodeAt(0));
  const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  const context = canvas.getContext('2d');
  if (context === null) throw new Error('no 2D context');
  context.drawImage(bitmap, 0, 0);
  bitmap.close();
  return context.getImageData(0, 0, canvas.width, canvas.height);
}

/** How far a frame may stray from its reference. */
export interface Tolerance {
  /** Largest channel difference (0–255) still counted as equal. */
  readonly channel: number;
  /** Share of pixels allowed beyond `channel`. */
  readonly ratio: number;
  /** Compare averages of `downsample` × `downsample` blocks (absorbs anti-aliasing). */
  readonly downsample?: number;
}

/** Default tolerance: GPU rasterisation and colour conversion differ by a few levels at edges. */
export const DEFAULT_TOLERANCE: Tolerance = { channel: 16, ratio: 0.005 };

/**
 * Tolerance of frames with text: glyph anti-aliasing, hinting and advances differ between the
 * platform font rasterisers (DirectWrite, FreeType, CoreText) even for the same font file (Linux
 * CI differed from Windows on 1.3 % of 4 × 4 blocks), so frames are compared as block averages
 * with room for that: a missing or misplaced text still fails. Text tests also check their
 * styles with platform-independent pixel measures (colours, extents, alignment).
 */
export const TEXT_TOLERANCE: Tolerance = { channel: 32, ratio: 0.03, downsample: 4 };

/** Left and right edges of the bright pixels (luminance > 128) in a region, or `null`. */
export function litExtent(
  pixels: ImageData,
  region: { readonly x0: number; readonly x1: number; readonly y0: number; readonly y1: number },
): { readonly left: number; readonly right: number } | null {
  let left = Infinity;
  let right = -Infinity;
  for (let y = region.y0; y < region.y1; y += 1) {
    for (let x = region.x0; x < region.x1; x += 1) {
      if (luminanceAt(pixels, x, y) > 128) {
        left = Math.min(left, x);
        right = Math.max(right, x);
      }
    }
  }
  return left === Infinity ? null : { left, right };
}

/** Averages of `factor` × `factor` blocks of `pixels`. */
export function downsample(pixels: ImageData, factor: number): ImageData {
  const width = Math.floor(pixels.width / factor);
  const height = Math.floor(pixels.height / factor);
  const out = new ImageData(Math.max(1, width), Math.max(1, height));
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      for (let channel = 0; channel < 4; channel += 1) {
        let sum = 0;
        for (let dy = 0; dy < factor; dy += 1) {
          for (let dx = 0; dx < factor; dx += 1) {
            const offset = ((y * factor + dy) * pixels.width + x * factor + dx) * 4 + channel;
            sum += pixels.data[offset] ?? 0;
          }
        }
        out.data[(y * width + x) * 4 + channel] = Math.round(sum / (factor * factor));
      }
    }
  }
  return out;
}

/** Pixels beyond the tolerance and a diff image (red where they differ). */
export function comparePixels(actual: ImageData, expected: ImageData, tolerance: Tolerance) {
  const diff = new ImageData(actual.width, actual.height);
  let differing = 0;
  for (let offset = 0; offset < actual.data.length; offset += 4) {
    let delta = 0;
    for (let channel = 0; channel < 4; channel += 1) {
      const a = actual.data[offset + channel] ?? 0;
      const b = expected.data[offset + channel] ?? 0;
      delta = Math.max(delta, Math.abs(a - b));
    }
    const beyond = delta > tolerance.channel;
    if (beyond) differing += 1;
    diff.data.set(beyond ? [255, 0, 0, 255] : [delta * 4, delta * 4, delta * 4, 255], offset);
  }
  return { differing, ratio: differing / (actual.width * actual.height), diff };
}

/**
 * Compares `pixels` with `references/<name>.png` (next to this file); on mismatch writes
 * `<name>.actual.png` and `<name>.diff.png` there (gitignored) and fails.
 */
export async function expectGolden(
  name: string,
  pixels: ImageData,
  tolerance: Tolerance = DEFAULT_TOLERANCE,
): Promise<void> {
  const path = `./test/golden/references/${name}.png`;
  if (inject('updateGolden')) {
    await commands.writeFile(path, toBase64(await pngBytes(pixels)), 'base64');
    return;
  }
  let reference: ImageData;
  try {
    reference = await decodePng(await commands.readFile(path, 'base64'));
  } catch {
    throw new Error(`No reference image ${path}: run the tests once with SE_UPDATE_GOLDEN=1.`);
  }
  expect([pixels.width, pixels.height]).toEqual([reference.width, reference.height]);
  const factor = tolerance.downsample ?? 1;
  const result =
    factor > 1
      ? comparePixels(downsample(pixels, factor), downsample(reference, factor), tolerance)
      : comparePixels(pixels, reference, tolerance);
  if (result.ratio > tolerance.ratio) {
    await commands.writeFile(
      `./test/golden/references/${name}.actual.png`,
      toBase64(await pngBytes(pixels)),
      'base64',
    );
    await commands.writeFile(
      `./test/golden/references/${name}.diff.png`,
      toBase64(await pngBytes(result.diff)),
      'base64',
    );
  }
  expect(
    result.ratio,
    `${String(result.differing)} pixels differ from ${path}`,
  ).toBeLessThanOrEqual(tolerance.ratio);
}
