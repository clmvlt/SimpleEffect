import { cva, type VariantProps } from 'class-variance-authority';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { Slot, Slottable } from '../../lib/slot';
import { Spinner } from '../spinner';

/**
 * Button styles, exported so other primitives (dialog actions, links rendered `asChild`) look the
 * same. Heights follow the control sizes: `xs` 24 px, `sm` 28 px (default), `md` 32 px.
 */
export const buttonVariants = cva(
  [
    'inline-flex shrink-0 items-center justify-center whitespace-nowrap font-medium select-none',
    'transition-colors duration-(--duration-fast) ease-standard',
    'disabled:pointer-events-none disabled:opacity-50',
    'aria-disabled:pointer-events-none aria-disabled:opacity-50',
    focusRing,
  ],
  {
    variants: {
      variant: {
        primary: 'bg-accent text-accent-foreground hover:bg-accent-hover',
        secondary: 'bg-control text-text hover:bg-control-hover',
        outline: 'border border-border-strong text-text hover:bg-hover',
        ghost: 'text-text-muted hover:bg-hover hover:text-text',
        danger: 'bg-danger-solid text-danger-foreground hover:bg-danger-solid-hover',
      },
      size: {
        xs: 'h-control-xs gap-1 rounded-sm px-2 text-xs',
        sm: 'h-control-sm gap-1.5 rounded-md px-2.5 text-sm',
        md: 'h-control-md gap-2 rounded-md px-3 text-sm',
      },
    },
    defaultVariants: { variant: 'secondary', size: 'sm' },
  },
);

/** Props of {@link Button}. */
export interface ButtonProps extends ComponentProps<'button'>, VariantProps<typeof buttonVariants> {
  /** Render the child element (e.g. a link) with the button styles instead of a `<button>`. */
  asChild?: boolean;
  /**
   * Shows a spinner before the label, disables the button and sets `aria-busy`, so a pending
   * action cannot be triggered twice.
   */
  loading?: boolean;
}

/**
 * The standard action button. Put an `Icon` before the label for icon + text buttons; use
 * `IconButton` for icon-only buttons. Defaults to `type="button"` (never submits a form by accident).
 */
export function Button({
  variant,
  size,
  asChild = false,
  loading = false,
  disabled,
  className,
  children,
  ...props
}: ButtonProps) {
  const Component = asChild ? Slot : 'button';
  return (
    <Component
      type={asChild ? undefined : 'button'}
      disabled={asChild ? undefined : disabled === true || loading}
      aria-busy={loading || undefined}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    >
      {loading && <Spinner size={size === 'xs' ? 'xs' : 'sm'} />}
      <Slottable>{children}</Slottable>
    </Component>
  );
}
