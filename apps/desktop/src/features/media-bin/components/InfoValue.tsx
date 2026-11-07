import type { ReactNode } from 'react';

/** Props of {@link InfoValue}. */
export interface InfoValueProps {
  readonly children: ReactNode;
  readonly title?: string | undefined;
}

/** A value of the details panel, selectable for copying. */
export function InfoValue({ children, title }: InfoValueProps) {
  return (
    <span className="block truncate text-xs text-text select-text" title={title}>
      {children}
    </span>
  );
}
