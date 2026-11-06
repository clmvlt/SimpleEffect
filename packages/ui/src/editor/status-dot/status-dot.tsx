import { cva, type VariantProps } from 'class-variance-authority';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/** Dot colours: semantic states, the accent and the AI colour; `neutral` for idle or unknown. */
export const statusDotVariants = cva('inline-block shrink-0 rounded-full', {
  variants: {
    tone: {
      neutral: 'bg-text-subtle',
      success: 'bg-success',
      warning: 'bg-warning',
      danger: 'bg-danger',
      info: 'bg-info',
      accent: 'bg-accent',
      ai: 'bg-ai',
    },
    size: { xs: 'size-1.5', sm: 'size-2' },
  },
  defaultVariants: { tone: 'neutral', size: 'sm' },
});

/** Props of {@link StatusDot}. */
export interface StatusDotProps
  extends Omit<ComponentProps<'span'>, 'children'>, VariantProps<typeof statusDotVariants> {
  /**
   * What the dot means, read by screen readers (« Cœur connecté »). Omit it when visible text next
   * to the dot already says it: the dot is then decorative.
   */
  label?: string | undefined;
  /** Breathing animation for ongoing states (connecting, running). */
  pulse?: boolean | undefined;
}

/**
 * A small coloured dot for statuses (core connection, AI availability, job state). Colour is never
 * the only signal: pair it with text or give it a `label`.
 */
export function StatusDot({
  tone,
  size,
  label,
  pulse = false,
  className,
  ...props
}: StatusDotProps) {
  return (
    <span
      role={label === undefined ? undefined : 'img'}
      aria-label={label}
      aria-hidden={label === undefined ? true : undefined}
      data-tone={tone ?? 'neutral'}
      className={cn(statusDotVariants({ tone, size }), pulse && 'animate-pulse', className)}
      {...props}
    />
  );
}
