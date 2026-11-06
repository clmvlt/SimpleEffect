import type { VariantProps } from 'class-variance-authority';
import { Check, ChevronRight, Circle } from 'lucide-react';
import { DropdownMenu as DropdownMenuPrimitive } from 'radix-ui';
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
 * A menu of commands opened from a button (toolbar overflow, « Fichier »…). Fully keyboard
 * operable: arrows move, Enter/Space select, typing jumps to an item, Escape closes. Compose:
 * `<DropdownMenu><DropdownMenuTrigger asChild>…</DropdownMenuTrigger><DropdownMenuContent>
 * <DropdownMenuItem onSelect={…}>…</DropdownMenuItem></DropdownMenuContent></DropdownMenu>`.
 */
export const DropdownMenu = DropdownMenuPrimitive.Root;

/** The button that opens the menu; use `asChild` to keep your own button. */
export const DropdownMenuTrigger = DropdownMenuPrimitive.Trigger;

/** Groups related items (no visual effect; pair with a label or separators). */
export const DropdownMenuGroup = DropdownMenuPrimitive.Group;

/** A set of mutually exclusive `DropdownMenuRadioItem`s bound to `value`/`onValueChange`. */
export const DropdownMenuRadioGroup = DropdownMenuPrimitive.RadioGroup;

/** Wraps a `DropdownMenuSubTrigger` and its `DropdownMenuSubContent`. */
export const DropdownMenuSub = DropdownMenuPrimitive.Sub;

/** Props of {@link DropdownMenuContent}. */
export type DropdownMenuContentProps = ComponentProps<typeof DropdownMenuPrimitive.Content>;

/** The floating list of items, rendered in a portal. */
export function DropdownMenuContent({
  className,
  sideOffset = 4,
  collisionPadding = 8,
  ...props
}: DropdownMenuContentProps) {
  return (
    <DropdownMenuPrimitive.Portal>
      <DropdownMenuPrimitive.Content
        sideOffset={sideOffset}
        collisionPadding={collisionPadding}
        className={cn(
          menuContentClasses,
          'max-h-(--radix-dropdown-menu-content-available-height)',
          className,
        )}
        {...props}
      />
    </DropdownMenuPrimitive.Portal>
  );
}

/** Props of {@link DropdownMenuItem}. */
export type DropdownMenuItemProps = ComponentProps<typeof DropdownMenuPrimitive.Item> &
  VariantProps<typeof menuItemVariants>;

/** A command. React to `onSelect`; `variant="danger"` for destructive commands. */
export function DropdownMenuItem({ variant, inset, className, ...props }: DropdownMenuItemProps) {
  return (
    <DropdownMenuPrimitive.Item
      className={cn(menuItemVariants({ variant, inset }), className)}
      {...props}
    />
  );
}

/** An on/off option with a check mark (`checked`/`onCheckedChange`). */
export function DropdownMenuCheckboxItem({
  className,
  children,
  ...props
}: ComponentProps<typeof DropdownMenuPrimitive.CheckboxItem>) {
  return (
    <DropdownMenuPrimitive.CheckboxItem
      className={cn(menuIndicatorItemClasses, className)}
      {...props}
    >
      <span className={menuIndicatorClasses}>
        <DropdownMenuPrimitive.ItemIndicator>
          <Icon icon={Check} size="sm" />
        </DropdownMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </DropdownMenuPrimitive.CheckboxItem>
  );
}

/** One choice of a `DropdownMenuRadioGroup`, with a dot when selected. */
export function DropdownMenuRadioItem({
  className,
  children,
  ...props
}: ComponentProps<typeof DropdownMenuPrimitive.RadioItem>) {
  return (
    <DropdownMenuPrimitive.RadioItem className={cn(menuIndicatorItemClasses, className)} {...props}>
      <span className={menuIndicatorClasses}>
        <DropdownMenuPrimitive.ItemIndicator>
          <Icon icon={Circle} size="xs" className="fill-current" />
        </DropdownMenuPrimitive.ItemIndicator>
      </span>
      {children}
    </DropdownMenuPrimitive.RadioItem>
  );
}

/** Props of {@link DropdownMenuLabel}. */
export type DropdownMenuLabelProps = ComponentProps<typeof DropdownMenuPrimitive.Label> & {
  /** Align with indicator rows (checkbox/radio items). */
  inset?: boolean;
};

/** Non-interactive heading of a group of items. */
export function DropdownMenuLabel({ inset = false, className, ...props }: DropdownMenuLabelProps) {
  return (
    <DropdownMenuPrimitive.Label
      className={cn(menuLabelClasses, inset && 'pl-7', className)}
      {...props}
    />
  );
}

/** Divider between groups of items. */
export function DropdownMenuSeparator({
  className,
  ...props
}: ComponentProps<typeof DropdownMenuPrimitive.Separator>) {
  return (
    <DropdownMenuPrimitive.Separator className={cn(menuSeparatorClasses, className)} {...props} />
  );
}

/** Keyboard shortcut shown at the end of an item (text such as `Ctrl+D`). */
export function DropdownMenuShortcut({ className, ...props }: ComponentProps<'span'>) {
  return <span className={cn(menuShortcutClasses, className)} {...props} />;
}

/** Props of {@link DropdownMenuSubTrigger}. */
export type DropdownMenuSubTriggerProps = ComponentProps<
  typeof DropdownMenuPrimitive.SubTrigger
> & {
  /** Align with indicator rows (checkbox/radio items). */
  inset?: boolean;
};

/** The item that opens a submenu (hover, Enter or Right arrow), with a chevron. */
export function DropdownMenuSubTrigger({
  inset = false,
  className,
  children,
  ...props
}: DropdownMenuSubTriggerProps) {
  return (
    <DropdownMenuPrimitive.SubTrigger
      className={cn(menuItemVariants({ inset }), menuSubTriggerClasses, className)}
      {...props}
    >
      {children}
      <Icon icon={ChevronRight} size="sm" className="ml-auto text-text-subtle" />
    </DropdownMenuPrimitive.SubTrigger>
  );
}

/** The floating list of a submenu, rendered in a portal. */
export function DropdownMenuSubContent({
  className,
  collisionPadding = 8,
  ...props
}: ComponentProps<typeof DropdownMenuPrimitive.SubContent>) {
  return (
    <DropdownMenuPrimitive.Portal>
      <DropdownMenuPrimitive.SubContent
        collisionPadding={collisionPadding}
        className={cn(menuContentClasses, className)}
        {...props}
      />
    </DropdownMenuPrimitive.Portal>
  );
}
