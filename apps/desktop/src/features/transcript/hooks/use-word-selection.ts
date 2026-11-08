import { useRef, useState, type PointerEvent } from 'react';

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

/** The gesture in progress, kept outside React state so a press and its release read it at once. */
interface Gesture {
  anchor: number | null;
  range: WordRange | null;
  dragging: boolean;
}

/**
 * Word selection with the pointer: a click on a word calls `onClick(index)` (move the playhead
 * there); a drag across words, or a Maj+click after a click, selects a range and calls
 * `onRange(range)` when the button is released (mark it as in/out). The gesture lives in a ref
 * (a release may come before React re-renders after the press); the range is mirrored in state
 * for display.
 */
export function useWordSelection(
  onClick: (index: number) => void,
  onRange: (range: WordRange) => void,
): WordSelection {
  const gesture = useRef<Gesture>({ anchor: null, range: null, dragging: false });
  const [range, setRange] = useState<WordRange | null>(null);
  const show = (next: WordRange | null) => {
    gesture.current.range = next;
    setRange(next);
  };
  return {
    range,
    onPointerDown: (event) => {
      const index = wordIndexOf(event);
      if (index === null || event.button !== 0) return;
      event.preventDefault();
      const { anchor } = gesture.current;
      if (event.shiftKey && anchor !== null) {
        show(rangeOf(anchor, index));
      } else {
        gesture.current.anchor = index;
        show({ first: index, last: index });
      }
      gesture.current.dragging = true;
    },
    onPointerMove: (event) => {
      const { anchor, dragging } = gesture.current;
      if (!dragging || anchor === null) return;
      const index = wordIndexOf(event);
      const current = gesture.current.range;
      if (index !== null && (current === null || !sameRange(current, rangeOf(anchor, index)))) {
        show(rangeOf(anchor, index));
      }
    },
    onPointerUp: () => {
      const { dragging, range: selected } = gesture.current;
      if (!dragging || selected === null) return;
      gesture.current.dragging = false;
      if (selected.first === selected.last) {
        onClick(selected.first);
      } else {
        onRange(selected);
      }
    },
    clear: () => {
      gesture.current = { anchor: null, range: null, dragging: false };
      setRange(null);
    },
  };
}

function sameRange(left: WordRange, right: WordRange): boolean {
  return left.first === right.first && left.last === right.last;
}
