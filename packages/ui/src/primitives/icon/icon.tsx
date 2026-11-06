import type { LucideIcon, LucideProps } from 'lucide-react';

import { cn } from '../../lib/cn';

/** Normalised icon sizes, in pixels: dense controls use `sm`, most UI uses `md`. */
export const ICON_SIZES = { xs: 12, sm: 14, md: 16, lg: 20 } as const;

/** One of the normalised icon sizes. */
export type IconSize = keyof typeof ICON_SIZES;

/** Props of {@link Icon}. */
export interface IconProps extends Omit<LucideProps, 'size' | 'ref'> {
  /** The icon to draw, imported from `@se/ui/icons` (e.g. `Scissors`). */
  icon: LucideIcon;
  /** Normalised size (default `md`, 16 px). */
  size?: IconSize;
  /**
   * Accessible name. Omit it for decorative icons (next to a visible label, or inside a control
   * that has its own label): the icon is then hidden from assistive technologies.
   */
  label?: string;
}

/**
 * Draws a lucide icon at a normalised size, in the current text colour.
 * Decorative by default; pass `label` when the icon alone carries meaning.
 */
export function Icon({
  icon: IconGlyph,
  size = 'md',
  label,
  strokeWidth = 1.75,
  className,
  ...props
}: IconProps) {
  const accessibility =
    label === undefined
      ? ({ 'aria-hidden': true } as const)
      : ({ role: 'img', 'aria-label': label } as const);
  return (
    <IconGlyph
      size={ICON_SIZES[size]}
      strokeWidth={strokeWidth}
      focusable="false"
      className={cn('shrink-0', className)}
      {...accessibility}
      {...props}
    />
  );
}
