import { cva, type VariantProps } from 'class-variance-authority';
import type { LucideIcon } from 'lucide-react';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { buttonVariants } from '../button';
import { Icon, type IconSize } from '../icon';
import { Spinner } from '../spinner';

/** Square sizes of icon-only buttons, matching the control heights (24/28/32 px). */
export const iconButtonVariants = cva('px-0', {
  variants: {
    size: {
      xs: 'w-control-xs',
      sm: 'w-control-sm',
      md: 'w-control-md',
    },
  },
  defaultVariants: { size: 'sm' },
});

/** Glyph size per button size: 14 px in 24 px buttons, 16 px otherwise. */
const GLYPH_SIZES = { xs: 'sm', sm: 'md', md: 'md' } as const satisfies Record<string, IconSize>;

/** Props of {@link IconButton}. */
export interface IconButtonProps
  extends Omit<ComponentProps<'button'>, 'children'>, VariantProps<typeof buttonVariants> {
  /** The icon, imported from `@se/ui/icons`. */
  icon: LucideIcon;
  /** Accessible name (the button has no visible text), e.g. « Couper le clip ». */
  label: string;
  /** Replaces the icon with a spinner, disables the button and sets `aria-busy`. */
  loading?: boolean;
}

/**
 * Icon-only button for toolbars and dense panels. `label` is required because the icon alone gives
 * no accessible name; pair it with a `Tooltip` to show the same text on hover. Defaults to the
 * `ghost` variant, the usual look of toolbar buttons.
 */
export function IconButton({
  icon,
  label,
  variant = 'ghost',
  size,
  loading = false,
  disabled,
  className,
  ...props
}: IconButtonProps) {
  const glyphSize = GLYPH_SIZES[size ?? 'sm'];
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled === true || loading}
      aria-busy={loading || undefined}
      className={cn(buttonVariants({ variant, size }), iconButtonVariants({ size }), className)}
      {...props}
    >
      {loading ? <Spinner size={glyphSize} /> : <Icon icon={icon} size={glyphSize} />}
    </button>
  );
}
