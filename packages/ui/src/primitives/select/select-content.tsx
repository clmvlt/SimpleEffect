import { Check, ChevronDown, ChevronUp } from 'lucide-react';
import { Select as SelectPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { floatingSurfaceClasses } from '../../lib/floating';
import { Icon } from '../icon';

/** Props of {@link SelectContent}. */
export type SelectContentProps = ComponentProps<typeof SelectPrimitive.Content>;

/**
 * The floating list, rendered in a portal on the dropdown layer. Defaults to `position="popper"`:
 * a dropdown under the trigger, at least as wide as it and never taller than the window allows
 * (pass `position="item-aligned"` for the macOS-style list over the trigger; `sideOffset` then has
 * no effect).
 */
export function SelectContent({
  className,
  children,
  position = 'popper',
  sideOffset = 4,
  ...props
}: SelectContentProps) {
  const popper = position === 'popper';
  return (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Content
        position={position}
        sideOffset={sideOffset}
        className={cn(
          'relative z-(--z-dropdown) max-h-(--radix-select-content-available-height) min-w-32',
          'overflow-hidden',
          floatingSurfaceClasses,
          popper && 'min-w-(--radix-select-trigger-width)',
          className,
        )}
        {...props}
      >
        <SelectScrollButton direction="up" />
        <SelectPrimitive.Viewport className="p-1">{children}</SelectPrimitive.Viewport>
        <SelectScrollButton direction="down" />
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  );
}

/** Arrow shown at the top or bottom of a long list; hovering it scrolls the list. */
function SelectScrollButton({ direction }: { direction: 'up' | 'down' }) {
  const ScrollButton =
    direction === 'up' ? SelectPrimitive.ScrollUpButton : SelectPrimitive.ScrollDownButton;
  return (
    <ScrollButton className="flex h-5 cursor-default items-center justify-center text-text-subtle">
      <Icon icon={direction === 'up' ? ChevronUp : ChevronDown} size="sm" />
    </ScrollButton>
  );
}

/** Props of {@link SelectItem}. */
export type SelectItemProps = ComponentProps<typeof SelectPrimitive.Item>;

/** One option; `value` identifies it, `children` is its visible text. A check marks the selection. */
export function SelectItem({ className, children, ...props }: SelectItemProps) {
  return (
    <SelectPrimitive.Item
      className={cn(
        'relative flex min-h-control-xs cursor-default items-center rounded-sm py-1 pr-7 pl-2',
        'text-sm text-text outline-none select-none data-highlighted:bg-control-hover',
        'data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        className,
      )}
      {...props}
    >
      <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
      <span className="absolute right-2 flex items-center">
        <SelectPrimitive.ItemIndicator>
          <Icon icon={Check} size="sm" className="text-text" />
        </SelectPrimitive.ItemIndicator>
      </span>
    </SelectPrimitive.Item>
  );
}

/** Props of {@link SelectLabel}. */
export type SelectLabelProps = ComponentProps<typeof SelectPrimitive.Label>;

/** Heading of a `SelectGroup`; not selectable. */
export function SelectLabel({ className, ...props }: SelectLabelProps) {
  return (
    <SelectPrimitive.Label
      className={cn('px-2 pt-1.5 pb-1 text-2xs font-medium text-text-subtle', className)}
      {...props}
    />
  );
}

/** Props of {@link SelectSeparator}. */
export type SelectSeparatorProps = ComponentProps<typeof SelectPrimitive.Separator>;

/** Thin line between groups of items. */
export function SelectSeparator({ className, ...props }: SelectSeparatorProps) {
  return (
    <SelectPrimitive.Separator className={cn('-mx-1 my-1 h-px bg-border', className)} {...props} />
  );
}
