/**
 * Document builders for engine tests (unit and golden-frame): compact ways to write sequences,
 * items and assets. Not part of the public API.
 */
import type {
  Animatable,
  Asset,
  AssetId,
  Color,
  FadeShape,
  Flicks,
  FrameRate,
  ImageFit,
  ItemKind,
  Reframe,
  ShapeItem,
  Stroke,
  TextItem,
  TextStyle,
  Transform2D,
  TransitionAlignment,
  TransitionKind,
  Vec2,
} from '@se/protocol';

import type { EvaluableItem, EvaluableSequence, EvaluableTrack } from '../evaluate/evaluate';
import type { EvaluableTransition } from '../evaluate/transitions';
import { FLICKS_PER_SECOND } from '../time/flicks';

/** `count` seconds in flicks. */
export function seconds(count: number): Flicks {
  return Math.round(count * FLICKS_PER_SECOND) as Flicks;
}

/** Frame `index` at `rate`, in flicks (exact). */
export function frameTime(index: number, rate: FrameRate): Flicks {
  return ((index * FLICKS_PER_SECOND * rate.denominator) / rate.numerator) as Flicks;
}

/** 29.97 fps. */
export const NTSC_30: FrameRate = { numerator: 30000, denominator: 1001 };

/** A constant property. */
export function still<T>(value: T): Animatable<T> {
  return { kind: 'static', value };
}

/** Overrides of a transform; each may be a constant or an animatable. */
export interface TransformSpec {
  readonly position?: Vec2 | Animatable<Vec2>;
  readonly scale?: Vec2 | Animatable<Vec2>;
  readonly rotation?: number | Animatable<number>;
  readonly anchor?: Vec2 | Animatable<Vec2>;
  readonly opacity?: number | Animatable<number>;
  readonly blur?: number | Animatable<number>;
}

function animatable<T>(value: T | Animatable<T> | undefined, fallback: T): Animatable<T> {
  if (value === undefined) return still(fallback);
  if (typeof value === 'object' && value !== null && 'kind' in value) return value;
  return still(value as T);
}

/** A transform centred in the frame at natural size, with `spec` applied. */
export function transform(spec: TransformSpec = {}): Transform2D {
  return {
    position: animatable(spec.position, { x: 0.5, y: 0.5 }),
    scale: animatable(spec.scale, { x: 1, y: 1 }),
    rotation: animatable(spec.rotation, 0),
    anchor: animatable(spec.anchor, { x: 0.5, y: 0.5 }),
    opacity: animatable(spec.opacity, 1),
    blur: animatable(spec.blur, 0),
  };
}

/** An outline of `width` design pixels (`null`: none). */
export function stroke(color: Color | null = null, width = 4): Stroke {
  return { enabled: color !== null, color: still(color ?? '#000000'), width: still(width) };
}

/** The default text style (white Inter 96 px semi-bold, centred), with `overrides`. */
export function textStyle(overrides: Partial<TextStyle> = {}): TextStyle {
  return {
    fontFamily: 'Inter',
    fontWeight: 600,
    italic: false,
    fontSize: still(96),
    color: still('#ffffff'),
    stroke: stroke(),
    shadow: {
      enabled: false,
      color: still('#000000b3'),
      blur: still(8),
      offset: still({ x: 0, y: 4 }),
    },
    background: {
      enabled: false,
      color: still('#000000b3'),
      padding: { x: 24, y: 12 },
      cornerRadius: 8,
    },
    letterSpacing: still(0),
    lineHeight: 1.2,
    align: 'center',
    maxWidth: null,
    textCase: 'normal',
    ...overrides,
  };
}

/** A text item. */
export function text(
  content: string,
  options: {
    readonly style?: Partial<TextStyle>;
    readonly transform?: TransformSpec;
    readonly reveal?: number | Animatable<number>;
  } = {},
): ItemKind {
  const item: TextItem = {
    content,
    style: textStyle(options.style),
    transform: transform(options.transform),
    reveal: animatable(options.reveal, 1),
  };
  return { kind: 'text', ...item };
}

let nextId = 1;

/** A fresh UUIDv7-shaped id for tests. */
export function testId(): string {
  const index = (nextId++).toString(16).padStart(12, '0');
  return `00000000-0000-7000-8000-${index}`;
}

/** An item covering `[start, start + duration)`. */
export function item(start: Flicks, duration: Flicks, kind: ItemKind): EvaluableItem {
  return { id: testId(), timelineRange: { start, duration }, kind };
}

/** A reframe mode: a name (fill centred) or the domain value. */
function reframeOf(reframe: Reframe | 'fit' | 'fill' | 'custom'): Reframe {
  if (typeof reframe !== 'string') return reframe;
  return reframe === 'fill'
    ? { kind: 'fill', focus: still({ x: 0.5, y: 0.5 }) }
    : { kind: reframe };
}

/** A media clip of `assetId` starting at `sourceStart` in the asset. */
export function clip(
  assetId: AssetId,
  options: {
    readonly sourceStart?: Flicks;
    readonly speed?: number;
    readonly reframe?: Reframe | 'fit' | 'fill' | 'custom';
    readonly fadeIn?: Flicks;
    readonly fadeOut?: Flicks;
    readonly fadeInShape?: FadeShape;
    readonly fadeOutShape?: FadeShape;
    readonly volumeDb?: Animatable<number>;
    readonly transform?: TransformSpec;
  } = {},
): ItemKind {
  return {
    kind: 'mediaClip',
    assetId,
    sourceStart: options.sourceStart ?? (0 as Flicks),
    speed: options.speed ?? 1,
    transform: transform(options.transform),
    volumeDb: options.volumeDb ?? still(0),
    fadeIn: options.fadeIn ?? (0 as Flicks),
    fadeOut: options.fadeOut ?? (0 as Flicks),
    fadeInShape: options.fadeInShape ?? 'linear',
    fadeOutShape: options.fadeOutShape ?? 'linear',
    reframe: reframeOf(options.reframe ?? 'fit'),
  };
}

