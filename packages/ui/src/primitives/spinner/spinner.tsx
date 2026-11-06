import { LoaderCircle } from 'lucide-react';

import { cn } from '../../lib/cn';
import { Icon, type IconSize } from '../icon';

/** Props of {@link Spinner}. */
export interface SpinnerProps {
  /** Normalised size (default `md`, 16 px). */
  size?: IconSize;
  /**
   * Accessible status text (e.g. « Chargement… »). Omit it when the surrounding control already
   * announces the busy state (a loading `Button` sets `aria-busy`): the spinner is then decorative.
   */
  label?: string;
  className?: string;
}

/** Indeterminate activity indicator, in the current text colour. */
export function Spinner({ size = 'md', label, className }: SpinnerProps) {
  const spinner = <Icon icon={LoaderCircle} size={size} className="animate-spin" />;
  if (label === undefined) {
    return (
      <span aria-hidden="true" className={cn('inline-flex', className)}>
        {spinner}
      </span>
    );
  }
  return (
    <span role="status" aria-label={label} className={cn('inline-flex', className)}>
      {spinner}
    </span>
  );
}
