import { Separator as SeparatorPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/** Props of {@link Separator}. */
export type SeparatorProps = ComponentProps<typeof SeparatorPrimitive.Root>;

/**
 * A 1 px line between groups of content, horizontal (default) or vertical.
 *
 * Decorative by default: it is hidden from assistive technologies because most separators only
 * draw a visual boundary. Pass `decorative={false}` when it marks a real boundary between sections
 * (it then gets `role="separator"` and `aria-orientation`).
 */
export function Separator({
  orientation = 'horizontal',
  decorative = true,
  className,
  ...props
}: SeparatorProps) {
  return (
    <SeparatorPrimitive.Root
      orientation={orientation}
      decorative={decorative}
      className={cn(
        'shrink-0 bg-border',
        orientation === 'horizontal' ? 'h-px w-full' : 'h-full w-px self-stretch',
        className,
      )}
      {...props}
    />
  );
}
