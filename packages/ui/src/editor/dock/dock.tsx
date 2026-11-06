import { useState, type ComponentProps } from 'react';
import {
  Group as GroupPrimitive,
  Panel as PanelPrimitive,
  Separator as SeparatorPrimitive,
} from 'react-resizable-panels';

import { cn } from '../../lib/cn';
import { loadDockLayout, saveDockLayout, type DockLayoutStorage } from './dock-layout-storage';

/** Props of {@link Dock}. */
export interface DockProps extends Omit<
  ComponentProps<typeof GroupPrimitive>,
  'id' | 'defaultLayout' | 'onLayoutChanged' | 'orientation'
> {
  /**
   * Stable identifier of this layout (e.g. `editor.main`): sizes are saved under it, so it must be
   * unique in the app and must not change between versions without a reason.
   */
  layoutId: string;
  /** Persists the sizes the user drags to; omit it for a layout that always starts from defaults. */
  storage?: DockLayoutStorage | undefined;
  /** `horizontal` puts panels side by side (default), `vertical` stacks them. */
  orientation?: 'horizontal' | 'vertical' | undefined;
}

/**
 * A group of resizable panels (`DockPanel`) separated by `DockHandle`s. Nest docks to build the
 * editor layout. Sizes changed by the user (pointer or keyboard on a handle) are saved in
 * `storage` under `layoutId` and restored on the next start.
 */
export function Dock({
  layoutId,
  storage,
  orientation = 'horizontal',
  className,
  ...props
}: DockProps) {
  // Read once: later saves must not re-render the dock with a new default layout.
  const [savedLayout] = useState(() =>
    storage === undefined ? undefined : loadDockLayout(storage, layoutId),
  );
  return (
    <GroupPrimitive
      id={layoutId}
      orientation={orientation}
      defaultLayout={savedLayout}
      onLayoutChanged={(layout, meta) => {
        // Only what the user chose is remembered, not sizes forced by a small window: the
        // requested layout comes back when the window grows again.
        if (storage !== undefined && meta.isUserInteraction) {
          saveDockLayout(storage, layoutId, meta.requestedLayout ?? layout);
        }
      }}
      className={cn('size-full min-h-0 min-w-0', className)}
      {...props}
    />
  );
}

/** Props of {@link DockPanel}. */
export interface DockPanelProps extends Omit<ComponentProps<typeof PanelPrimitive>, 'id'> {
  /** Identifier of the panel inside its dock; saved sizes are matched by it. */
  id: string;
}

/**
 * One resizable area of a `Dock`. Sizes accept percentages (`"25%"`), pixels (`240`) or CSS units
 * (`"16rem"`): give editor panels a `minSize` in pixels so they never become unusable.
 */
export function DockPanel({ className, ...props }: DockPanelProps) {
  return <PanelPrimitive className={cn('size-full min-h-0 min-w-0', className)} {...props} />;
}

/** Props of {@link DockHandle}. */
export interface DockHandleProps extends ComponentProps<typeof SeparatorPrimitive> {
  /** Accessible name of the handle (default English « Resize panels »; the app passes i18n). */
  label?: string | undefined;
}

/**
 * The separator between two `DockPanel`s: a 1 px line with a wider invisible grab area that turns
 * accent on hover and drag. Focus it and use the arrow keys to resize from the keyboard; Enter
 * collapses or restores a collapsible neighbour, double-click resets the default size.
 */
export function DockHandle({ label = 'Resize panels', className, ...props }: DockHandleProps) {
  return (
    <SeparatorPrimitive
      aria-label={label}
      className={cn(
        'relative shrink-0 bg-border outline-none',
        'transition-colors duration-(--duration-fast) ease-standard',
        'aria-[orientation=vertical]:w-px aria-[orientation=horizontal]:h-px',
        'data-[separator=hover]:bg-accent data-[separator=active]:bg-accent',
        'data-[separator=focus]:bg-focus-ring',
        // Keyboard focus: a 1 px line is too thin to spot, so add a ring around it.
        'focus-visible:outline-2 focus-visible:outline-offset-0 focus-visible:outline-focus-ring',
        className,
      )}
      {...props}
    />
  );
}
