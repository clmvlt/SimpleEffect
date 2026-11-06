import type { LucideIcon } from 'lucide-react';
import type { ComponentProps, ReactNode } from 'react';

import { cn } from '../../lib/cn';
import { Icon } from '../../primitives/icon';

/** Props of {@link EmptyState}. */
export interface EmptyStateProps extends Omit<ComponentProps<'div'>, 'title'> {
  /** A decorative icon from `@se/ui/icons` evoking the missing content. */
  icon?: LucideIcon | undefined;
  /** What is missing, e.g. « Aucun média ». */
  title: string;
  /** What to do about it, e.g. « Importez des fichiers pour commencer. ». */
  description?: ReactNode | undefined;
  /** Buttons that resolve the empty state (import, create…). */
  actions?: ReactNode | undefined;
  /** `sm` for panels (default), `md` for full screens such as the home page. */
  size?: 'sm' | 'md' | undefined;
}

/**
 * The placeholder of an area with nothing to show yet: icon, title, short explanation and the
 * action that fills it. Centred in its container.
 */
export function EmptyState({
  icon,
  title,
  description,
  actions,
  size = 'sm',
  className,
  ...props
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex size-full flex-col items-center justify-center p-4 text-center',
        size === 'md' ? 'gap-3' : 'gap-2',
        className,
      )}
      {...props}
    >
      {icon !== undefined && (
        <div
          className={cn(
            'flex items-center justify-center rounded-lg bg-control text-text-subtle',
            size === 'md' ? 'size-12' : 'size-9',
          )}
        >
          <Icon icon={icon} size={size === 'md' ? 'lg' : 'md'} />
        </div>
      )}
      <p className={cn('font-medium text-text', size === 'md' ? 'text-lg' : 'text-sm')}>{title}</p>
      {description !== undefined && (
        <p className={cn('max-w-72 text-text-muted', size === 'md' ? 'text-sm' : 'text-xs')}>
          {description}
        </p>
      )}
      {actions !== undefined && <div className="mt-1 flex items-center gap-2">{actions}</div>}
    </div>
  );
}
