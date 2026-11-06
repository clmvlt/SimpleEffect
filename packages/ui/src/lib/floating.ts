/**
 * Surface of floating panels anchored to a trigger (popovers, menus, select lists): overlay colour,
 * thin border, the only shadow of the system, and the enter/exit animation bound to Radix states.
 * Callers add their own stacking layer (`z-(--z-dropdown)`) and padding.
 */
export const floatingSurfaceClasses = [
  'rounded-lg border border-border bg-overlay text-text shadow-overlay',
  'data-[state=open]:animate-content-in data-[state=closed]:animate-content-out',
];
