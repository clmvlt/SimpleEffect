import type {
  Asset,
  AssetId,
  HistoryState,
  ItemEntity,
  ItemId,
  MarkerEntity,
  MarkerId,
  ProjectHeader,
  ProjectId,
  ProjectPatch,
  ProjectRecovery,
  ProjectSession,
  SequenceEntity,
  SequenceId,
  TrackEntity,
  TrackId,
} from '@se/protocol';

/**
 * The renderer's read-only copy of an open project: the entities of the core's snapshot by id,
 * kept current by `projectPatched` events (`docs/architecture.md` §5.6). Never edited by
 * features: every change is a command to the core, which answers with a patch.
 */
export interface ProjectMirror {
  readonly projectId: ProjectId;
  /** Absolute path of the project folder. */
  readonly path: string;
  /** Revision of the last snapshot or patch applied. */
  readonly revision: number;
  readonly project: ProjectHeader;
  readonly sequences: Readonly<Record<SequenceId, SequenceEntity>>;
  /** Sequence ids in project order. */
  readonly sequenceOrder: readonly SequenceId[];
  readonly tracks: Readonly<Record<TrackId, TrackEntity>>;
  readonly items: Readonly<Record<ItemId, ItemEntity>>;
  readonly markers: Readonly<Record<MarkerId, MarkerEntity>>;
  readonly assets: Readonly<Record<AssetId, Asset>>;
  readonly history: HistoryState;
  /**
   * What opening repaired (journal replayed after a crash, backup restored), or `null` when the
   * project opened cleanly. Shown once by the editor (`useRecoveryNotice`).
   */
  readonly recovery: ProjectRecovery | null;
}

/** What {@link applyPatch} did with a patch. */
export type PatchOutcome =
  | { readonly kind: 'applied'; readonly mirror: ProjectMirror }
  /** Already included (its revision is not newer): ignore it. */
  | { readonly kind: 'stale' }
  /** A patch is missing in between: reload the snapshot. */
  | { readonly kind: 'gap' }
  /** It belongs to another project: ignore it. */
  | { readonly kind: 'foreign' };

/** Entities keyed by id. */
function byId<T extends { readonly id: string }>(entities: readonly T[]): Record<string, T> {
  return Object.fromEntries(entities.map((entity) => [entity.id, entity]));
}

/** The mirror of a session received from the core. */
export function mirrorFromSession(session: ProjectSession): ProjectMirror {
  const { snapshot } = session;
  return {
    projectId: snapshot.project.id,
    path: session.path,
    revision: snapshot.revision,
    project: snapshot.project,
    sequences: byId(snapshot.sequences),
    sequenceOrder: snapshot.sequences.map((sequence) => sequence.id),
    tracks: byId(snapshot.tracks),
    items: byId(snapshot.items),
    markers: byId(snapshot.markers),
    assets: byId(snapshot.assets),
    history: session.history,
    recovery: session.recovery,
  };
}

/** `records` without `deleted` and with `upserted` put in place (replacing by id). */
function replaceEntities<T extends { readonly id: string }>(
  records: Readonly<Record<string, T>>,
  upserted: readonly T[],
  deleted: readonly string[],
): Record<string, T> {
  const next = { ...records };
  for (const id of deleted) {
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete -- records are keyed by id
    delete next[id];
  }
  for (const entity of upserted) {
    next[entity.id] = entity;
  }
  return next;
}

/** Sequence order after a patch: removed ones dropped, new ones appended. */
function nextSequenceOrder(mirror: ProjectMirror, patch: ProjectPatch): SequenceId[] {
  const deleted = new Set(patch.deletes.sequences);
  const order = mirror.sequenceOrder.filter((id) => !deleted.has(id));
  for (const sequence of patch.upserts.sequences) {
    if (!order.includes(sequence.id)) {
      order.push(sequence.id);
    }
  }
  return order;
}

/**
 * Applies a patch when it is the next revision of `mirror`. Whole entities replace their
 * previous version: no operation logic is repeated here.
 */
export function applyPatch(mirror: ProjectMirror, patch: ProjectPatch): PatchOutcome {
  if (patch.projectId !== mirror.projectId) {
    return { kind: 'foreign' };
  }
  if (patch.revision <= mirror.revision) {
    return { kind: 'stale' };
  }
  if (patch.revision !== mirror.revision + 1) {
    return { kind: 'gap' };
  }
  const { upserts, deletes } = patch;
  return {
    kind: 'applied',
    mirror: {
      ...mirror,
      revision: patch.revision,
      project: upserts.project ?? mirror.project,
      sequences: replaceEntities(mirror.sequences, upserts.sequences, deletes.sequences),
      sequenceOrder: nextSequenceOrder(mirror, patch),
      tracks: replaceEntities(mirror.tracks, upserts.tracks, deletes.tracks),
      items: replaceEntities(mirror.items, upserts.items, deletes.items),
      markers: replaceEntities(mirror.markers, upserts.markers, deletes.markers),
      assets: replaceEntities(mirror.assets, upserts.assets, deletes.assets),
    },
  };
}
