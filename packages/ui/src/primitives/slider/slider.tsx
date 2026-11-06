import { Slider as SliderPrimitive } from 'radix-ui';
import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { focusRing } from '../../lib/focus-ring';

/** Props of {@link Slider}. */
export interface SliderProps extends ComponentProps<typeof SliderPrimitive.Root> {
  /**
   * Accessible name of each thumb, in order; their number is the number of thumbs. One label for a
   * single value (« Volume »), two for a range (« Début », « Fin »). `value`/`defaultValue` must
   * hold as many numbers.
   */
  thumbLabels: readonly string[];
}

/**
 * Picks a number (or a range) on a track. Keyboard: arrows step by `step`, Page Up/Down by ten
 * steps, Home/End jump to `min`/`max`. Each thumb is a separate `slider` named by `thumbLabels`.
 */
export function Slider({ thumbLabels, className, ...props }: SliderProps) {
  return (
    <SliderPrimitive.Root
      className={cn(
        'relative flex touch-none items-center select-none data-[disabled]:opacity-50',
        'data-[orientation=horizontal]:h-4 data-[orientation=horizontal]:w-full',
        'data-[orientation=vertical]:h-full data-[orientation=vertical]:min-h-32',
        'data-[orientation=vertical]:w-4 data-[orientation=vertical]:flex-col',
        className,
      )}
      {...props}
    >
      <SliderPrimitive.Track
        className={cn(
          'relative grow overflow-hidden rounded-full bg-border-strong',
          'data-[orientation=horizontal]:h-1 data-[orientation=horizontal]:w-full',
          'data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1',
        )}
      >
        <SliderPrimitive.Range
          className={cn(
            'absolute bg-accent',
            'data-[orientation=horizontal]:h-full data-[orientation=vertical]:w-full',
          )}
        />
      </SliderPrimitive.Track>
      {thumbLabels.map((label, index) => (
        <SliderPrimitive.Thumb
          // Thumbs are positional: their index is their identity.
          key={index}
          aria-label={label}
          className={cn(
            'block size-3 rounded-full border border-border-strong bg-text',
            'transition-transform duration-(--duration-fast) ease-standard hover:scale-110',
            'data-[disabled]:pointer-events-none',
            focusRing,
          )}
        />
      ))}
    </SliderPrimitive.Root>
  );
}
