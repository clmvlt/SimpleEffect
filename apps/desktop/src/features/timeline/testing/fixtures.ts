import type {
  Asset,
  Flicks,
  ItemEntity,
  ItemKind,
  SequenceEntity,
  TrackEntity,
  TrackKind,
} from '@se/protocol';
import { captionKind } from '../../../shared/project-store/testing/session-fixture';

/** `value` seconds as flicks (test values: exact multiples). */
export function seconds(value: number): Flicks {
  return Math.round(value * 705_600_000) as Flicks;
}

/** A caption (the simplest item kind). */
export const CAPTION: ItemKind = captionKind('Bonjour');

/** An item on `trackId` from `start` lasting `duration` seconds. */
export function testItem(
  id: string,
  trackId: string,
  start: number,
  duration: number,
  overrides: Partial<ItemEntity> = {},
): ItemEntity {
  return {
    trackId,
    id,
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: seconds(start), duration: seconds(duration) },
    linkGroup: null,
    kind: CAPTION,
    ...overrides,
  };
}

/** A track of `kind` holding `itemIds`. */
export function testTrack(
  id: string,
  kind: TrackKind,
  itemIds: string[] = [],
  overrides: Partial<TrackEntity> = {},
): TrackEntity {
  return {
    id,
    sequenceId: 'sequence-1',
    kind,
    name: id.toUpperCase(),
    muted: false,
    solo: false,
    locked: false,
    hidden: false,
    volumeDb: 0,
    pan: 0,
    itemIds,
    transitions: [],
    ...overrides,
  };
}

/** A 25 fps HD sequence holding `trackIds`. */
export function testSequence(trackIds: string[], markerIds: string[] = []): SequenceEntity {
  return {
    id: 'sequence-1',
    name: 'Film',
    format: {
      width: 1920,
      height: 1080,
      frameRate: { numerator: 25, denominator: 1 },
      sampleRate: 48_000,
    },
    trackIds,
    markerIds,
  };
}

/** A video asset with sound at `path`. */
export function testAsset(id: string, path: string, kind: Asset['kind'] = 'video'): Asset {
  return {
    id,
    originalPath: path,
    relativeHint: null,
    kind,
    mediaInfo: {
      duration: seconds(30),
      container: 'mp4',
      videoStream: null,
      audioStreams: [{ codec: 'aac', profile: null, channels: 2, sampleRate: 48_000 }],
    },
    fingerprint: null,
    derived: { mezzanine: null, proxy: null },
    analysis: { audio: null, scenes: null, transcript: null },
    decodability: { video: 'noStream', audio: 'decodable' },
    missing: false,
  };
}
