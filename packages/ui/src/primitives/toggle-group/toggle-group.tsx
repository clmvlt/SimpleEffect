import { cva, type VariantProps } from 'class-variance-authority';
import { ToggleGroup as ToggleGroupPrimitive } from 'radix-ui';
import { createContext, use, type ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';

/**
 * Group styles. `ghost`: free-standing toggles for tool palettes (on = accent tint).
 * `segmented`: a joined pill group on a control surface for exclusive choices (on = raised).
 */
export const toggleGroupVariants = cva('inline-flex shrink-0 items-center', {
  variants: {
    variant: {
      ghost: 'gap-0.5',
      segmented: 'gap-0.5 rounded-md bg-control p-0.5',
    },
    size: { xs: '', sm: '', md: '' },
  },
  compoundVariants: [
    { variant: 'segmented', size: 'xs', className: 'h-control-xs' },
    { variant: 'segmented', size: 'sm', className: 'h-control-sm' },
    { variant: 'segmented', size: 'md', className: 'h-control-md' },
  ],
  defaultVariants: { variant: 'ghost', size: 'sm' },
});

/** Item styles; the variant and size come from the enclosing `ToggleGroup`. */
export const toggleGroupItemVariants = cva(
  [
    'inline-flex items-center justify-center gap-1.5 whitespace-nowrap font-medium text-text-muted',
    'transition-colors duration-(--duration-fast) ease-standard hover:text-text',
    'disabled:pointer-events-none disabled:opacity-50 data-[state=on]:text-text',
    focusRing,
  ],
  {
    variants: {
      variant: {
        ghost: 'rounded-md hover:bg-hover data-[state=on]:bg-accent-muted',
        segmented: 'h-full rounded-sm data-[state=on]:bg-control-hover',
      },
      size: {
        xs: 'min-w-5 px-1 text-xs',
        sm: 'min-w-6 px-1.5 text-sm',
        md: 'min-w-7 px-2 text-sm',
      },
    },
    compoundVariants: [
      { variant: 'ghost', size: 'xs', className: 'h-control-xs min-w-control-xs' },
      { variant: 'ghost', size: 'sm', className: 'h-control-sm min-w-control-sm' },
      { variant: 'ghost', size: 'md', className: 'h-control-md min-w-control-md' },
    ],
    defaultVariants: { variant: 'ghost', size: 'sm' },
  },
);

/** Look of a toggle group, shared with its items. */
interface ToggleGroupStyle {
  variant: 'ghost' | 'segmented';
  size: 'xs' | 'sm' | 'md';
}

/** Lets items follow the look chosen on their `ToggleGroup` without repeating props. */
const ToggleGroupStyleContext = createContext<ToggleGroupStyle>({ variant: 'ghost', size: 'sm' });

/** Props of {@link ToggleGroup}. */
export type ToggleGroupProps = ComponentProps<typeof ToggleGroupPrimitive.Root> &
  VariantProps<typeof toggleGroupVariants>;

/**
 * A set of toggle buttons. `type="single"`: one choice at most (a tool, an alignment), exposed as
 * a radio group. `type="multiple"`: independent on/off toggles (bold, italic), exposed as a
 * toolbar of pressed buttons. The group is a single Tab stop: arrow keys move between items and
 * Space or Enter toggles. Give the group an `aria-label`.
 */
export function ToggleGroup({ variant, size, className, ...props }: ToggleGroupProps) {
  const style: ToggleGroupStyle = { variant: variant ?? 'ghost', size: size ?? 'sm' };
  return (
    <ToggleGroupStyleContext value={style}>
      <ToggleGroupPrimitive.Root className={cn(toggleGroupVariants(style), className)} {...props} />
    </ToggleGroupStyleContext>
  );
}

/** Props of {@link ToggleGroupItem}. */
export type ToggleGroupItemProps = ComponentProps<typeof ToggleGroupPrimitive.Item>;

/**
 * One toggle of a `ToggleGroup`. Icon-only items must have an `aria-label` (and usually a
 * `Tooltip` naming them); text items are named by their children.
 */
export function ToggleGroupItem({ className, ...props }: ToggleGroupItemProps) {
  const style = use(ToggleGroupStyleContext);
  return (
    <ToggleGroupPrimitive.Item
      className={cn(toggleGroupItemVariants(style), className)}
      {...props}
    />
  );
}
