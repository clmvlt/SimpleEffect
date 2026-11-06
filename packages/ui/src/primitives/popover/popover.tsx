import { Popover as PopoverPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { floatingSurfaceClasses } from '../../lib/floating';

/**
 * A floating panel anchored to a trigger, for rich, interactive content (pickers, small forms).
 * Closes on Escape and outside click; focus moves into the content and back to the trigger.
 * Compose: `<Popover><PopoverTrigger asChild>…</PopoverTrigger><PopoverContent>…</PopoverContent></Popover>`.
 */
export const Popover = PopoverPrimitive.Root;

/** The element that toggles the popover; use `asChild` to keep your own button. */
export const PopoverTrigger = PopoverPrimitive.Trigger;

/** Positions the popover against another element than its trigger. */
export const PopoverAnchor = PopoverPrimitive.Anchor;

/** Closes the popover when activated (e.g. an « Appliquer » button inside it). */
export const PopoverClose = PopoverPrimitive.Close;

/** Props of {@link PopoverContent}. */
export type PopoverContentProps = ComponentProps<typeof PopoverPrimitive.Content>;

/** The floating panel, rendered in a portal above panels and dialogs. */
export function PopoverContent({
  className,
  align = 'center',
  sideOffset = 6,
  collisionPadding = 8,
  ...props
}: PopoverContentProps) {
  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        align={align}
        sideOffset={sideOffset}
        collisionPadding={collisionPadding}
        className={cn('z-(--z-dropdown) w-64 p-3 outline-none', floatingSurfaceClasses, className)}
        {...props}
      />
    </PopoverPrimitive.Portal>
  );
}
