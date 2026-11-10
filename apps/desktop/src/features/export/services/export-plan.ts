import type {
  Asset,
  AssetId,
  CaptionStyle,
  ExportSettings,
  Flicks,
  FontFamily,
  Format,
  ItemEntity,
  ItemId,
  SequenceEntity,
  TrackEntity,
  TrackId,
} from '@se/protocol';

import { flicksToSeconds } from '../../../shared/time';
import { settingsOf, type ExportDraft } from './export-draft';
import {
  exportWarnings,
  isBlocked,
  type EncoderSupport,
  type ExportWarning,
} from './export-warnings';
import { expandFileName, fileNameWithExtension } from './file-name';
import { estimatedBytes } from './size-estimate';

/** The part of a sequence an export covers (`end` exclusive). */
export interface ExportSpan {
  readonly start: Flicks;
  readonly end: Flicks;
}

/** What an export plan is computed from (the dialog's state and the project). */
export interface ExportPlanInput {
  readonly draft: ExportDraft;
  readonly format: Format;
  readonly span: ExportSpan;
  readonly template: string;
  readonly names: { readonly project: string; readonly sequence: string; readonly preset: string };
  readonly now: Date;
  readonly support: EncoderSupport | null;
  readonly aspect: { readonly width: number; readonly height: number } | null;
  readonly items: readonly ItemEntity[];
  readonly assets: Readonly<Record<AssetId, Asset>>;
  readonly captionStyles: readonly CaptionStyle[];
  readonly families: readonly FontFamily[] | null;
}

/** What the dialog shows and sends for an export. */
export interface ExportPlan {
  readonly settings: ExportSettings;
  /** The file's name with its extension. */
  readonly fileName: string;
  readonly seconds: number;
  readonly estimatedBytes: number;
  readonly warnings: readonly ExportWarning[];
  /** Whether a warning prevents the export. */
  readonly blocked: boolean;
  /** Whether ffmpeg encodes the AAC sound (no WebCodecs AAC encoder here). */
  readonly aacByFfmpeg: boolean;
}

/** The items of `sequence`, track by track. */
export function sequenceItems(
  sequence: SequenceEntity,
  tracks: Readonly<Record<TrackId, TrackEntity>>,
  items: Readonly<Record<ItemId, ItemEntity>>,
): ItemEntity[] {
  return sequence.trackIds.flatMap((trackId) =>
    (tracks[trackId]?.itemIds ?? []).flatMap((itemId) => {
      const item = items[itemId];
      return item === undefined ? [] : [item];
    }),
  );
}

/** Where the last item ends (zero for an empty sequence). */
export function contentEnd(items: readonly ItemEntity[]): Flicks {
  let end = 0;
  for (const { timelineRange } of items) {
    end = Math.max(end, timelineRange.start + timelineRange.duration);
  }
  return end as Flicks;
}

/** The settings, file name, size estimate and warnings of an export. */
export function exportPlan(input: ExportPlanInput): ExportPlan {
  const aacByFfmpeg = input.draft.audio?.codec === 'aac' && input.support?.audio === 'none';
  const settings = settingsOf(input.draft, input.format, aacByFfmpeg ? 'ffmpeg' : 'webCodecs');
  const name = expandFileName(input.template, {
    ...input.names,
    date: input.now,
    ...(settings.video === null
      ? {}
      : { width: settings.video.width, height: settings.video.height }),
  });
  const seconds = flicksToSeconds(Math.max(0, input.span.end - input.span.start) as Flicks);
  const { frameRate } = input.format;
  const warnings = exportWarnings({
    items: input.items,
    assets: input.assets,
    captionStyles: input.captionStyles,
    families: input.families,
    format: input.format,
    start: input.span.start,
    end: input.span.end,
    aspect: input.aspect,
    support: input.support,
    aacByFfmpeg,
  });
  return {
    settings,
    fileName: fileNameWithExtension(name, settings.container),
    seconds,
    estimatedBytes: estimatedBytes(settings, seconds, frameRate.numerator / frameRate.denominator),
    warnings,
    blocked: isBlocked(warnings),
    aacByFfmpeg,
  };
}
