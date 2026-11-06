import { cva, type VariantProps } from 'class-variance-authority';
import { ChevronDown } from 'lucide-react';
import { Select as SelectPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { Icon } from '../icon';
import { textFieldClasses } from '../input';

/**
 * Root of a select: holds the value (`value`/`defaultValue`, `onValueChange`) and the open state.
 * Compose it like shadcn: `Select` › `SelectTrigger` (› `SelectValue`) + `SelectContent` ›
 * `SelectItem`s. Keyboard: Enter, Space or arrows open it; arrows move; Enter selects; Escape closes
 * and returns focus to the trigger; typing jumps to the matching item.
 */
export const Select = SelectPrimitive.Root;

/** Groups related items under a `SelectLabel`. */
export const SelectGroup = SelectPrimitive.Group;

/** Shows the selected item's text inside the trigger, or `placeholder` when nothing is selected. */
export const SelectValue = SelectPrimitive.Value;

/** Trigger styles: a text field look, at the control heights (24/28/32 px, default `sm`). */
export const selectTriggerVariants = cva(
  [
    ...textFieldClasses,
    'flex items-center justify-between gap-2 px-2 text-left whitespace-nowrap',
    'data-[placeholder]:text-text-subtle [&>span]:truncate',
  ],
  {
    variants: {
      size: {
        xs: 'h-control-xs text-xs',
        sm: 'h-control-sm text-sm',
        md: 'h-control-md text-sm',
      },
    },
    defaultVariants: { size: 'sm' },
  },
);

/** Props of {@link SelectTrigger}. */
export interface SelectTriggerProps
  extends
    ComponentProps<typeof SelectPrimitive.Trigger>,
    VariantProps<typeof selectTriggerVariants> {}

/**
 * The button that opens the list. Name it with a `Label`/`Field` (wrap the trigger in `Field`) or
 * `aria-label`. Put a `SelectValue` inside.
 */
export function SelectTrigger({ size, className, children, ...props }: SelectTriggerProps) {
  return (
    <SelectPrimitive.Trigger className={cn(selectTriggerVariants({ size }), className)} {...props}>
      {children}
      <SelectPrimitive.Icon asChild>
        <Icon icon={ChevronDown} size="sm" className="text-text-subtle" />
      </SelectPrimitive.Icon>
    </SelectPrimitive.Trigger>
  );
}
