import { describe, expect, it } from 'vitest';

import { recoveryNotice } from './recovery-notice';

const clean = { replayedChanges: 0, restoredFrom: null, problem: null, lostChanges: 0 };

describe('recoveryNotice', () => {
  it('says nothing for a clean opening', () => {
    expect(recoveryNotice(null)).toBeNull();
    expect(recoveryNotice(clean)).toBeNull();
  });

  it('counts the changes replayed after an unexpected stop', () => {
    expect(recoveryNotice({ ...clean, replayedChanges: 7 })).toEqual({
      kind: 'replayed',
      count: 7,
    });
  });

  it('puts a restored backup first, with what was replayed and lost', () => {
    expect(
      recoveryNotice({
        replayedChanges: 2,
        restoredFrom: 'project.json.bak.1',
        problem: 'expected value at line 1',
        lostChanges: 5,
      }),
    ).toEqual({ kind: 'restored', backup: 'project.json.bak.1', replayed: 2, lost: 5 });
  });
});
