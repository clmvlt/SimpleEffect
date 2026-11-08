import { describe, expect, it } from 'vitest';

import { readCaptionsPreferences } from '../store/captions-store';
import { captionRowsOf, captionSession, seconds } from '../testing/caption-fixture';
import { captionKeyAction, parseShiftSeconds, splitWordIndex } from './caption-editing';
import { activeRowIndex, styleTargets } from './caption-rows';

describe('caption rows', () => {
  it('list every caption of the sequence in time order with its text', () => {
    const rows = captionRowsOf(captionSession());
    expect(rows.map((row) => row.text)).toEqual(['Bonjour à tous', 'on commence', 'un titre']);
    expect(activeRowIndex(rows, seconds(1.5))).toBe(1);
    expect(activeRowIndex(rows, seconds(9))).toBe(-1);
  });

  it('target the selection, the caption under the playhead, or the whole track', () => {
    const rows = captionRowsOf(captionSession());
    expect(styleTargets(rows, 'selection', ['caption-2'], 0)).toEqual(['caption-2']);
    expect(styleTargets(rows, 'selection', [], 0)).toEqual(['caption-1']);
    expect(styleTargets(rows, 'track', ['caption-1'], -1)).toEqual(['caption-1', 'caption-2']);
    expect(styleTargets(rows, 'track', ['caption-3'], -1)).toEqual(['caption-3']);
  });
});

describe('caption editing', () => {
  it('splits before the word at or after the caret', () => {
    expect(splitWordIndex('Bonjour à tous', 7)).toBe(1);
    expect(splitWordIndex('Bonjour à tous', 8)).toBe(1);
    expect(splitWordIndex('Bonjour à tous', 11)).toBe(2);
    expect(splitWordIndex('Bonjour à tous', 0)).toBeNull();
    expect(splitWordIndex('Bonjour à tous', 14)).toBeNull();
  });

  it('reads keys: Entrée splits, Ctrl+Entrée commits, Retour arrière at the start merges', () => {
    const key = (name: string, ctrlKey = false) => ({
      key: name,
      ctrlKey,
      metaKey: false,
      shiftKey: false,
    });
    expect(captionKeyAction(key('Enter'), { start: 3, end: 3 })).toBe('split');
    expect(captionKeyAction(key('Enter', true), { start: 3, end: 3 })).toBe('commit');
    expect(captionKeyAction(key('Backspace'), { start: 0, end: 0 })).toBe('mergeWithPrevious');
    expect(captionKeyAction(key('Backspace'), { start: 0, end: 4 })).toBeNull();
    expect(captionKeyAction(key('Escape'), { start: 2, end: 2 })).toBe('cancel');
  });

  it('reads shifts in French or English decimals', () => {
    expect(parseShiftSeconds('-0,5')).toBe(-0.5);
    expect(parseShiftSeconds(' 1.25 s')).toBe(1.25);
    expect(parseShiftSeconds('0')).toBeNull();
    expect(parseShiftSeconds('abc')).toBeNull();
  });

  it('restores valid preferences only', () => {
    expect(readCaptionsPreferences('{"scope":"selection","preset":"box"}')).toEqual({
      scope: 'selection',
      preset: 'box',
    });
    expect(readCaptionsPreferences('{"scope":"x","preset":"disco"}')).toEqual({
      scope: 'track',
      preset: 'karaoke',
    });
    expect(readCaptionsPreferences('not json')).toEqual({ scope: 'track', preset: 'karaoke' });
  });
});
