import { cva, type VariantProps } from 'class-variance-authority';
import { Tabs as TabsPrimitive } from 'radix-ui';
import { createContext, use, type ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';

/**
 * Tab list styles. `segmented`: a compact pill group on a control surface, for toolbars and
 * dense inspectors. `underline`: text tabs over a bottom border, for panel headers.
 */
export const tabsListVariants = cva('inline-flex shrink-0 items-center', {
  variants: {
    variant: {
      segmented: 'gap-0.5 rounded-md bg-control p-0.5',
      underline: 'gap-4 border-b border-border',
    },
    size: { xs: '', sm: '' },
  },
  compoundVariants: [
    { variant: 'segmented', size: 'xs', className: 'h-control-xs' },
    { variant: 'segmented', size: 'sm', className: 'h-control-sm' },
  ],
  defaultVariants: { variant: 'segmented', size: 'sm' },
});

/** Tab trigger styles; the variant and size come from the enclosing `TabsList`. */
export const tabsTriggerVariants = cva(
  [
    'inline-flex items-center justify-center gap-1.5 whitespace-nowrap font-medium text-text-muted',
    'transition-colors duration-(--duration-fast) ease-standard hover:text-text',
    'disabled:pointer-events-none disabled:opacity-50 data-[state=active]:text-text',
    focusRing,
  ],
  {
    variants: {
      variant: {
        segmented: 'h-full rounded-sm px-2 data-[state=active]:bg-control-hover',
        underline: '-mb-px border-b-2 border-transparent px-0.5 data-[state=active]:border-accent',
      },
      size: { xs: 'text-xs', sm: 'text-sm' },
    },
    compoundVariants: [
      { variant: 'underline', size: 'xs', className: 'h-control-xs' },
      { variant: 'underline', size: 'sm', className: 'h-control-sm' },
    ],
    defaultVariants: { variant: 'segmented', size: 'sm' },
  },
);

/** Look of a tab list, shared with its triggers. */
interface TabsListStyle {
  variant: 'segmented' | 'underline';
  size: 'xs' | 'sm';
}

/** Lets triggers follow the look chosen on their `TabsList` without repeating props. */
const TabsListStyleContext = createContext<TabsListStyle>({ variant: 'segmented', size: 'sm' });

/** Props of {@link Tabs}. */
export type TabsProps = ComponentProps<typeof TabsPrimitive.Root>;

/**
 * Switches between panels that share one place: `<Tabs defaultValue="video">` with a `TabsList`
 * of `TabsTrigger`s and one `TabsContent` per value. Arrow keys move between tabs, Home/End jump
 * to the ends, and the focused tab is activated.
 */
export function Tabs({ className, ...props }: TabsProps) {
  return <TabsPrimitive.Root className={cn('flex flex-col gap-2', className)} {...props} />;
}

/** Props of {@link TabsList}. */
export interface TabsListProps
  extends ComponentProps<typeof TabsPrimitive.List>, VariantProps<typeof tabsListVariants> {}

/** The row of tab triggers; its `variant` and `size` apply to every trigger inside. */
export function TabsList({ variant, size, className, ...props }: TabsListProps) {
  const style: TabsListStyle = { variant: variant ?? 'segmented', size: size ?? 'sm' };
  return (
    <TabsListStyleContext value={style}>
      <TabsPrimitive.List className={cn(tabsListVariants(style), className)} {...props} />
    </TabsListStyleContext>
  );
}

/** Props of {@link TabsTrigger}. */
export type TabsTriggerProps = ComponentProps<typeof TabsPrimitive.Trigger>;

/** One tab. Its label comes from `children` (an `Icon` may precede the text). */
export function TabsTrigger({ className, ...props }: TabsTriggerProps) {
  const style = use(TabsListStyleContext);
  return <TabsPrimitive.Trigger className={cn(tabsTriggerVariants(style), className)} {...props} />;
}

/** Props of {@link TabsContent}. */
export type TabsContentProps = ComponentProps<typeof TabsPrimitive.Content>;

/** The panel shown while its `value` is the active tab. Focusable, so Tab reaches it. */
export function TabsContent({ className, ...props }: TabsContentProps) {
  return <TabsPrimitive.Content className={cn('flex-1', focusRing, className)} {...props} />;
}
