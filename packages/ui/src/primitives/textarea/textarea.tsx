import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';
import { textFieldClasses } from '../input';

/** Props of {@link Textarea}. */
export type TextareaProps = ComponentProps<'textarea'>;

/**
 * Multi-line text field (prompts, notes, caption text). Grows with its content up to `max-h-64`,
 * then scrolls; the user can still resize it vertically. Same focus and invalid styles as `Input`.
 */
export function Textarea({ className, rows = 3, ...props }: TextareaProps) {
  return (
    <textarea
      rows={rows}
      className={cn(
        textFieldClasses,
        'field-sizing-content max-h-64 min-h-16 resize-y px-2 py-1.5 text-sm',
        className,
      )}
      {...props}
    />
  );
}
