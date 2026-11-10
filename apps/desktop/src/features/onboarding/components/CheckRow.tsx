import { StatusDot } from '@se/ui';
import type { ReactNode } from 'react';

import type { CheckTone } from '../services/ai-step';

/** Props of {@link CheckRow}. */
export interface CheckRowProps {
  /** `success` found, `warning` usable with limits, `danger` missing, `neutral` checking. */
  readonly tone: CheckTone;
  readonly label: string;
  /** What was found, or what to do. */
  readonly children?: ReactNode;
}

/** One checked item of the wizard: a status dot, its name and the details. */
export function CheckRow({ tone, label, children }: CheckRowProps) {
  return (
    <li className="flex items-start gap-2 py-1.5" data-check-tone={tone}>
      <StatusDot tone={tone} pulse={tone === 'neutral'} className="mt-1.5 shrink-0" />
      <div className="flex min-w-0 flex-col gap-0.5">
        <span className="text-sm text-text">{label}</span>
        {children !== undefined && <div className="text-xs text-text-muted">{children}</div>}
      </div>
    </li>
  );
}
