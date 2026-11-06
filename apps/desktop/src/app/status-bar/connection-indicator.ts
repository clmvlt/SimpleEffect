import type { ConnectionState } from '@se/protocol';
import type { StatusDotProps } from '@se/ui';

/** How the status bar shows a core connection state. */
export interface ConnectionIndicator {
  readonly tone: NonNullable<StatusDotProps['tone']>;
  readonly labelKey: `statusBar.core.${'connecting' | 'connected' | 'reconnecting' | 'closed'}`;
  /** Ongoing states breathe. */
  readonly pulse: boolean;
}

const INDICATORS: Record<ConnectionState, ConnectionIndicator> = {
  connecting: { tone: 'warning', labelKey: 'statusBar.core.connecting', pulse: true },
  open: { tone: 'success', labelKey: 'statusBar.core.connected', pulse: false },
  reconnecting: { tone: 'warning', labelKey: 'statusBar.core.reconnecting', pulse: true },
  closed: { tone: 'danger', labelKey: 'statusBar.core.closed', pulse: false },
};

/** Dot colour, label key and animation of a connection state. */
export function connectionIndicator(state: ConnectionState): ConnectionIndicator {
  return INDICATORS[state];
}
