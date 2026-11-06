import { cva, type VariantProps } from 'class-variance-authority';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/**
 * Surface, border, focus and invalid styles shared by every text-like field (Input, Textarea,
 * Select trigger, and later editor fields such as scrub or timecode inputs). Focus is shown for
 * mouse and keyboard alike, as browsers do for text fields: the border and a 1 px outline turn
 * `focus-ring`, reading as a 2 px ring. `aria-invalid` turns them `danger`.
 */
export const textFieldClasses = [
  'w-full min-w-0 rounded-md border border-border bg-input text-text placeholder:text-text-subtle',
  'transition-colors duration-(--duration-fast) ease-standard hover:border-border-strong',
  'outline-none focus-visible:border-focus-ring focus-visible:outline-1 focus-visible:outline-offset-0 focus-visible:outline-focus-ring',
  'aria-invalid:border-danger aria-invalid:focus-visible:border-danger aria-invalid:focus-visible:outline-danger',
  'disabled:cursor-not-allowed disabled:opacity-50',
];

/** Input styles; heights follow the control sizes: `xs` 24 px, `sm` 28 px (default), `md` 32 px. */
export const inputVariants = cva([...textFieldClasses, 'flex px-2'], {
  variants: {
    size: {
      xs: 'h-control-xs text-xs',
      sm: 'h-control-sm text-sm',
      md: 'h-control-md text-sm',
    },
  },
  defaultVariants: { size: 'sm' },
});

/** Props of {@link Input}. The native `size` attribute is replaced by the control size. */
export interface InputProps
  extends Omit<ComponentProps<'input'>, 'size'>, VariantProps<typeof inputVariants> {}

/**
 * Single-line text field. Name it with a `Label` (or wrap it in `Field`); mark errors with
 * `aria-invalid` (`Field` does it when it has an `error`).
 */
export function Input({ size, type = 'text', className, ...props }: InputProps) {
  return <input type={type} className={cn(inputVariants({ size }), className)} {...props} />;
}
