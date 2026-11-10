import type {
  Asset,
  AssetId,
  CaptionStyle,
  Flicks,
  FontFamily,
  Format,
  ItemEntity,
} from '@se/protocol';
import type { AudioEncoderSupport, VideoEncoderSupport } from '@se/engine/export';
import { playbackSource } from '@se/engine/sources';

import { hasAspect } from './export-presets';

/** Something the user should know before exporting; `blocking` ones prevent it. */
export type ExportWarning =
  | { readonly kind: 'emptyRange' }
  | { readonly kind: 'noVideoEncoder' }
  | { readonly kind: 'noAudioEncoder' }
  | { readonly kind: 'missingMedia'; readonly count: number }
  | { readonly kind: 'preparingMedia'; readonly count: number }
  | { readonly kind: 'missingFonts'; readonly families: readonly string[] }
  | {
      readonly kind: 'aspect';
      readonly aspect: { readonly width: number; readonly height: number };
    }
  | { readonly kind: 'softwareEncoding' }
  | { readonly kind: 'aacByFfmpeg' };

/** Encoders found for the chosen settings (`null` while they are being probed). */
export interface EncoderSupport {
  readonly video: VideoEncoderSupport | null;
  readonly audio: AudioEncoderSupport | null;
}

/** What the warnings are computed from. */
export interface WarningInput {
  /** Items of the exported sequence. */
  readonly items: readonly ItemEntity[];
  readonly assets: Readonly<Record<AssetId, Asset>>;
  readonly captionStyles: readonly CaptionStyle[];
  /** The font catalogue, `null` until listed (no font warning then). */
  readonly families: readonly FontFamily[] | null;
  readonly format: Format;
  readonly start: Flicks;
  readonly end: Flicks;
  /** The aspect the chosen preset targets (`null`: any). */
  readonly aspect: { readonly width: number; readonly height: number } | null;
  readonly support: EncoderSupport | null;
  /** Whether AAC sound was handed to ffmpeg (no WebCodecs AAC encoder here). */
  readonly aacByFfmpeg: boolean;
}

/** Kinds that prevent the export. */
const BLOCKING = new Set<ExportWarning['kind']>(['emptyRange', 'noVideoEncoder', 'noAudioEncoder']);

/** Whether `warnings` prevent the export. */
export function isBlocked(warnings: readonly ExportWarning[]): boolean {
  return warnings.some((warning) => BLOCKING.has(warning.kind));
}

/** Whether `warning` prevents the export. */
export function isBlocking(warning: ExportWarning): boolean {
  return BLOCKING.has(warning.kind);
}

function overlaps(item: ItemEntity, start: Flicks, end: Flicks): boolean {
  const { start: itemStart, duration } = item.timelineRange;
  return itemStart < end && itemStart + duration > start;
}

/** How each asset the range shows can be read now. */
function mediaWarnings(input: WarningInput, items: readonly ItemEntity[]): ExportWarning[] {
  const used = new Set<AssetId>();
  for (const item of items) {
    if (item.kind.kind === 'mediaClip' || item.kind.kind === 'image') used.add(item.kind.assetId);
  }
  let missing = 0;
  let preparing = 0;
  for (const assetId of used) {
    const asset = input.assets[assetId];
    const source =
      asset === undefined
        ? 'unavailable'
        : asset.kind === 'image'
          ? asset.missing
            ? 'unavailable'
            : 'original'
          : playbackSource(asset).kind;
    if (source === 'unavailable') missing += 1;
    if (source === 'preparing') preparing += 1;
  }
  const warnings: ExportWarning[] = [];
  if (missing > 0) warnings.push({ kind: 'missingMedia', count: missing });
  if (preparing > 0) warnings.push({ kind: 'preparingMedia', count: preparing });
  return warnings;
}

/** Families of texts and captions the catalogue lacks. */
function missingFamilies(input: WarningInput, items: readonly ItemEntity[]): string[] {
  if (input.families === null) return [];
  const known = new Set(input.families.map((family) => family.name));
  const wanted = new Set<string>();
  for (const item of items) {
    if (item.kind.kind === 'text') wanted.add(item.kind.style.fontFamily);
    if (item.kind.kind === 'caption') {
      const { styleId } = item.kind;
      const style = input.captionStyles.find((candidate) => candidate.id === styleId);
      if (style !== undefined) wanted.add(style.fontFamily);
    }
  }
  return [...wanted].filter((family) => !known.has(family)).sort();
}

function encoderWarnings(support: EncoderSupport | null, aacByFfmpeg: boolean): ExportWarning[] {
  const warnings: ExportWarning[] = [];
  if (support?.video === 'none') warnings.push({ kind: 'noVideoEncoder' });
  if (support?.audio === 'none' && !aacByFfmpeg) warnings.push({ kind: 'noAudioEncoder' });
  if (support?.video === 'software') warnings.push({ kind: 'softwareEncoding' });
  if (aacByFfmpeg) warnings.push({ kind: 'aacByFfmpeg' });
  return warnings;
}

/** What to tell the user about an export before it starts, blocking problems first. */
export function exportWarnings(input: WarningInput): ExportWarning[] {
  const items = input.items.filter((item) => overlaps(item, input.start, input.end));
  const warnings: ExportWarning[] = [];
  if (input.end <= input.start) warnings.push({ kind: 'emptyRange' });
  warnings.push(...encoderWarnings(input.support, input.aacByFfmpeg));
  warnings.push(...mediaWarnings(input, items));
  const families = missingFamilies(input, items);
  if (families.length > 0) warnings.push({ kind: 'missingFonts', families });
  if (input.aspect !== null && !hasAspect(input.format, input.aspect)) {
    warnings.push({ kind: 'aspect', aspect: input.aspect });
  }
  return warnings.sort((left, right) => Number(isBlocking(right)) - Number(isBlocking(left)));
}
