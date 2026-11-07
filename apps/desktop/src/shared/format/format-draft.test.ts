import { describe, expect, it } from 'vitest';

import { draftFromFormat, draftToFormat, isDraftSizeValid, withDraftRatio } from './format-draft';

const hd = {
  width: 1920,
  height: 1080,
  frameRate: { numerator: 30_000, denominator: 1001 },
  sampleRate: 48_000,
};

describe('format drafts', () => {
  it('round-trip a format and take the 1080p-class size of a new ratio', () => {
    const draft = draftFromFormat(hd);
    expect(draft.ratio).toEqual({ width: 16, height: 9 });
    expect(draftToFormat(draft)).toEqual(hd);
    const vertical = withDraftRatio(draft, { width: 9, height: 16 });
    expect(draftToFormat(vertical)).toEqual({ ...hd, width: 1080, height: 1920 });
    expect(isDraftSizeValid({ ...vertical, width: 1081 })).toBe(false);
  });
});
