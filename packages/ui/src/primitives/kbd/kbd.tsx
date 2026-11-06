import type { ComponentProps } from 'react';

import { cn } from '../../lib/cn';

/** Props of {@link Kbd}. */
export interface KbdProps extends ComponentProps<'kbd'> {
  /**
   * Keys of a shortcut, rendered as separate key caps: `['Ctrl', 'K']`. When omitted, `children`
   * is rendered as a single key cap.
   */
  keys?: readonly string[];
}

const keyCapClasses =
  'inline-flex h-4 min-w-4 items-center justify-center rounded-xs border border-border-strong bg-control px-1 font-mono text-2xs leading-none text-text-muted tabular-nums';

/**
 * A keyboard key or shortcut, e.g. in tooltips and menus. With `keys`, renders one key cap per key
 * inside a wrapping `<kbd>` (the HTML idiom for a key combination).
 */
export function Kbd({ keys, className, children, ...props }: KbdProps) {
  if (keys === undefined) {
    return (
      <kbd className={cn(keyCapClasses, className)} {...props}>
        {children}
      </kbd>
    );
  }
  return (
    <kbd className={cn('inline-flex items-center gap-0.5', className)} {...props}>
      {keys.map((key, index) => (
        // Keys of a shortcut may repeat (e.g. `['G', 'G']`), so the index is part of the key.
        <kbd key={`${key}-${String(index)}`} className={keyCapClasses}>
          {key}
        </kbd>
      ))}
    </kbd>
  );
}
