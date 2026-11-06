import { Switch as SwitchPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';

/** Props of {@link Switch}: Radix switch props (`checked`, `defaultChecked`, `onCheckedChange`…). */
export type SwitchProps = ComponentProps<typeof SwitchPrimitive.Root>;

/**
 * On/off toggle for settings that apply immediately (a checkbox is for choices confirmed later).
 * A compact 28 × 16 px track; name it with a `Label` or a horizontal `Field`.
 */
export function Switch({ className, ...props }: SwitchProps) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        'peer inline-flex h-4 w-7 shrink-0 items-center rounded-full border border-transparent',
        'bg-border-strong transition-colors duration-(--duration-fast) ease-standard',
        'data-[state=checked]:bg-accent disabled:cursor-not-allowed disabled:opacity-50',
        focusRing,
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          'pointer-events-none block size-3 rounded-full bg-text',
          'translate-x-0.5 transition-transform duration-(--duration-fast) ease-standard',
          'data-[state=checked]:translate-x-3.5 data-[state=checked]:bg-accent-foreground',
        )}
      />
    </SwitchPrimitive.Root>
  );
}
