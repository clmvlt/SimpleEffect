import type { Format, FrameRate } from '@se/protocol';
import type { AspectRatio } from '@se/ui';

import { aspectRatioOf } from './format-label';
import { defaultResolutionFor, isValidResolution } from './resolutions';

/** A frame format being edited: its ratio, size and rates (the fields of `FormatFields`). */
export interface FormatDraft {
  readonly ratio: AspectRatio;
  readonly width: number;
  readonly height: number;
  readonly frameRate: FrameRate;
  /** Audio sample rate, kept as it is (not edited yet). */
  readonly sampleRate: number;
}

/** The draft of an existing format. */
export function draftFromFormat(format: Format): FormatDraft {
  return {
    ratio: aspectRatioOf(format.width, format.height),
    width: format.width,
    height: format.height,
    frameRate: format.frameRate,
    sampleRate: format.sampleRate,
  };
}

/** `draft` with another ratio and that ratio's default resolution (1080 on the shorter side). */
export function withDraftRatio<T extends FormatDraft>(draft: T, ratio: AspectRatio): T {
  return { ...draft, ratio, ...defaultResolutionFor(ratio) };
}

/** Whether the core accepts the draft's frame size (even sides between 2 and 16 384). */
export function isDraftSizeValid(draft: FormatDraft): boolean {
  return isValidResolution(draft);
}

/** The format a draft describes. */
export function draftToFormat(draft: FormatDraft): Format {
  return {
    width: draft.width,
    height: draft.height,
    frameRate: draft.frameRate,
    sampleRate: draft.sampleRate,
  };
}
