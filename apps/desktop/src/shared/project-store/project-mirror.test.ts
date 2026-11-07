import { describe, expect, it } from 'vitest';

import { applyPatch, mirrorFromSession, type ProjectMirror } from './project-mirror';
import {
  IDS,
  captionItem,
  emptyPatch,
  sequence,
  session,
  videoTrack,
} from './testing/session-fixture';

function applied(mirror: ProjectMirror, patch: Parameters<typeof applyPatch>[1]): ProjectMirror {
  const outcome = applyPatch(mirror, patch);
  if (outcome.kind !== 'applied') throw new Error(`expected applied, got ${outcome.kind}`);
  return outcome.mirror;
}

describe('mirrorFromSession', () => {
  it('indexes every entity by id and keeps the sequence order', () => {
    const mirror = mirrorFromSession(session());
    expect(mirror.projectId).toBe(IDS.project);
    expect(mirror.revision).toBe(3);
    expect(mirror.sequenceOrder).toEqual([IDS.sequence]);
    expect(mirror.tracks[IDS.v1]?.itemIds).toEqual([IDS.item]);
    expect(mirror.items[IDS.item]?.trackId).toBe(IDS.v1);
  });
});

describe('applyPatch', () => {
  it('replaces whole entities when an item moves to another track', () => {
    const mirror = mirrorFromSession(session());
    const patch = emptyPatch(4);
    patch.upserts.items = [captionItem(IDS.item, IDS.v2, 5000)];
    patch.upserts.tracks = [videoTrack(IDS.v1, []), videoTrack(IDS.v2, [IDS.item])];
    const next = applied(mirror, patch);
    expect(next.revision).toBe(4);
    expect(next.items[IDS.item]?.trackId).toBe(IDS.v2);
    expect(next.tracks[IDS.v1]?.itemIds).toEqual([]);
    expect(next.tracks[IDS.v2]?.itemIds).toEqual([IDS.item]);
    // The previous mirror is untouched.
    expect(mirror.items[IDS.item]?.trackId).toBe(IDS.v1);
  });

  it('removes deleted entities and appends new sequences', () => {
    const mirror = mirrorFromSession(session());
    const patch = emptyPatch(4);
    patch.deletes.items = [IDS.item];
    patch.upserts.tracks = [videoTrack(IDS.v1, [])];
    patch.upserts.sequences = [{ ...sequence([]), id: 'sequence-2', name: 'Short' }];
    patch.upserts.project = { ...session().snapshot.project, name: 'Renamed' };
    const next = applied(mirror, patch);
    expect(next.items[IDS.item]).toBeUndefined();
    expect(next.sequenceOrder).toEqual([IDS.sequence, 'sequence-2']);
    expect(next.project.name).toBe('Renamed');

    const removal = emptyPatch(5);
    removal.deletes.sequences = ['sequence-2'];
    expect(applied(next, removal).sequenceOrder).toEqual([IDS.sequence]);
  });

  it('ignores old and foreign patches and detects missing ones', () => {
    const mirror = mirrorFromSession(session(3));
    expect(applyPatch(mirror, emptyPatch(3)).kind).toBe('stale');
    expect(applyPatch(mirror, emptyPatch(5)).kind).toBe('gap');
    expect(applyPatch(mirror, { ...emptyPatch(4), projectId: 'other' }).kind).toBe('foreign');
  });
});
