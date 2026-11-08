import type {
  Animatable,
  Asset,
  Flicks,
  ItemEntity,
  ItemKind,
  ProjectSession,
  Vec2,
} from '@se/protocol';

import { SECOND, graphicItem, textKind, transform } from './graphics-fixture';
import { IDS, sequence, session, videoTrack } from './session-fixture';

/** Id of the fixture's 1920 × 1080 video asset. */
export const VIDEO_ASSET = 'asset-video';

/** A decodable 1920 × 1080 H.264 video asset of 30 s. */
export function videoAsset(id = VIDEO_ASSET, width = 1920, height = 1080): Asset {
  return {
    id,
    originalPath: `/media/${id}.mp4`,
    relativeHint: null,
    kind: 'video',
    mediaInfo: {
      duration: (30 * SECOND) as Flicks,
      container: 'mp4',
      videoStream: {
        codec: 'h264',
        profile: null,
        level: null,
        codecTag: null,
        pixelFormat: null,
        bitDepth: null,
        chromaSubsampling: null,
        width,
        height,
        frameRate: { numerator: 25, denominator: 1 },
        rotation: 0,
      },
      audioStreams: [],
    },
    fingerprint: null,
    derived: { mezzanine: null, proxy: null },
    analysis: { audio: null, scenes: null, transcript: null },
    decodability: { video: 'decodable', audio: 'noStream' },
    missing: false,
  };
}

const still = <T>(value: T): Animatable<T> => ({ kind: 'static', value });

/** A clip of the fixture's video, framed by `reframe`. */
export function clipKind(reframe: Extract<ItemKind, { kind: 'mediaClip' }>['reframe']): ItemKind {
  return {
    kind: 'mediaClip',
    assetId: VIDEO_ASSET,
    sourceStart: 0 as Flicks,
    speed: 1,
    transform: transform(),
    volumeDb: still(0),
    fadeIn: 0 as Flicks,
    fadeOut: 0 as Flicks,
    fadeInShape: 'linear',
    fadeOutShape: 'linear',
    reframe,
  };
}

/** A fill reframe around `focus`. */
export function fill(focus: Vec2 = { x: 0.5, y: 0.5 }) {
  return { kind: 'fill' as const, focus: still(focus) };
}

/**
 * A 1920 × 1080 sequence: V1 holds the clip `wide` (fit), V2 the title `title` centred with a
 * 1600 px wrapping width, the 3 s clip `filled` (fill) sits on V1 after `wide`.
 */
export function reframeSession(): ProjectSession {
  const base = session();
  const title = textKind();
  const wideTitle: ItemKind =
    title.kind === 'text'
      ? {
          ...title,
          content: 'Un titre très large',
          style: { ...title.style, maxWidth: 1600, fontSize: still(140) },
        }
      : title;
  const filled: ItemEntity = {
    ...graphicItem('filled', clipKind(fill())),
    timelineRange: { start: (4 * SECOND) as Flicks, duration: (3 * SECOND) as Flicks },
  };
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [sequence([IDS.v1, IDS.v2])],
      tracks: [videoTrack(IDS.v1, ['wide', 'filled']), videoTrack(IDS.v2, ['title'])],
      items: [
        graphicItem('wide', clipKind({ kind: 'fit' })),
        filled,
        { ...graphicItem('title', wideTitle), trackId: IDS.v2 },
      ],
      assets: [videoAsset()],
    },
  };
}
