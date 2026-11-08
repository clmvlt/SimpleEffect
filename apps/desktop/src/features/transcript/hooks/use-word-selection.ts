import { useState, type PointerEvent } from 'react';

/** A range of selected words (indices, `first ≤ last`). */
export interface WordRange {
  readonly first: number;
  readonly last: number;
}

/** What the transcript text needs to select words with the pointer. */
export interface WordSelection {
  /** The words being or last selected. */
  readonly range: WordRange | null;
  readonly onPointerDown: (event: PointerEvent<HTMLElement>) => void;
  readonly onPointerMove: (event: PointerEvent<HTMLElement>) => void;
  readonly onPointerUp: (event: PointerEvent<HTMLElement>) => void;
  readonly clear: () => void;
}

/** The index of the word element under an event, or `null`. */
function wordIndexOf(event: PointerEvent<HTMLElement>): number | null {
  const target = event.target instanceof Element ? event.target.closest('[data-word-index]') : null;
  const value = target?.getAttribute('data-word-index');
  return value === null || value === undefined ? null : Number(value);
}

function rangeOf(anchor: number, focus: number): WordRange {
  return { first: Math.min(anchor, focus), last: Math.max(anchor, focus) };
}

/**
 * Word selection with the pointer: a click on a word calls `onClick(index)` (move the playhead
 * there); a drag across words, or a Maj+click after a click, selects a range and calls
 * `onRange(range)` when the button is released (mark it as in/out).
 */
export function useWordSelection(
  onClick: (index: number) => void,
  onRange: (range: WordRange) => void,
): WordSelection {
  const [anchor, setAnchor] = useState<number | null>(null);
  const [range, setRange] = useState<WordRange | null>(null);
  const [dragging, setDragging] = useState(false);
  return {
    range,
    onPointerDown: (event) => {
      const index = wordIndexOf(event);
      if (index === null || event.button !== 0) return;
      event.preventDefault();
      if (event.shiftKey && anchor !== null) {
        setRange(rangeOf(anchor, index));
      } else {
        setAnchor(index);
        setRange({ first: index, last: index });
      }
      setDragging(true);
    },
    onPointerMove: (event) => {
      if (!dragging || anchor === null) return;
      const index = wordIndexOf(event);
      if (index !== null) setRange(rangeOf(anchor, index));
    },
    onPointerUp: () => {
      if (!dragging || range === null) return;
      setDragging(false);
      if (range.first === range.last) {
        onClick(range.first);
      } else {
        onRange(range);
      }
    },
    clear: () => {
      setRange(null);
      setAnchor(null);
    },
  };
}
