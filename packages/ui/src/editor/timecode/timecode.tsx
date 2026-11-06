import { cva, type VariantProps } from 'class-variance-authority';
import { useState, type ComponentProps, type KeyboardEvent } from 'react';

import { cn } from '../../lib/cn';
import { textFieldClasses } from '../../primitives/input';

/**
 * Timecode widgets know nothing about time units: the caller owns the value (flicks, frames…) and
 * passes `format` and `parse`, so the domain rules (frame rate, drop frame) stay in the domain.
 */
interface TimecodeCodec {
  /** Text of a value, e.g. `00:01:12:05`. */
  format: (value: number) => string;
  /** Value of a typed text, or `null` when it is not a valid timecode. */
  parse: (text: string) => number | null;
}

/** Field sizes; widths fit `00:00:00:00` in the mono face. */
export const timecodeInputVariants = cva(
  [...textFieldClasses, 'timecode w-auto px-2 text-center'],
  {
    variants: {
      size: {
        xs: 'h-control-xs w-[11ch] text-xs',
        sm: 'h-control-sm w-[12ch] text-sm',
        md: 'h-control-md w-[12ch] text-sm',
      },
    },
    defaultVariants: { size: 'sm' },
  },
);

/** Props of {@link TimecodeInput}. */
export interface TimecodeInputProps
  extends TimecodeCodec, VariantProps<typeof timecodeInputVariants> {
  value: number;
  /** Called with the parsed value when a valid timecode is entered (Enter or blur). */
  onCommit: (value: number) => void;
  /** Accessible name, e.g. « Position de la tête de lecture ». */
  label?: string | undefined;
  id?: string | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

/**
 * A timecode field: shows `format(value)`, selects everything on focus, and on Enter or blur
 * parses the text. Invalid text is flagged (`aria-invalid`) and kept so it can be fixed; Escape
 * restores the current value.
 */
export function TimecodeInput({
  value,
  format,
  parse,
  onCommit,
  label,
  id,
  size,
  disabled = false,
  className,
}: TimecodeInputProps) {
  const [draft, setDraft] = useState<string | null>(null);
  const [invalid, setInvalid] = useState(false);

  const reset = () => {
    setDraft(null);
    setInvalid(false);
  };

  /** Commits a valid draft; returns false when the draft cannot be parsed. */
  const acceptDraft = (): boolean => {
    if (draft === null) {
      return true;
    }
    const parsed = parse(draft);
    if (parsed === null) {
      setInvalid(true);
      return false;
    }
    reset();
    if (parsed !== value) {
      onCommit(parsed);
    }
    return true;
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      if (acceptDraft()) {
        event.currentTarget.select();
      }
    } else if (event.key === 'Escape') {
      event.preventDefault();
      reset();
    }
  };

  return (
    <input
      id={id}
      type="text"
      aria-label={label}
      aria-invalid={invalid || undefined}
      autoComplete="off"
      spellCheck={false}
      disabled={disabled}
      value={draft ?? format(value)}
      onChange={(event) => {
        setDraft(event.target.value);
        setInvalid(false);
      }}
      onFocus={(event) => {
        event.currentTarget.select();
      }}
      onBlur={() => {
        if (!acceptDraft()) {
          reset();
        }
      }}
      onKeyDown={handleKeyDown}
      className={cn(timecodeInputVariants({ size }), className)}
    />
  );
}

/** Props of {@link TimecodeDisplay}. */
export interface TimecodeDisplayProps
  extends Omit<ComponentProps<'span'>, 'children'>, Pick<TimecodeCodec, 'format'> {
  value: number;
  /** Accessible name; when given, the display is exposed as a `timer` (never announced live). */
  label?: string | undefined;
}

/** A read-only timecode (playhead position, duration) in the mono face with tabular digits. */
export function TimecodeDisplay({
  value,
  format,
  label,
  className,
  ...props
}: TimecodeDisplayProps) {
  return (
    <span
      role={label === undefined ? undefined : 'timer'}
      aria-label={label}
      className={cn('timecode text-sm text-text select-none', className)}
      {...props}
    >
      {format(value)}
    </span>
  );
}
