import { cva, type VariantProps } from 'class-variance-authority';
import type { ComponentProps, ReactNode } from 'react';

import { cn } from '../../lib/cn';

/** Clip colours by kind of timeline item, plus the selected and dimmed states. */
export const clipShellVariants = cva(
  [
    'group/clip relative flex h-full min-w-0 flex-col overflow-hidden rounded-xs border',
    'text-2xs text-text select-none transition-colors duration-(--duration-fast)',
  ],
  {
    variants: {
      tone: {
        video: 'border-item-video-strong/50 bg-item-video-muted hover:border-item-video-strong',
        audio: 'border-item-audio-strong/50 bg-item-audio-muted hover:border-item-audio-strong',
        graphic:
          'border-item-graphic-strong/50 bg-item-graphic-muted hover:border-item-graphic-strong',
        caption:
          'border-item-caption-strong/50 bg-item-caption-muted hover:border-item-caption-strong',
      },
      selected: {
        true: 'border-accent ring-1 ring-accent ring-inset hover:border-accent',
        false: '',
      },
      dimmed: { true: 'opacity-50', false: '' },
    },
    defaultVariants: { tone: 'video', selected: false, dimmed: false },
  },
);

/** Props of {@link ClipShell}. */
export interface ClipShellProps
  extends Omit<ComponentProps<'div'>, 'children'>, VariantProps<typeof clipShellVariants> {
  /** Name shown in the clip's top bar (file name, text…), truncated when the clip is short. */
  label: string;
  /** Small marks after the label, e.g. a {@link ClipBadge} for AI-made items. */
  badges?: ReactNode | undefined;
  /** The clip's picture layer (thumbnails, waveform), drawn behind the label and filling it. */
  children?: ReactNode | undefined;
}

/**
 * The body of a timeline item: kind colour, label bar, optional badges, selected (accent ring)
 * and dimmed (hidden or muted track) states. Purely visual: the timeline positions it, gives it
 * its role and handles pointer events.
 */
export function ClipShell({
  tone,
  selected,
  dimmed,
  label,
  badges,
  children,
  className,
  ...props
}: ClipShellProps) {
  return (
    <div
      data-tone={tone ?? 'video'}
      data-selected={selected === true ? '' : undefined}
      className={cn(clipShellVariants({ tone, selected, dimmed }), className)}
      {...props}
    >
      {children !== undefined && <div className="absolute inset-0">{children}</div>}
      <div className="relative flex h-4 shrink-0 items-center gap-1 px-1">
        <span className="min-w-0 truncate rounded-xs bg-panel/60 px-0.5 leading-4">{label}</span>
        {badges}
      </div>
    </div>
  );
}

/** Props of {@link ClipBadge}. */
export interface ClipBadgeProps extends Omit<ComponentProps<'span'>, 'children'> {
  /** The badge text (« IA »), short. */
  label: string;
  /** What the badge means, read by screen readers and shown on hover (« Créé par l’IA »). */
  description?: string | undefined;
}

/** A tiny AI-coloured badge for a clip's top bar (items made by an AI task). */
export function ClipBadge({ label, description, className, ...props }: ClipBadgeProps) {
  return (
    <span
      title={description}
      className={cn(
        'shrink-0 rounded-xs border border-ai bg-ai-muted px-0.5 text-2xs leading-3 font-medium text-text',
        className,
      )}
      {...props}
    >
      {label}
      {description !== undefined && <span className="sr-only">{` — ${description}`}</span>}
    </span>
  );
}
