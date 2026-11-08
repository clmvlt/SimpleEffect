import type {
  Asset,
  Flicks,
  ItemEntity,
  SequenceEntity,
  TrackEntity,
  TrackKind,
} from '@se/protocol';

/** Builders of the transcript panel's tests (a 25 fps sequence, tracks, items, assets). */

/** `value` seconds in flicks. */
export function seconds(value: number): Flicks {
  return Math.round(value * 705_600_000) as Flicks;
}

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
    kind: { kind: 'caption', text: 'Bonjour' },
    ...overrides,
  };
}

/** A track of `kind` holding `itemIds` in `sequence-1`. */
export function testTrack(id: string, kind: TrackKind, itemIds: string[] = []): TrackEntity {
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
  };
}

/** The 25 fps HD sequence `sequence-1` holding `trackIds`. */
export function testSequence(trackIds: string[]): SequenceEntity {
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
    markerIds: [],
  };
}

/** A media file with sound at `path`. */
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