/** Options shared by the shape builders. */
export interface ShapeOptions {
  readonly transform?: TransformSpec;
  readonly stroke?: Stroke;
  readonly coverFrame?: boolean;
}

function shape(
  geometry: ShapeItem['geometry'],
  fill: Color | Animatable<Color>,
  options: ShapeOptions,
): ItemKind {
  return {
    kind: 'shape',
    geometry,
    fill: animatable(fill, '#000000'),
    stroke: options.stroke ?? stroke(),
    coverFrame: options.coverFrame ?? false,
    transform: transform(options.transform),
  };
}

/** A filled rectangle of `width` × `height` design pixels. */
export function rectangle(
  fill: Color | Animatable<Color>,
  width: number,
  height: number,
  options: ShapeOptions & { readonly cornerRadius?: number } = {},
): ItemKind {
  const geometry = {
    kind: 'rectangle' as const,
    width,
    height,
    cornerRadius: options.cornerRadius ?? 0,
  };
  return shape(geometry, fill, options);
}

/** A filled ellipse of `width` × `height` design pixels. */
export function ellipse(
  fill: Color | Animatable<Color>,
  width: number,
  height: number,
  spec: TransformSpec = {},
  options: ShapeOptions = {},
): ItemKind {
  return shape({ kind: 'ellipse', width, height }, fill, { ...options, transform: spec });
}

/** A line of `length` × `thickness` design pixels. */
export function line(
  fill: Color,
  length: number,
  thickness: number,
  options: ShapeOptions = {},
): ItemKind {
  return shape({ kind: 'line', length, thickness }, fill, options);
}

/** An image item of `assetId`. */
export function image(assetId: AssetId, spec: TransformSpec = {}, fit: ImageFit = 'fit'): ItemKind {
  return { kind: 'image', assetId, fit, transform: transform(spec) };
}

/** A video track holding `items` (sorted by start) and `transitions`. */
export function videoTrack(
  items: readonly EvaluableItem[],
  hidden = false,
  transitions: readonly EvaluableTransition[] = [],
): EvaluableTrack {
  return { kind: 'video', hidden, items, transitions };
}

/** A cross-fade of `duration` on the cut between `left` and `right`. */
export function crossFade(
  left: EvaluableItem,
  right: EvaluableItem,
  duration: Flicks,
  options: {
    readonly alignment?: TransitionAlignment;
    readonly kind?: TransitionKind;
  } = {},
): EvaluableTransition {
  return {
    id: testId(),
    kind: options.kind ?? 'crossDissolve',
    leftItem: left.id,
    rightItem: right.id,
    duration,
    alignment: options.alignment ?? 'center',
  };
}

/** A sequence of `width` × `height` whose tracks stack bottom first. */
export function sequence(
  width: number,
  height: number,
  tracks: readonly EvaluableTrack[],
): EvaluableSequence {
  return { format: { width, height }, tracks };
}

/** Facts of a video asset: coded size, rate, rotation, ready proxy and mezzanine. */
export interface VideoAssetSpec {
  readonly id?: AssetId;
  readonly width: number;
  readonly height: number;
  readonly frameRate?: FrameRate;
  readonly duration?: Flicks;
  readonly rotation?: number;
  readonly proxy?: boolean;
  readonly mezzanine?: boolean;
  readonly decodable?: boolean;
}

/** A probed, decodable video asset. */
export function videoAsset(spec: VideoAssetSpec): Asset {
  const ready = (relativePath: string) => ({ relativePath, status: { kind: 'ready' as const } });
  return {
    id: spec.id ?? testId(),
    originalPath: '/media/clip.mp4',
    relativeHint: null,
    kind: 'video',
    mediaInfo: {
      duration: spec.duration ?? seconds(3),
      container: 'mov,mp4,m4a,3gp,3g2,mj2',
      videoStream: {
        codec: 'h264',
        profile: 'High',
        level: 30,
        codecTag: 'avc1',
        pixelFormat: 'yuv420p',
        bitDepth: 8,
        chromaSubsampling: '4:2:0',
        width: spec.width,
        height: spec.height,
        frameRate: spec.frameRate ?? NTSC_30,
        rotation: spec.rotation ?? 0,
      },
      audioStreams: [],
    },
    fingerprint: null,
    derived: {
      mezzanine: spec.mezzanine === true ? ready('cache/mezzanine.mp4') : null,
      proxy: spec.proxy === true ? ready('cache/proxy-540.mp4') : null,
    },
    analysis: { audio: null, scenes: null, transcript: null },
    decodability: {
      video: spec.decodable === false ? 'undecodable' : 'decodable',
      audio: 'noStream',
    },
    missing: false,
  };
}

/** A probed image asset of `width` × `height`. */
export function imageAsset(width: number, height: number, id?: AssetId): Asset {
  const base = videoAsset({ width, height, ...(id === undefined ? {} : { id }) });
  const stream = base.mediaInfo?.videoStream ?? null;
  return {
    ...base,
    kind: 'image',
    originalPath: '/media/still.png',
    mediaInfo: {
      duration: null,
      container: 'png_pipe',
      videoStream: stream === null ? null : { ...stream, codec: 'png', frameRate: null },
      audioStreams: [],
    },
  };
}

/** Assets by id. */
export function assetTable(...assets: Asset[]): Readonly<Record<AssetId, Asset>> {
  return Object.fromEntries(assets.map((asset) => [asset.id, asset]));
}
