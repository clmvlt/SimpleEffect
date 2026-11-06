import { Toolbar as ToolbarPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { IconButton, type IconButtonProps } from '../../primitives/icon-button';

/** Props of {@link Toolbar}. */
export interface ToolbarProps extends ComponentProps<typeof ToolbarPrimitive.Root> {
  /** Accessible name of the toolbar, e.g. « Outils de la timeline ». */
  label: string;
}

/**
 * A row of tools that behaves as one tab stop: Tab enters it, the arrow keys move between its
 * buttons (Home/End jump to the ends), Tab leaves it. Put `ToolbarButton`s in `ToolbarGroup`s
 * separated by `ToolbarSeparator`s.
 */
export function Toolbar({ label, orientation = 'horizontal', className, ...props }: ToolbarProps) {
  return (
    <ToolbarPrimitive.Root
      aria-label={label}
      orientation={orientation}
      className={cn('flex items-center gap-1', orientation === 'vertical' && 'flex-col', className)}
      {...props}
    />
  );
}

/** Props of {@link ToolbarGroup}. */
export interface ToolbarGroupProps extends ComponentProps<'div'> {
  /** Accessible name of the group when its purpose is not obvious from its buttons. */
  label?: string | undefined;
}

/** Related tools kept together (e.g. playback, editing tools). */
export function ToolbarGroup({ label, className, ...props }: ToolbarGroupProps) {
  return (
    <div
      role="group"
      aria-label={label}
      className={cn('flex items-center gap-0.5 in-aria-[orientation=vertical]:flex-col', className)}
      {...props}
    />
  );
}

/** Props of {@link ToolbarSeparator}. */
export type ToolbarSeparatorProps = ComponentProps<typeof ToolbarPrimitive.Separator>;

/** A thin divider between groups, perpendicular to the toolbar. */
export function ToolbarSeparator({ className, ...props }: ToolbarSeparatorProps) {
  return (
    <ToolbarPrimitive.Separator
      className={cn(
        'shrink-0 bg-border',
        'data-[orientation=vertical]:mx-1 data-[orientation=vertical]:h-4 data-[orientation=vertical]:w-px',
        'data-[orientation=horizontal]:my-1 data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-4',
        className,
      )}
      {...props}
    />
  );
}

/** Props of {@link ToolbarButton}. */
export interface ToolbarButtonProps extends IconButtonProps {
  /** Toggle tools (loop, snapping…): the pressed state, exposed as `aria-pressed`. */
  pressed?: boolean | undefined;
}

/**
 * An icon tool inside a `Toolbar` (it joins the toolbar's arrow-key navigation). Wrap it in a
 * `Tooltip` to show its label and shortcut. With `pressed`, it is a toggle drawn with the accent
 * tint while on.
 */
export function ToolbarButton({ pressed, className, ...props }: ToolbarButtonProps) {
  return (
    <ToolbarPrimitive.Button asChild>
      <IconButton
        aria-pressed={pressed}
        className={cn(
          pressed === true && 'bg-accent-muted text-text hover:bg-accent-muted',
          className,
        )}
        {...props}
      />
    </ToolbarPrimitive.Button>
  );
}
