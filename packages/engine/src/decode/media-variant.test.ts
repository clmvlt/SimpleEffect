import { describe, expect, it } from 'vitest';

import { imageAsset, videoAsset } from '../testing/document';
import { chooseVariant } from './media-variant';

describe('chooseVariant', () => {
  it('decodes the original when it is decodable', () => {
    expect(chooseVariant(videoAsset({ width: 64, height: 36 }), { proxy: false })).toEqual({
      kind: 'ready',
      variant: 'original',
    });
  });

  it('decodes the mezzanine of an undecodable original', () => {
    const prores = videoAsset({ width: 64, height: 36, decodable: false, mezzanine: true });
    expect(chooseVariant(prores, { proxy: false })).toEqual({
      kind: 'ready',
      variant: 'mezzanine',
    });
    const waiting = videoAsset({ width: 64, height: 36, decodable: false });
    expect(chooseVariant(waiting, { proxy: false })).toEqual({ kind: 'preparing' });
  });

  it('prefers a ready proxy when asked, even for an offline original', () => {
    const asset = { ...videoAsset({ width: 64, height: 36, proxy: true }), missing: true };
    expect(chooseVariant(asset, { proxy: true })).toEqual({ kind: 'ready', variant: 'proxy' });
    expect(chooseVariant(asset, { proxy: false })).toEqual({
      kind: 'unavailable',
      reason: 'missing',
    });
    const noProxy = videoAsset({ width: 64, height: 36 });
    expect(chooseVariant(noProxy, { proxy: true })).toEqual({ kind: 'ready', variant: 'original' });
  });

  it('always reads still images from their original', () => {
    expect(chooseVariant(imageAsset(64, 64), { proxy: true })).toEqual({
      kind: 'ready',
      variant: 'original',
    });
    expect(chooseVariant({ ...imageAsset(64, 64), missing: true }, { proxy: false })).toEqual({
      kind: 'unavailable',
      reason: 'missing',
    });
  });
});
