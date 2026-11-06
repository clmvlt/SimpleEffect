import { Check, Minus } from 'lucide-react';
import { Checkbox as CheckboxPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';
import { Icon } from '../icon';

/** Props of {@link Checkbox}: Radix checkbox props (`checked`, `defaultChecked`, `onCheckedChange`…). */
export type CheckboxProps = ComponentProps<typeof CheckboxPrimitive.Root>;

/**
 * A 16 px checkbox. Pass `checked="indeterminate"` for a partial selection (e.g. "some tracks
 * locked"): it shows a dash and is announced as mixed. Name it with a `Label` or a horizontal
 * `Field`; toggled with Space or a click.
 */
export function Checkbox({ className, ...props }: CheckboxProps) {
  return (
    <CheckboxPrimitive.Root
      className={cn(
        'peer group inline-flex size-4 shrink-0 items-center justify-center rounded-sm',
        'border border-border-strong bg-input text-accent-foreground',
        'transition-colors duration-(--duration-fast) ease-standard hover:border-text-subtle',
        'data-[state=checked]:border-accent data-[state=checked]:bg-accent',
        'data-[state=indeterminate]:border-accent data-[state=indeterminate]:bg-accent',
        'aria-invalid:border-danger disabled:cursor-not-allowed disabled:opacity-50',
        focusRing,
        className,
      )}
      {...props}
    >
      <CheckboxPrimitive.Indicator className="flex items-center justify-center">
        <Icon
          icon={Check}
          size="xs"
          strokeWidth={3}
          className="hidden group-data-[state=checked]:block"
        />
        <Icon
          icon={Minus}
          size="xs"
          strokeWidth={3}
          className="hidden group-data-[state=indeterminate]:block"
        />
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  );
}
