import type {
  Flicks,
  ItemEntity,
  ItemKind,
  ProjectPatch,
  ProjectSession,
  SequenceEntity,
  TrackEntity,
} from '@se/protocol';

/** A caption showing `text` as one word over its whole item, in the style `style-1`. */
export function captionKind(text: string): ItemKind {
  return {
    kind: 'caption',
    words: [{ text, start: 0 as Flicks, end: 0 as Flicks, emphasis: false }],
    styleId: 'style-1',
  };
}

/** Ids used by the fixtures. */
export const IDS = {
  project: 'project-1',
  sequence: 'sequence-1',
  v1: 'track-v1',
  v2: 'track-v2',
  item: 'item-1',
} as const;

const HD = {
  width: 1920,
  height: 1080,
  frameRate: { numerator: 25, denominator: 1 },
  sampleRate: 48_000,
};

/** A caption item on `trackId` starting at `start` flicks. */
export function captionItem(id: string, trackId: string, start: number): ItemEntity {
  return {
    trackId,
    id,
    revision: 1,
    origin: { kind: 'user' },
    // Test values: plain numbers are valid flicks.
    timelineRange: { start: start as Flicks, duration: 1000 as Flicks },
    linkGroup: null,
    kind: captionKind(id),
  };
}

/** A video track of the fixture sequence holding `itemIds`. */
export function videoTrack(id: string, itemIds: string[]): TrackEntity {
  return {
    id,
    sequenceId: IDS.sequence,
    kind: 'video',
    name: id,
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

/** The fixture sequence with `trackIds`. */
export function sequence(trackIds: string[], name = 'Film'): SequenceEntity {
  return { id: IDS.sequence, name, format: HD, trackIds, markerIds: [] };
}

/** A session at `revision`: one sequence, tracks V1 (one item) and V2 (empty). */
export function session(revision = 3): ProjectSession {
  return {
    path: '/projects/Film.seproj',
    snapshot: {
      revision,
      project: {
        id: IDS.project,
        name: 'Film',
        settings: { defaultFormat: HD, transcription: { language: 'fr', vocabulary: '' } },
        templates: [],
        brandKit: null,
        captionStyles: [],
      },
      sequences: [sequence([IDS.v1, IDS.v2])],
      tracks: [videoTrack(IDS.v1, [IDS.item]), videoTrack(IDS.v2, [])],
      items: [captionItem(IDS.item, IDS.v1, 0)],
      markers: [],
      assets: [],
    },
    history: { canUndo: false, canRedo: false, undoLabel: null, redoLabel: null },
    recovery: null,
  };
}

/** An empty patch of the fixture project at `revision`. */
export function emptyPatch(revision: number): ProjectPatch {
  return {
    projectId: IDS.project,
    revision,
    actor: { kind: 'user' },
    upserts: { project: null, sequences: [], tracks: [], items: [], markers: [], assets: [] },
    deletes: { sequences: [], tracks: [], items: [], markers: [], assets: [] },
  };
}
