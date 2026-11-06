import { RadioGroup as RadioGroupPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';

/** Props of {@link RadioGroup}: Radix radio group props (`value`, `onValueChange`, `orientation`…). */
export type RadioGroupProps = ComponentProps<typeof RadioGroupPrimitive.Root>;

/**
 * A set of mutually exclusive options. Give the group a name (`aria-label` or `aria-labelledby`) and
 * each `RadioGroupItem` a `Label`. One Tab stop: arrow keys move and select within the group.
 */
export function RadioGroup({ className, ...props }: RadioGroupProps) {
  return <RadioGroupPrimitive.Root className={cn('grid gap-2', className)} {...props} />;
}

/** Props of {@link RadioGroupItem}. */
export type RadioGroupItemProps = ComponentProps<typeof RadioGroupPrimitive.Item>;

/** One 16 px option of a {@link RadioGroup}; `value` identifies it. */
export function RadioGroupItem({ className, ...props }: RadioGroupItemProps) {
  return (
    <RadioGroupPrimitive.Item
      className={cn(
        'peer inline-flex size-4 shrink-0 items-center justify-center rounded-full',
        'border border-border-strong bg-input transition-colors duration-(--duration-fast) ease-standard',
        'hover:border-text-subtle data-[state=checked]:border-accent data-[state=checked]:bg-accent',
        'aria-invalid:border-danger disabled:cursor-not-allowed disabled:opacity-50',
        focusRing,
        className,
      )}
      {...props}
    >
      <RadioGroupPrimitive.Indicator className="size-1.5 rounded-full bg-accent-foreground" />
    </RadioGroupPrimitive.Item>
  );
}
