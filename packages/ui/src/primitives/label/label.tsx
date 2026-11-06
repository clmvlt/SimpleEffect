import { Label as LabelPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/** Props of {@link Label}. */
export type LabelProps = ComponentProps<typeof LabelPrimitive.Root>;

/**
 * Text label of a form control. Link it with `htmlFor` = the control's `id` (or use `Field`, which
 * does it for you). Clicking the label focuses or toggles the control. Dims itself when the control
 * that precedes it (`peer`) or its enclosing group (`data-disabled`) is disabled.
 */
export function Label({ className, ...props }: LabelProps) {
  return (
    <LabelPrimitive.Root
      className={cn(
        'text-xs font-medium text-text-muted select-none',
        'peer-disabled:cursor-not-allowed peer-disabled:opacity-50',
        'group-data-[disabled=true]:cursor-not-allowed group-data-[disabled=true]:opacity-50',
        className,
      )}
      {...props}
    />
  );
}
