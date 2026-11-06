import { cva } from 'class-variance-authority';

import { floatingSurfaceClasses } from '../../lib/floating';

/**
 * Styles shared by `DropdownMenu` and `ContextMenu` (internal to @se/ui). Radix gives both menus
 * the same parts and data attributes, so one set of classes keeps them visually identical.
 */

/** The floating list (root menu and submenus). */
export const menuContentClasses = [
  'z-(--z-dropdown) min-w-44 overflow-x-hidden overflow-y-auto p-1 outline-none',
  floatingSurfaceClasses,
];

/** A menu row. `danger` is for destructive commands; `inset` aligns text with indicator rows. */
export const menuItemVariants = cva(
  [
    'relative flex min-h-control-sm cursor-default items-center gap-2 rounded-sm px-2 text-sm',
    'outline-none select-none data-highlighted:bg-control-hover',
    'data-disabled:pointer-events-none data-disabled:opacity-50',
  ],
  {
    variants: {
      variant: {
        default: 'text-text',
        danger: 'text-danger data-highlighted:bg-danger-muted',
      },
      inset: { true: 'pl-7', false: '' },
    },
    defaultVariants: { variant: 'default', inset: false },
  },
);

/** A row with a check mark or radio dot on the left (checkbox and radio items). */
export const menuIndicatorItemClasses = menuItemVariants({ inset: true });

/** The slot holding the check mark or radio dot of an indicator row. */
export const menuIndicatorClasses =
  'pointer-events-none absolute left-2 inline-flex size-3.5 items-center justify-center';

/** Row opening a submenu: highlighted while its submenu is open. */
export const menuSubTriggerClasses = 'data-[state=open]:bg-control-hover';

/** Non-interactive group heading. */
export const menuLabelClasses = 'px-2 py-1 text-xs font-medium text-text-subtle';

/** Thin divider between groups of rows. */
export const menuSeparatorClasses = '-mx-1 my-1 h-px bg-border';

/** Keyboard shortcut text at the end of a row. */
export const menuShortcutClasses = 'ml-auto pl-4 text-xs tracking-wide text-text-subtle';
