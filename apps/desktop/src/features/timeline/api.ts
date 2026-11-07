import type {
  CoreClient,
  Flicks,
  ItemId,
  ItemPlacement,
  ItemTrim,
  LinkGroupId,
  MarkerColor,
  MarkerId,
  MarkerPatch,
  PastedItem,
  PlaceMediaRequest,
  PlacementMode,
  SequenceId,
  TrackId,
  TrackKind,
  TrackPatch,
} from '@se/protocol';

/** A gap to resize: its track and bounds. */
export interface GapBounds {
  readonly trackId: TrackId;
  readonly start: Flicks;
  readonly end: Flicks;
}

/**
 * Protocol calls of the timeline. Each is one transaction in the core (one undo step); the change
 * comes back as a `projectPatched` event that updates the mirror (se-core sends it before the
 * response, so the mirror is up to date when a call resolves).
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
  readonly placeMedia: (request: Omit<PlaceMediaRequest, 'projectId'>) => Promise<ItemId[]>;
  readonly moveItems: (moves: readonly ItemPlacement[], mode: PlacementMode) => Promise<void>;
  readonly duplicateItems: (
    copies: readonly ItemPlacement[],
    mode: PlacementMode,
  ) => Promise<ItemId[]>;
  readonly pasteItems: (items: readonly PastedItem[], mode: PlacementMode) => Promise<ItemId[]>;
  readonly trimItems: (trims: readonly ItemTrim[], ripple: boolean) => Promise<void>;
  readonly splitItems: (itemIds: readonly ItemId[], at: Flicks) => Promise<ItemId[]>;
  readonly deleteItems: (itemIds: readonly ItemId[], ripple: boolean) => Promise<void>;
  readonly linkItems: (itemIds: readonly ItemId[]) => Promise<LinkGroupId>;
  readonly unlinkItems: (itemIds: readonly ItemId[]) => Promise<void>;
  readonly resizeGap: (gap: GapBounds, duration: Flicks) => Promise<void>;
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
    placeMedia: async (request) =>
      (await client.send({ kind: 'placeMedia', projectId, ...request })).itemIds,
    moveItems: async (moves, mode) => {
      await client.send({ kind: 'moveItems', projectId, moves: [...moves], mode });
    },
    duplicateItems: async (copies, mode) =>
      (await client.send({ kind: 'duplicateItems', projectId, copies: [...copies], mode })).itemIds,
    pasteItems: async (items, mode) =>
      (await client.send({ kind: 'pasteItems', projectId, items: [...items], mode })).itemIds,
    trimItems: async (trims, ripple) => {
      await client.send({ kind: 'trimItems', projectId, trims: [...trims], ripple });
    },
    splitItems: async (itemIds, at) =>
      (await client.send({ kind: 'splitItems', projectId, itemIds: [...itemIds], at })).itemIds,
    deleteItems: async (itemIds, ripple) => {
      await client.send({ kind: 'deleteItems', projectId, itemIds: [...itemIds], ripple });
    },
    linkItems: async (itemIds) =>
      (await client.send({ kind: 'linkItems', projectId, itemIds: [...itemIds] })).linkGroup,
    unlinkItems: async (itemIds) => {
      await client.send({ kind: 'unlinkItems', projectId, itemIds: [...itemIds] });
    },
    resizeGap: async ({ trackId, start, end }, duration) => {
      await client.send({ kind: 'resizeGap', projectId, trackId, start, end, duration });
    },
  };
}
