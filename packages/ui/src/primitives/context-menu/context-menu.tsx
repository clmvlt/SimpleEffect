import type { VariantProps } from 'class-variance-authority';
import { Check, ChevronRight, Circle } from 'lucide-react';
import { ContextMenu as ContextMenuPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { Icon } from '../icon';
import {
  menuContentClasses,
  menuIndicatorClasses,
  menuIndicatorItemClasses,
  menuItemVariants,
  menuLabelClasses,
  menuSeparatorClasses,
  menuShortcutClasses,
  menuSubTriggerClasses,
} from '../menu/menu-styles';

/**
 * A menu opened by right-click (or the context-menu key / Shift+F10) on an area: clips, tracks,
 * media. Same parts and keyboard behaviour as `DropdownMenu`. Compose:
 * `<ContextMenu><ContextMenuTrigger asChild><div>…</div></ContextMenuTrigger><ContextMenuContent>
 * <ContextMenuItem onSelect={…}>…</ContextMenuItem></ContextMenuContent></ContextMenu>`.
 */
export const ContextMenu = ContextMenuPrimitive.Root;

/** The area that opens the menu on right-click; use `asChild` to keep your own element. */
export const ContextMenuTrigger = ContextMenuPrimitive.Trigger;

/** Groups related items (no visual effect; pair with a label or separators). */
export const ContextMenuGroup = ContextMenuPrimitive.Group;

/** A set of mutually exclusive `ContextMenuRadioItem`s bound to `value`/`onValueChange`. */
export const ContextMenuRadioGroup = ContextMenuPrimitive.RadioGroup;

/** Wraps a `ContextMenuSubTrigger` and its `ContextMenuSubContent`. */
export const ContextMenuSub = ContextMenuPrimitive.Sub;

/** Props of {@link ContextMenuContent}. */
export type ContextMenuContentProps = ComponentProps<typeof ContextMenuPrimitive.Content>;

/** The floating list of items, rendered in a portal at the pointer. */
export function ContextMenuContent({
  className,
  collisionPadding = 8,
  ...props
}: ContextMenuContentProps) {
  return (
    <ContextMenuPrimitive.Portal>
      <ContextMenuPrimitive.Content
        collisionPadding={collisionPadding}
        className={cn(
          menuContentClasses,
          'max-h-(--radix-context-menu-content-available-height)',
          className,
        )}
        {...props}
      />
    </ContextMenuPrimitive.Portal>
  );
}

/** Props of {@link ContextMenuItem}. */
export type ContextMenuItemProps = ComponentProps<typeof ContextMenuPrimitive.Item> &
  VariantProps<typeof menuItemVariants>;

/** A command. React to `onSelect`; `variant="danger"` for destructive commands. */
export function ContextMenuItem({ variant, inset, className, ...props }: ContextMenuItemProps) {
  return (
    <ContextMenuPrimitive.Item
      className={cn(menuItemVariants({ variant, inset }), className)}
      {...props}
    />
  );
}

/** An on/off option with a check mark (`checked`/`onCheckedChange`). */
export function ContextMenuCheckboxItem({
  className,
  children,
  ...props
}: ComponentProps<typeof ContextMenuPrimitive.CheckboxItem>) {
  return (
    <ContextMenuPrimitive.CheckboxItem
      className={cn(menuIndicatorItemClasses, className)}
      {...props}
    >
      <span className={menuIndicatorClasses}>
        <ContextMenuPrimitive.ItemIndicator>
          <Icon icon={Check} size="sm" />
        </ContextMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </ContextMenuPrimitive.CheckboxItem>
  );
}

/** One choice of a `ContextMenuRadioGroup`, with a dot when selected. */
export function ContextMenuRadioItem({
  className,
  children,
  ...props
}: ComponentProps<typeof ContextMenuPrimitive.RadioItem>) {
  return (
    <ContextMenuPrimitive.RadioItem className={cn(menuIndicatorItemClasses, className)} {...props}>
      <span className={menuIndicatorClasses}>
        <ContextMenuPrimitive.ItemIndicator>
          <Icon icon={Circle} size="xs" className="fill-current" />
        </ContextMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </ContextMenuPrimitive.RadioItem>
  );
}

/** Props of {@link ContextMenuLabel}. */
export type ContextMenuLabelProps = ComponentProps<typeof ContextMenuPrimitive.Label> & {
  /** Align with indicator rows (checkbox/radio items). */
  inset?: boolean;
};

/** Non-interactive heading of a group of items. */
export function ContextMenuLabel({ inset = false, className, ...props }: ContextMenuLabelProps) {
  return (
    <ContextMenuPrimitive.Label
      className={cn(menuLabelClasses, inset && 'pl-7', className)}
      {...props}
    />
  );
}

/** Divider between groups of items. */
export function ContextMenuSeparator({
  className,
  ...props
}: ComponentProps<typeof ContextMenuPrimitive.Separator>) {
  return (
    <ContextMenuPrimitive.Separator className={cn(menuSeparatorClasses, className)} {...props} />
  );
}

/** Keyboard shortcut shown at the end of an item (text such as `Ctrl+D`). */
export function ContextMenuShortcut({ className, ...props }: ComponentProps<'span'>) {
  return <span className={cn(menuShortcutClasses, className)} {...props} />;
}

/** Props of {@link ContextMenuSubTrigger}. */
export type ContextMenuSubTriggerProps = ComponentProps<typeof ContextMenuPrimitive.SubTrigger> & {
  /** Align with indicator rows (checkbox/radio items). */
  inset?: boolean;
};

/** The item that opens a submenu (hover, Enter or Right arrow), with a chevron. */
export function ContextMenuSubTrigger({
  inset = false,
  className,
  children,
  ...props
}: ContextMenuSubTriggerProps) {
  return (
    <ContextMenuPrimitive.SubTrigger
      className={cn(menuItemVariants({ inset }), menuSubTriggerClasses, className)}
      {...props}
    >
      {children}
      <Icon icon={ChevronRight} size="sm" className="ml-auto text-text-subtle" />
    </ContextMenuPrimitive.SubTrigger>
  );
}

/** The floating list of a submenu, rendered in a portal. */
export function ContextMenuSubContent({
  className,
  collisionPadding = 8,
  ...props
}: ComponentProps<typeof ContextMenuPrimitive.SubContent>) {
  return (
    <ContextMenuPrimitive.Portal>
      <ContextMenuPrimitive.SubContent
        collisionPadding={collisionPadding}
        className={cn(menuContentClasses, className)}
        {...props}
      />
    </ContextMenuPrimitive.Portal>
  );
}
