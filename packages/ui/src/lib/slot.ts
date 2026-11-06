import { Slot as SlotPrimitive } from 'radix-ui';

/**
 * Renders its single child instead of its own element, merging props, classes and refs into it.
 * Powers the `asChild` prop: `<Button asChild><a href="…">Docs</a></Button>` renders a styled `<a>`.
 */
export const Slot: typeof SlotPrimitive.Root = SlotPrimitive.Root;

/**
 * Marks which child of a `Slot` receives the props when the component adds siblings around
 * `children` (e.g. a loading spinner before the label).
 */
export const Slottable: typeof SlotPrimitive.Slottable = SlotPrimitive.Slottable;
