import type {
  CoreClient,
  Flicks,
  MarkerColor,
  MarkerId,
  MarkerPatch,
  SequenceId,
  TrackId,
  TrackKind,
  TrackPatch,
} from '@se/protocol';

/**
 * Protocol calls of the timeline. Each is one transaction in the core (one undo step); the change
 * comes back as a `projectPatched` event that updates the mirror.
 */
export interface TimelineApi {
  readonly addTrack: (
    sequenceId: SequenceId,
    kind: TrackKind,
    index: number | null,
  ) => Promise<TrackId>;
  readonly removeTrack: (trackId: TrackId) => Promise<void>;
  readonly updateTrack: (trackId: TrackId, patch: TrackPatch) => Promise<void>;
  readonly reorderTracks: (sequenceId: SequenceId, order: readonly TrackId[]) => Promise<void>;
  readonly addMarker: (
    sequenceId: SequenceId,
    time: Flicks,
    label: string,
    color: MarkerColor | null,
  ) => Promise<MarkerId>;
  readonly updateMarker: (markerId: MarkerId, patch: MarkerPatch) => Promise<void>;
  readonly removeMarker: (markerId: MarkerId) => Promise<void>;
  readonly createSequence: (name: string) => Promise<SequenceId>;
}

/** The timeline calls for project `projectId` over `client`. */
export function timelineApi(client: Pick<CoreClient, 'send'>, projectId: string): TimelineApi {
  return {
    addTrack: async (sequenceId, trackKind, index) =>
      (await client.send({ kind: 'addTrack', projectId, sequenceId, trackKind, index })).trackId,
    removeTrack: async (trackId) => {
      await client.send({ kind: 'removeTrack', projectId, trackId });
    },
    updateTrack: async (trackId, patch) => {
      await client.send({ kind: 'updateTrack', projectId, trackId, patch });
    },
    reorderTracks: async (sequenceId, order) => {
      await client.send({ kind: 'reorderTracks', projectId, sequenceId, order: [...order] });
    },
    addMarker: async (sequenceId, time, label, color) =>
      (await client.send({ kind: 'addMarker', projectId, sequenceId, time, label, color }))
        .markerId,
    updateMarker: async (markerId, patch) => {
      await client.send({ kind: 'updateMarker', projectId, markerId, patch });
    },
    removeMarker: async (markerId) => {
      await client.send({ kind: 'removeMarker', projectId, markerId });
    },
    createSequence: async (name) =>
      (await client.send({ kind: 'createSequence', projectId, name, format: null })).sequenceId,
  };
}
