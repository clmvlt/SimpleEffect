import { ChevronLeft, ChevronRight } from 'lucide-react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { IconButton } from '../../primitives/icon-button';

/**
 * Keyframe status of a property at the playhead: not animated (`none`), animated with a keyframe
 * exactly here (`onKeyframe`), animated but between two keyframes (`betweenKeyframes`).
 */
export type KeyframeState = 'none' | 'onKeyframe' | 'betweenKeyframes';

/** Props of {@link KeyframeButton}. */
export interface KeyframeButtonProps {
  state: KeyframeState;
  /** Adds a keyframe at the playhead, or removes the one there (`onKeyframe`). */
  onToggle: () => void;
  /** Moves the playhead to the previous keyframe; the button is disabled when omitted. */
  onPrevious?: (() => void) | undefined;
  /** Moves the playhead to the next keyframe; the button is disabled when omitted. */
  onNext?: (() => void) | undefined;
  /** Show the previous/next buttons (default: when the property is animated). */
  showNavigation?: boolean | undefined;
  /** Accessible name of the diamond (default English « Toggle keyframe »). */
  label?: string | undefined;
  /** Accessible name of the previous button (default English « Previous keyframe »). */
  previousLabel?: string | undefined;
  /** Accessible name of the next button (default English « Next keyframe »). */
  nextLabel?: string | undefined;
  disabled?: boolean | undefined;
  className?: string | undefined;
}

const diamondClasses: Record<KeyframeState, string> = {
  none: 'border-text-subtle bg-transparent group-hover:border-text',
  betweenKeyframes: 'border-accent bg-accent-muted',
  onKeyframe: 'border-accent bg-accent',
};

/**
 * The diamond next to an animatable property: hollow when not animated, tinted between keyframes,
 * filled on a keyframe. It is a toggle button (`aria-pressed` while on a keyframe), optionally
 * flanked by previous/next keyframe buttons.
 */
export function KeyframeButton({
  state,
  onToggle,
  onPrevious,
  onNext,
  showNavigation = state !== 'none',
  label = 'Toggle keyframe',
  previousLabel = 'Previous keyframe',
  nextLabel = 'Next keyframe',
  disabled = false,
  className,
}: KeyframeButtonProps) {
  const toggle = (
    <button
      type="button"
      aria-label={label}
      aria-pressed={state === 'onKeyframe'}
      data-state={state}
      disabled={disabled}
      onClick={onToggle}
      className={cn(
        'group inline-flex size-control-xs shrink-0 items-center justify-center rounded-sm',
        'hover:bg-hover disabled:pointer-events-none disabled:opacity-50',
        focusRing,
      )}
    >
      <span
        aria-hidden
        className={cn(
          'size-2.5 rotate-45 rounded-[1px] border transition-colors duration-(--duration-fast)',
          diamondClasses[state],
        )}
      />
    </button>
  );
  if (!showNavigation) {
    return <div className={cn('inline-flex items-center', className)}>{toggle}</div>;
  }
  return (
    <div className={cn('inline-flex items-center', className)}>
      <IconButton
        icon={ChevronLeft}
        label={previousLabel}
        size="xs"
        className="w-4"
        disabled={disabled || onPrevious === undefined}
        onClick={onPrevious}
      />
      {toggle}
      <IconButton
        icon={ChevronRight}
        label={nextLabel}
        size="xs"
        className="w-4"
        disabled={disabled || onNext === undefined}
        onClick={onNext}
      />
    </div>
  );
}
