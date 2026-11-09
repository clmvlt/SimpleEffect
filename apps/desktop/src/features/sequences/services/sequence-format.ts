import type {
  ClipReframePolicy,
  Format,
  FormatAdaptation,
  ItemEntity,
  ItemId,
  SequenceEntity,
  TrackEntity,
  TrackId,
} from '@se/protocol';
import type { EvaluableItem, EvaluableSequence } from '@se/engine/evaluate';
import { formatAspectRatio } from '@se/ui';

import { draftFromFormat, withDraftRatio, type FormatDraft } from '../../../shared/format';

/** What the « Format de la séquence » and « Créer une version… » dialogs edit. */
export interface SequenceFormatValues extends FormatDraft {
  /** Name of the version (unused when changing the format in place). */
  readonly name: string;
  /** Whether the user edited the name (it follows the format otherwise). */
  readonly nameEdited: boolean;
  /** The clips' reframe policy; `null` follows the format (see {@link defaultClipPolicy}). */
  readonly clipPolicy: ClipReframePolicy | null;
  readonly fitTextWidths: boolean;
  /** A version's clips follow their subject (prompt 26; unused when changing the format). */
  readonly autoReframe: boolean;
}

/** The mirror records a sequence is read from. */
export interface SequenceRecords {
  readonly sequence: SequenceEntity;
  readonly tracks: Readonly<Record<TrackId, TrackEntity>>;
  readonly items: Readonly<Record<ItemId, ItemEntity>>;
}

/** Whether a sequence holds items (its frame rate is then fixed). */
export function holdsItems({ sequence, tracks }: Omit<SequenceRecords, 'items'>): boolean {
  return sequence.trackIds.some((id) => (tracks[id]?.itemIds.length ?? 0) > 0);
}

/** A sequence of the mirror as `evaluate` reads it. */
export function evaluableOf({ sequence, tracks, items }: SequenceRecords): EvaluableSequence {
  return {
    format: sequence.format,
    tracks: sequence.trackIds.flatMap((trackId) => {
      const track = tracks[trackId];
      if (track === undefined) return [];
      const trackItems: EvaluableItem[] = track.itemIds.flatMap((itemId) => {
        const item = items[itemId];
        return item === undefined ? [] : [item];
      });
      return [
        {
          kind: track.kind,
          hidden: track.hidden,
          items: trackItems,
          transitions: track.transitions,
        },
      ];
    }),
  };
}

/**
 * The reframe policy proposed for a change from `from` to `to`: clips fill a frame of another
 * aspect (no bands in a vertical version), keep their mode when only the size changes.
 */
export function defaultClipPolicy(from: Format, to: FormatDraft): ClipReframePolicy {
  const same = from.width * to.height === to.width * from.height;
  return same ? 'keep' : 'fill';
}

/** The policy in effect: the user's choice, else the default for the format. */
export function effectiveClipPolicy(values: SequenceFormatValues, from: Format): ClipReframePolicy {
  return values.clipPolicy ?? defaultClipPolicy(from, values);
}

/** The name of a version: the source's name and the version's ratio (« Montage 9:16 »). */
export function versionName(sourceName: string, draft: FormatDraft): string {
  return `${sourceName} ${formatAspectRatio(draft.ratio)}`;
}

/** The format a version starts from: 9:16 for landscape and square sequences, 16:9 otherwise. */
export function versionDraft(format: Format): FormatDraft {
  const portrait = format.height > format.width;
  return withDraftRatio(
    draftFromFormat(format),
    portrait ? { width: 16, height: 9 } : { width: 9, height: 16 },
  );
}

/** The values a dialog opens with: the current format, or the version's for a version. */
export function initialFormatValues(
  sequence: SequenceEntity,
  mode: 'format' | 'version',
): SequenceFormatValues {
  const draft =
    mode === 'version' ? versionDraft(sequence.format) : draftFromFormat(sequence.format);
  return {
    ...draft,
    name: mode === 'version' ? versionName(sequence.name, draft) : sequence.name,
    nameEdited: false,
    clipPolicy: null,
    fitTextWidths: true,
    autoReframe: mode === 'version',
  };
}

/** `values` with a new format part, the version's name following its ratio until edited. */
export function withFormat(
  values: SequenceFormatValues,
  draft: FormatDraft,
  sourceName: string,
): SequenceFormatValues {
  const name = values.nameEdited ? values.name : versionName(sourceName, draft);
  return { ...values, ...draft, name };
}

/** The adaptation sent with the change. */
export function adaptationOf(values: SequenceFormatValues, from: Format): FormatAdaptation {
  return {
    clipReframe: effectiveClipPolicy(values, from),
    fitTextWidths: values.fitTextWidths,
  };
}

/** Whether the format part differs from `format` (frame size or rate). */
export function formatChanged(values: FormatDraft, format: Format): boolean {
  return (
    values.width !== format.width ||
    values.height !== format.height ||
    values.frameRate.numerator !== format.frameRate.numerator ||
    values.frameRate.denominator !== format.frameRate.denominator
  );
}
