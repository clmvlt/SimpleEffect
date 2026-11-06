import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/** Props of {@link Skeleton}. */
export type SkeletonProps = ComponentProps<'div'>;

/**
 * A pulsing placeholder shown where content is still loading (thumbnails, list rows, metadata).
 * Size and shape it with classes (`h-4 w-32`, `aspect-video`, `rounded-full`). It is hidden from
 * assistive technologies: announce loading on the container instead (`aria-busy`, or a `Spinner`
 * with a label).
 */
export function Skeleton({ className, ...props }: SkeletonProps) {
  return (
    <div
      aria-hidden="true"
      className={cn('animate-pulse rounded-md bg-control', className)}
      {...props}
    />
  );
}
