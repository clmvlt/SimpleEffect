import { ScrollArea as ScrollAreaPrimitive } from 'radix-ui';
import type { ComponentProps, Ref } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';

/** Which scrollbars a {@link ScrollArea} shows. */
export type ScrollAreaScrollbars = 'vertical' | 'horizontal' | 'both';

/** Props of {@link ScrollArea}. */
export interface ScrollAreaProps extends ComponentProps<typeof ScrollAreaPrimitive.Root> {
  /** Scroll directions (default `vertical`). */
  scrollbars?: ScrollAreaScrollbars;
  /**
   * Makes the viewport focusable (with a focus ring) so keyboard users can scroll content that
   * contains no focusable element, such as logs or long descriptions.
   */
  keyboardScrollable?: boolean;
  /** Ref to the scrolling element, for virtualised lists or scroll syncing. */
  viewportRef?: Ref<HTMLDivElement>;
}

/**
 * A scrollable region with thin themed scrollbars that appear on hover, identical on every OS.
 * Give it a bounded size (`h-64`, `flex-1 min-h-0`…): it scrolls its children within that box.
 */
export function ScrollArea({
  scrollbars = 'vertical',
  keyboardScrollable = false,
  viewportRef,
  className,
  children,
  ...props
}: ScrollAreaProps) {
  return (
    <ScrollAreaPrimitive.Root className={cn('relative overflow-hidden', className)} {...props}>
      <ScrollAreaPrimitive.Viewport
        ref={viewportRef}
        tabIndex={keyboardScrollable ? 0 : undefined}
        className={cn('size-full rounded-[inherit]', keyboardScrollable && focusRing)}
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      {scrollbars !== 'horizontal' && <ScrollBar orientation="vertical" />}
      {scrollbars !== 'vertical' && <ScrollBar orientation="horizontal" />}
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  );
}

/** Props of {@link ScrollBar}. */
export type ScrollBarProps = ComponentProps<typeof ScrollAreaPrimitive.ScrollAreaScrollbar>;

/**
 * One thin scrollbar of a `ScrollArea` (8 px track, thumb brightening on hover). `ScrollArea`
 * already renders the ones it needs; use this only when composing Radix-level parts yourself.
 */
export function ScrollBar({ orientation = 'vertical', className, ...props }: ScrollBarProps) {
  return (
    <ScrollAreaPrimitive.ScrollAreaScrollbar
      orientation={orientation}
      className={cn(
        'flex touch-none p-px select-none',
        orientation === 'vertical' ? 'h-full w-2' : 'h-2 flex-col',
        className,
      )}
      {...props}
    >
      <ScrollAreaPrimitive.ScrollAreaThumb className="relative flex-1 rounded-full bg-border-strong transition-colors duration-(--duration-fast) hover:bg-text-subtle" />
    </ScrollAreaPrimitive.ScrollAreaScrollbar>
  );
}
