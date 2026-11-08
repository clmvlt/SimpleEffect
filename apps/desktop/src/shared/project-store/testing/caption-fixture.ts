import type { CaptionWord, Flicks, ItemEntity, ProjectSession } from '@se/protocol';
import { presetStyle } from '@se/engine/caption-styles';

import { sequence, session, videoTrack } from './session-fixture';

/** `value` seconds in flicks. */
export function captionSeconds(value: number): Flicks {
  return Math.round(value * 705_600_000) as Flicks;
}

/** Words of a caption, one each 0.3 s from its start. */
function words(texts: readonly string[]): CaptionWord[] {
  return texts.map((text, index) => ({
    text,
    start: captionSeconds(index * 0.3),
    end: captionSeconds(index * 0.3 + 0.24),
    emphasis: false,
  }));
}

/** A caption on `trackId` over `[start, start + duration)` seconds, in `styleId`. */
export function timedCaption(
  id: string,
  trackId: string,
  start: number,
  duration: number,
  texts: readonly string[],
  styleId = 'style-1',
): ItemEntity {
  return {
    trackId,
    id,
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: captionSeconds(start), duration: captionSeconds(duration) },
    linkGroup: null,
    kind: { kind: 'caption', words: words(texts), styleId },
  };
}

/**
 * The fixture sequence (25 fps HD, « Film »): track `v2` holds two captions (« Bonjour à tous »
 * 0–1 s, « on commence » 1–2 s) sharing the karaoke style `style-1`; `v1` holds « un titre »
 * (5–6 s) in the minimal style `style-2`.
 */
export function captionSession(): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      project: {
        ...base.snapshot.project,
        captionStyles: [presetStyle('karaoke', 'style-1'), presetStyle('minimal', 'style-2')],
      },
      sequences: [sequence(['v1', 'v2'])],
      tracks: [videoTrack('v1', ['caption-3']), videoTrack('v2', ['caption-1', 'caption-2'])],
      items: [
        timedCaption('caption-1', 'v2', 0, 1, ['Bonjour', 'à', 'tous']),
        timedCaption('caption-2', 'v2', 1, 1, ['on', 'commence']),
        timedCaption('caption-3', 'v1', 5, 1, ['un', 'titre'], 'style-2'),
      ],
      markers: [],
      assets: [],
    },
  };
}
