import { useRef, useState, type KeyboardEvent } from 'react';
import { flushSync } from 'react-dom';

import { cn } from '../../lib/cn';
import { usePointerDrag, type PointerDragMove } from '../../lib/use-pointer-drag';
import { inputVariants } from '../../primitives/input';
import {
  clampToRange,
  decimalsOf,
  formatScrubValue,
  incrementFor,
  parseScrubValue,
  snapToIncrement,
  stepValue,
  type ScrubSpeeds,
} from './scrub-math';

/** Props of {@link ScrubInput}. */
export interface ScrubInputProps {
  /** Current value (controlled). */
  value: number;
  /** Live updates: every drag movement, keyboard step and typed value. */
  onChange: (value: number) => void;
  /**
   * The end of a gesture, fired **once**: on pointer release after a drag, after each keyboard
   * step and after a typed value is accepted. Commit here so one gesture is one undo step.
   */
  onCommit?: ((value: number) => void) | undefined;
  /** Accessible name, e.g. « Opacité » (pass `id` instead when a `<label htmlFor>` names it). */
  label?: string | undefined;
  /** Element id, for `<label htmlFor>` (e.g. inside `PropertyRow`). */
  id?: string | undefined;
  min?: number | undefined;
  max?: number | undefined;
  /** Increment of one step (keyboard and drag); default 1. */
  step?: number | undefined;
  /** Multiplier while Shift is held; default 10. */
  fastMultiplier?: number | undefined;
  /** Multiplier while Alt is held; default 0.1. */
  fineMultiplier?: number | undefined;
  /** Pointer travel, in CSS pixels, for one step; default 2. */
  pixelsPerStep?: number | undefined;
  /** Unit shown after the value (`px`, `%`, `°`). Not part of the typed text. */
  unit?: string | undefined;
  /** Formats the value for display and editing (default: number at step precision). */
  format?: ((value: number) => string) | undefined;
  /** Parses typed text; return `null` to reject it (default accepts `12,5` and `12.5 px`). */
  parse?: ((text: string) => number | null) | undefined;
  /** Control height: `xs` 24 px, `sm` 28 px (default), `md` 32 px. */
  size?: 'xs' | 'sm' | 'md' | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

/** Live state of one drag gesture (kept in a ref: it changes on every pointer move). */
interface DragState {
  readonly startValue: number;
  /** Unsnapped accumulated value, so sub-step movements add up. */
  raw: number;
  lastEmitted: number;
}

/** Keys that open the text editor when typed on the idle field (they start the new value). */
const EDIT_START_KEY = /^[\d.,+-]$/;

/**
 * A number field edited by dragging horizontally (`ew-resize` cursor): Shift = fast, Alt = fine,
 * arrow keys step, double-click / Enter / typing a digit opens keyboard entry (Enter or blur
 * accepts, Escape cancels). Emits `onChange` while dragging and `onCommit` once at the end.
 */
export function ScrubInput({
  value,
  onChange,
  onCommit,
  label,
  id,
  min,
  max,
  step = 1,
  fastMultiplier = 10,
  fineMultiplier = 0.1,
  pixelsPerStep = 2,
  unit,
  format,
  parse = parseScrubValue,
  size,
  disabled = false,
  className,
}: ScrubInputProps) {
  const [draft, setDraft] = useState<string | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const range = { min, max, step };
  const speeds: ScrubSpeeds = { fast: fastMultiplier, fine: fineMultiplier };
  const formatValue =
    format ?? ((current: number) => formatScrubValue(current, decimalsOf(step * fineMultiplier)));
  const valueText = unit === undefined ? formatValue(value) : `${formatValue(value)} ${unit}`;
  const editing = draft !== null;

  const emit = (next: number) => {
    onChange(next);
    onCommit?.(next);
  };

  const drag = usePointerDrag({
    onStart: (event) => {
      if (disabled || editing) {
        return false;
      }
      // Keep the text unselected while scrubbing; focus by hand since the default is prevented.
      event.preventDefault();
      if (event.currentTarget instanceof HTMLElement) {
        event.currentTarget.focus();
      }
      dragRef.current = { startValue: value, raw: value, lastEmitted: value };
      return undefined;
    },
    onMove: (move: PointerDragMove) => {
      const state = dragRef.current;
      if (state === null) {
        return;
      }
      const increment = incrementFor(step, move, speeds);
      state.raw = clampToRange(state.raw + (move.deltaX / pixelsPerStep) * increment, range);
      const next = clampToRange(snapToIncrement(state.raw, increment), range);
      if (next !== state.lastEmitted) {
        state.lastEmitted = next;
        onChange(next);
      }
    },
    onEnd: ({ moved }) => {
      const state = dragRef.current;
      dragRef.current = null;
      if (moved && state !== null && state.lastEmitted !== state.startValue) {
        onCommit?.(state.lastEmitted);
      }
    },
  });

  /** Opens keyboard entry: the whole value selected, or the caret after a typed first key. */
  const startEditing = (input: HTMLInputElement, text: string, selectAll: boolean) => {
    if (disabled) {
      return;
    }
    // Render the draft synchronously so the selection applies to the new text.
    flushSync(() => {
      setDraft(text);
    });
    if (selectAll) {
      input.select();
    } else {
      input.setSelectionRange(text.length, text.length);
    }
  };

  const acceptDraft = () => {
    if (draft === null) {
      return;
    }
    setDraft(null);
    const parsed = parse(draft);
    if (parsed !== null) {
      const next = clampToRange(parsed, range);
      if (next !== value) {
        emit(next);
      }
    }
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (editing) {
      if (event.key === 'Enter') {
        event.preventDefault();
        acceptDraft();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        setDraft(null);
      }
      return;
    }
    if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
      event.preventDefault();
      const next = stepValue(
        value,
        event.key === 'ArrowUp' ? 1 : -1,
        incrementFor(step, event, speeds),
        range,
        speeds,
      );
      if (next !== value) {
        emit(next);
      }
    } else if (event.key === 'Enter' || event.key === 'F2') {
      event.preventDefault();
      startEditing(event.currentTarget, formatValue(value), true);
    } else if (EDIT_START_KEY.test(event.key) && !event.ctrlKey && !event.metaKey) {
      event.preventDefault();
      startEditing(event.currentTarget, event.key, false);
    }
  };

  return (
    <input
      id={id}
      type="text"
      role="spinbutton"
      aria-label={label}
      aria-valuenow={value}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuetext={valueText}
      aria-readonly={false}
      inputMode="decimal"
      autoComplete="off"
      spellCheck={false}
      disabled={disabled}
      readOnly={!editing}
      value={editing ? draft : valueText}
      onChange={(event) => {
        setDraft(event.target.value);
      }}
      onKeyDown={handleKeyDown}
      onDoubleClick={(event) => {
        if (!editing) {
          startEditing(event.currentTarget, formatValue(value), true);
        }
      }}
      onBlur={acceptDraft}
      onPointerDown={drag.onPointerDown}
      className={cn(
        inputVariants({ size }),
        'tabular-nums',
        !editing && 'cursor-ew-resize caret-transparent select-none',
        className,
      )}
    />
  );
}
