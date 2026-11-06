import { Tooltip as TooltipPrimitive } from 'radix-ui';
import type { ComponentProps, ReactElement, ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { Kbd } from '../kbd';

/** Props of {@link TooltipProvider}. */
export type TooltipProviderProps = ComponentProps<typeof TooltipPrimitive.Provider>;

/**
 * Shares tooltip timing across the app: mount it once at the root. After one tooltip closed, the
 * next one opens instantly for `skipDelayDuration`, so scanning a toolbar feels immediate.
 */
export function TooltipProvider({
  delayDuration = 400,
  skipDelayDuration = 300,
  ...props
}: TooltipProviderProps) {
  return (
    <TooltipPrimitive.Provider
      delayDuration={delayDuration}
      skipDelayDuration={skipDelayDuration}
      {...props}
    />
  );
}

/** Props of {@link Tooltip}. */
export interface TooltipProps extends Omit<
  ComponentProps<typeof TooltipPrimitive.Root>,
  'children'
> {
  /** What the tooltip says (a short label, from i18n). */
  content: ReactNode;
  /** Keyboard shortcut of the action, shown as key caps after the label: `['Ctrl', 'K']`. */
  shortcut?: readonly string[];
  /** Side of the trigger where the tooltip appears (default `top`). */
  side?: ComponentProps<typeof TooltipPrimitive.Content>['side'];
  /** Alignment along that side (default `center`). */
  align?: ComponentProps<typeof TooltipPrimitive.Content>['align'];
  /** Extra classes for the tooltip surface. */
  className?: string;
  /** The trigger: a single focusable element (rendered as is, `asChild`). */
  children: ReactElement;
}

const tooltipContentClasses = [
  'z-(--z-tooltip) flex items-center gap-2 rounded-md border border-border bg-panel-raised',
  'px-2 py-1 text-xs text-text shadow-overlay select-none',
  'data-[state=delayed-open]:animate-overlay-in data-[state=closed]:animate-overlay-out',
];

/**
 * A short label shown on hover and keyboard focus, with an optional keyboard shortcut.
 * Requires a `TooltipProvider` ancestor. Wrap icon-only buttons so their action is discoverable:
 * `<Tooltip content="Couper" shortcut={['Ctrl', 'K']}><IconButton … /></Tooltip>`.
 */
export function Tooltip({
  content,
  shortcut,
  side = 'top',
  align = 'center',
  className,
  children,
  ...rootProps
}: TooltipProps) {
  return (
    <TooltipPrimitive.Root {...rootProps}>
      <TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content
          side={side}
          align={align}
          sideOffset={6}
          collisionPadding={8}
          className={cn(tooltipContentClasses, className)}
        >
          {content}
          {shortcut !== undefined && <Kbd keys={shortcut} />}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
}
