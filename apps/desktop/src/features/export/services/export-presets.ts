import type { ExportPreset, Format } from '@se/protocol';

import { outputSize, type ExportDraft } from './export-draft';

/** Presets the app ships (their names are `export.presets.<id>` in `fr.json`). */
export type BuiltInPresetId =
  | 'youtube1080'
  | 'youtube4k'
  | 'youtubeHevc'
  | 'shorts'
  | 'square'
  | 'master'
  | 'soundWav'
  | 'soundAac';

/** A preset the app ships: its settings and the frame shape it is meant for, if any. */
export interface BuiltInPreset {
  readonly id: BuiltInPresetId;
  readonly draft: ExportDraft;
  /** The aspect of the platform it targets (Shorts 9:16, square 1:1): `null` for any. */
  readonly aspect: { readonly width: number; readonly height: number } | null;
}

/**
 * AAC at 192 kb/s: the highest rate every platform's encoder accepts (Windows' AAC encoder
 * offers 96 to 192 kb/s only).
 */
const AAC = { codec: 'aac', bitsPerSecond: 192_000 } as const;
const megabits = (value: number) =>
  ({ kind: 'bitrate', bitsPerSecond: value * 1_000_000 }) as const;

/**
 * The shipped presets. Bitrates follow YouTube's upload recommendations (1080p 8–12 Mb/s,
 * 4K 35–45 Mb/s for H.264); the master keeps the sequence's size in ProRes 422 HQ.
 */
export const BUILT_IN_PRESETS: readonly BuiltInPreset[] = [
  {
    id: 'youtube1080',
    draft: {
      container: 'mp4',
      video: { codec: 'h264', shortSide: 1080, rate: megabits(12) },
      audio: AAC,
    },
    aspect: null,
  },
  {
    id: 'youtube4k',
    draft: {
      container: 'mp4',
      video: { codec: 'h264', shortSide: 2160, rate: megabits(45) },
      audio: AAC,
    },
    aspect: null,
  },
  {
    id: 'youtubeHevc',
    draft: {
      container: 'mp4',
      video: { codec: 'hevc', shortSide: 1080, rate: megabits(8) },
      audio: AAC,
    },
    aspect: null,
  },
  {
    id: 'shorts',
    draft: {
      container: 'mp4',
      video: { codec: 'h264', shortSide: 1080, rate: megabits(12) },
      audio: AAC,
    },
    aspect: { width: 9, height: 16 },
  },
  {
    id: 'square',
    draft: {
      container: 'mp4',
      video: { codec: 'h264', shortSide: 1080, rate: megabits(10) },
      audio: AAC,
    },
    aspect: { width: 1, height: 1 },
  },
  {
    id: 'master',
    draft: {
      container: 'mov',
      video: {
        codec: 'prores422Hq',
        shortSide: 'sequence',
        rate: { kind: 'quality', level: 'veryHigh' },
      },
      audio: { codec: 'pcm', bitsPerSecond: 0 },
    },
    aspect: null,
  },
  {
    id: 'soundWav',
    draft: { container: 'wav', video: null, audio: { codec: 'pcm', bitsPerSecond: 0 } },
    aspect: null,
  },
  { id: 'soundAac', draft: { container: 'm4a', video: null, audio: AAC }, aspect: null },
];

/** The preset chosen in the dialog: shipped, saved by the user, or custom settings. */
export type PresetChoice =
  | { readonly kind: 'builtIn'; readonly id: BuiltInPresetId }
  | { readonly kind: 'saved'; readonly id: string }
  | { readonly kind: 'custom' };

/** The preset chosen by default. */
export const DEFAULT_PRESET_CHOICE: PresetChoice = { kind: 'builtIn', id: 'youtube1080' };

/** The value of a choice in the preset menu. */
export function choiceValue(choice: PresetChoice): string {
  return choice.kind === 'custom' ? 'custom' : `${choice.kind}:${choice.id}`;
}

/** The choice of a preset menu value (the default for an unknown one). */
export function parseChoice(value: string): PresetChoice {
  if (value === 'custom') return { kind: 'custom' };
  const builtIn = BUILT_IN_PRESETS.find((preset) => `builtIn:${preset.id}` === value);
  if (builtIn !== undefined) return { kind: 'builtIn', id: builtIn.id };
  if (value.startsWith('saved:')) return { kind: 'saved', id: value.slice('saved:'.length) };
  return DEFAULT_PRESET_CHOICE;
}

/** The shipped preset `id`. */
export function builtInPreset(id: BuiltInPresetId): BuiltInPreset {
  return (
    BUILT_IN_PRESETS.find((preset) => preset.id === id) ?? (BUILT_IN_PRESETS[0] as BuiltInPreset)
  );
}

/** The settings of a saved preset. */
export function draftOfSaved(preset: ExportPreset): ExportDraft {
  return { container: preset.container, video: preset.video, audio: preset.audio };
}

/**
 * A preset saving `draft` under `name` (`id` chosen by the caller). A picture at the sequence's
 * own size is saved with that sequence's shorter side.
 */
export function savedPresetOf(
  id: string,
  name: string,
  draft: ExportDraft,
  format: Format,
): ExportPreset {
  const video =
    draft.video === null
      ? null
      : {
          codec: draft.video.codec,
          shortSide:
            draft.video.shortSide === 'sequence'
              ? Math.min(...Object.values(outputSize(format, 'sequence')))
              : draft.video.shortSide,
          rate: draft.video.rate,
        };
  const audio = draft.audio === null ? null : { ...draft.audio, encoder: 'webCodecs' as const };
  return { id, name: name.trim(), container: draft.container, video, audio };
}

/** Whether a frame of `format` has `aspect` (within a pixel of rounding). */
export function hasAspect(
  format: Format,
  aspect: { readonly width: number; readonly height: number },
): boolean {
  return (
    Math.abs(format.width * aspect.height - format.height * aspect.width) <=
    Math.max(aspect.width, aspect.height)
  );
}
