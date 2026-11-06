import { cva, type VariantProps } from 'class-variance-authority';
import { Progress as ProgressPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/** Track styles: a thin rounded bar, 2 to 6 px high. */
export const progressVariants = cva('relative w-full overflow-hidden rounded-full bg-control', {
  variants: {
    size: { xs: 'h-0.5', sm: 'h-1', md: 'h-1.5' },
  },
  defaultVariants: { size: 'sm' },
});

/** Indicator colours: accent for ordinary jobs, semantic colours for outcomes, `ai` for AI tasks. */
export const progressIndicatorVariants = cva(
  'h-full w-full rounded-full transition-transform duration-(--duration-normal) ease-standard',
  {
    variants: {
      tone: {
        accent: 'bg-accent',
        success: 'bg-success',
        warning: 'bg-warning',
        danger: 'bg-danger',
        ai: 'bg-ai',
      },
    },
    defaultVariants: { tone: 'accent' },
  },
);

/** Props of {@link Progress}. */
export interface ProgressProps
  extends
    Omit<ComponentProps<typeof ProgressPrimitive.Root>, 'value'>,
    VariantProps<typeof progressVariants>,
    VariantProps<typeof progressIndicatorVariants> {
  /**
   * Completion between 0 and `max` (default 100), or `null` when the amount of work is unknown
   * (indeterminate: a segment sweeps across the track and no value is announced).
   */
  value: number | null;
}

/**
 * A progress bar for jobs (imports, analyses, exports, AI tasks). Give it an accessible name with
 * `aria-label` or `aria-labelledby`; screen readers announce the percentage.
 */
export function Progress({ value, max = 100, size, tone, className, ...props }: ProgressProps) {
  const indeterminate = value === null;
  const percent = indeterminate ? 0 : Math.min(100, Math.max(0, (value / max) * 100));
  return (
    <ProgressPrimitive.Root
      value={value}
      max={max}
      className={cn(progressVariants({ size }), className)}
      {...props}
    >
      <ProgressPrimitive.Indicator
        className={cn(
          progressIndicatorVariants({ tone }),
          indeterminate && 'w-2/5 animate-progress-indeterminate',
        )}
        style={indeterminate ? undefined : { transform: `translateX(-${String(100 - percent)}%)` }}
      />
    </ProgressPrimitive.Root>
  );
}
