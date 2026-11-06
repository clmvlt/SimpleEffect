import { cva, type VariantProps } from 'class-variance-authority';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/**
 * Badge styles. Semantic and item-kind variants use a translucent `-muted` fill with a readable
 * text colour (the solid semantic colour, or the item kind's `-strong` shade), all AA on panels.
 */
export const badgeVariants = cva(
  'inline-flex shrink-0 items-center gap-1 font-medium whitespace-nowrap select-none',
  {
    variants: {
      variant: {
        neutral: 'bg-control text-text-muted',
        accent: 'bg-accent-muted text-text',
        success: 'bg-success-muted text-success',
        warning: 'bg-warning-muted text-warning',
        danger: 'bg-danger-muted text-danger',
        info: 'bg-info-muted text-info',
        video: 'bg-item-video-muted text-item-video-strong',
        audio: 'bg-item-audio-muted text-item-audio-strong',
        graphic: 'bg-item-graphic-muted text-item-graphic-strong',
        caption: 'bg-item-caption-muted text-item-caption-strong',
        ai: 'bg-ai-muted text-ai-strong',
      },
      size: {
        xs: 'h-4 px-1 text-2xs',
        sm: 'h-5 px-1.5 text-xs',
      },
      shape: {
        rounded: 'rounded-sm',
        pill: 'rounded-full',
      },
    },
    compoundVariants: [
      { shape: 'pill', size: 'xs', className: 'px-1.5' },
      { shape: 'pill', size: 'sm', className: 'px-2' },
    ],
    defaultVariants: { variant: 'neutral', size: 'sm', shape: 'rounded' },
  },
);

/** Props of {@link Badge}. */
export interface BadgeProps extends ComponentProps<'span'>, VariantProps<typeof badgeVariants> {}

/**
 * A short, non-interactive label: a status (« Analysé », « Erreur »), a count, or the kind of a
 * timeline item (video, audio, graphic, caption, AI). Put a decorative `Icon` before the text if
 * needed. For anything clickable, use a `Button` instead.
 */
export function Badge({ variant, size, shape, className, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant, size, shape }), className)} {...props} />;
}
