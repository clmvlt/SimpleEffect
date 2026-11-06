import { clsx, type ClassValue } from 'clsx';
import { extendTailwindMerge } from 'tailwind-merge';

/**
 * tailwind-merge only knows Tailwind's default theme. Every custom token whose utility name it
 * cannot guess must be declared here, otherwise conflicting classes are not deduplicated
 * (`cn('h-8', 'h-control-md')` would keep both). Keep in sync with `tokens/*.css`.
 */
const mergeClasses = extendTailwindMerge({
  extend: {
    theme: {
      spacing: ['control-xs', 'control-sm', 'control-md'],
      shadow: ['overlay'],
      ease: ['standard'],
      animate: [
        'overlay-in',
        'overlay-out',
        'content-in',
        'content-out',
        'progress-indeterminate',
        'easing-preview',
      ],
    },
  },
});

/**
 * Joins class names (conditional ones included, via clsx) and resolves Tailwind conflicts so the
 * last class wins: `cn('px-2 text-text', isActive && 'text-accent', className)`.
 */
export function cn(...inputs: ClassValue[]): string {
  return mergeClasses(clsx(inputs));
}
